use std::mem;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Host, SampleFormat};
use flutter_rust_bridge::spawn;
use flutter_rust_bridge::{frb, spawn_blocking_with};
use kanal::{bounded_async, Sender};
use log::error;
use nnnoiseless::FRAME_SIZE;
use rubato::Resampler;
use tokio::select;
use tokio::sync::Notify;
use tokio::time::sleep;

use crate::api::audio_chat::{
    db_to_multiplier, get_output_device, mul, resampler_factory, DeviceName, SendStream,
};
use crate::api::error::{Error, ErrorKind};
use crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER;
use messages::AudioHeader;

#[frb(opaque)]
pub struct SoundPlayer {
    /// A multiplier applied to sound effects
    output_volume: Arc<AtomicF32>,

    /// The output device
    output_device: DeviceName,

    /// The cpal host
    host: Arc<Host>,
}

impl SoundPlayer {
    #[frb(sync)]
    pub fn new(output_volume: f32) -> SoundPlayer {
        let host = cpal::default_host();

        Self {
            output_volume: Arc::new(AtomicF32::new(db_to_multiplier(output_volume))),
            output_device: Default::default(),
            host: Arc::new(host),
        }
    }

    #[frb(sync)]
    pub fn host(&self) -> Arc<Host> {
        self.host.clone()
    }

    /// Public play function
    pub async fn play(&self, bytes: Vec<u8>) -> SoundHandle {
        let cancel = Arc::new(Notify::new());
        let cancel_clone = cancel.clone();

        let output_volume = self.output_volume.clone();
        let host = self.host.clone();
        let output_device = self.output_device.clone();

        spawn(async move {
            if let Err(error) =
                play_sound(bytes, cancel_clone, host, output_volume, output_device).await
            {
                error!("Error playing sound: {:?}", error)
            }
        });

        SoundHandle { cancel }
    }

    #[frb(sync)]
    pub fn update_output_volume(&self, volume: f32) {
        self.output_volume.store(db_to_multiplier(volume), Relaxed);
    }

    pub async fn update_output_device(&self, name: Option<String>) {
        *self.output_device.lock().await = name;
    }
}

#[frb(opaque)]
pub struct SoundHandle {
    cancel: Arc<Notify>,
}

impl SoundHandle {
    #[frb(sync)]
    pub fn cancel(&self) {
        self.cancel.notify_one();
    }
}

/// Internal play sound function
async fn play_sound(
    bytes: Vec<u8>,
    cancel: Arc<Notify>,
    host: Arc<Host>,
    output_volume: Arc<AtomicF32>,
    output_device: DeviceName,
) -> Result<(), Error> {
    if bytes.len() < 44 {
        return Err(ErrorKind::InvalidWav.into());
    }

    // get the output device & config
    let output_device = get_output_device(&output_device, &host).await?;
    let output_config = output_device.default_output_config()?;

    // parse the input spec
    let spec = AudioHeader::from(&bytes[0..44]);

    // match the correct sample format
    let sample_format = match spec.sample_format.as_str() {
        "u8" => SampleFormat::U8,
        "i16" => SampleFormat::I16,
        "i32" => SampleFormat::I32,
        "f32" => SampleFormat::F32,
        "f64" => SampleFormat::F64,
        _ => return Err(ErrorKind::UnknownSampleFormat.into()),
    };

    // the resampling ratio used by the processor
    let ratio = output_config.sample_rate().0 as f64 / spec.sample_rate as f64;

    // sends samples from the processor to the output stream
    let (processed_sender, processed_receiver) = bounded_async::<Vec<f32>>(1_000);

    // used to chunk the output buffer correctly
    let output_channels = output_config.channels() as usize;
    // the receiver used by the output stream
    let sync_receiver = processed_receiver.to_sync();
    // keep track of the last samples played
    let mut last_samples = vec![0_f32; output_channels];
    // a counter used for fading out the last samples when the sound is cancelled
    let mut i = 0;
    // used to provide a fade to 0 when the sound is cancelled
    let f32_sample_rate = output_config.sample_rate().0 as f32;

    let output_stream = SendStream {
        stream: output_device.build_output_stream(
            &output_config.into(),
            move |output: &mut [f32], _: &_| {
                for frame in output.chunks_mut(output_channels) {
                    if let Ok(samples) = sync_receiver.recv() {
                        // play the samples
                        frame.copy_from_slice(&samples);
                        last_samples = samples;
                    } else {
                        // fade each sample
                        for sample in &mut last_samples {
                            *sample *= (1_f32 - i as f32 / f32_sample_rate).max(0_f32);
                        }

                        // play the samples
                        frame.copy_from_slice(&last_samples);
                        i += 1; // advance the counter
                    }
                }
            },
            move |err| {
                error!("Error in player stream: {}", err);
            },
            None,
        )?,
    };

    // the sender used by the processor
    let sender = processed_sender.clone_sync();

    let processor_future = spawn_blocking_with(
        move || {
            processor(
                bytes,
                sample_format,
                spec,
                output_volume,
                sender,
                output_channels,
                ratio,
            )
        },
        FLUTTER_RUST_BRIDGE_HANDLER.thread_pool(),
    );

    output_stream.stream.play()?; // play the stream

    select! {
        _ = cancel.notified() => {
            // this causes the stream to begin fading out
            processed_sender.close();
            // we sleep to prevent the stream from being closed while fading
            sleep(std::time::Duration::from_secs(1)).await;
        }
        _ = processor_future => {}
    }

    Ok(())
}

/// Processes the WAV data
fn processor(
    bytes: Vec<u8>,
    sample_format: SampleFormat,
    spec: AudioHeader,
    output_volume: Arc<AtomicF32>,
    processed_sender: Sender<Vec<f32>>,
    output_channels: usize,
    ratio: f64,
) -> Result<(), Error> {
    let sample_size = sample_format.sample_size();
    let channels_usize = spec.channels as usize;

    // the number of samples in the file
    let sample_count = (bytes.len() - 44) / sample_size / channels_usize;
    // the number of audio samples which will be played
    let audio_len = (sample_count as f64 * ratio) as f32;
    let mut position = 0_f32; // the playback position

    // constants used for fading in and out
    let fade_basis = sample_count as f32 / 100_f32;
    let fade_out = fade_basis;
    let fade_in = audio_len - fade_basis;

    // rubato requires 10 extra bytes in the output buffer as a safety margin
    let post_len = (FRAME_SIZE as f64 * ratio + 10.0) as usize;

    // the output for the resampler
    let mut post_buf = vec![vec![0_f32; post_len]; channels_usize];
    // the input for the resampler
    let mut pre_buf = vec![vec![0_f32; FRAME_SIZE]; channels_usize];
    // groups of samples ready to be sent to the output
    let mut out_buf = Vec::with_capacity(output_channels);

    let mut resampler = resampler_factory(ratio, channels_usize, FRAME_SIZE)?;
    let output_volume = output_volume.load(Relaxed);

    for chunk in bytes[44..].chunks(FRAME_SIZE * sample_size * channels_usize) {
        match sample_format {
            SampleFormat::U8 => {
                let float_u8_max = u8::MAX as f32;

                for (i, sample) in chunk.chunks(channels_usize).enumerate() {
                    for (j, channel) in sample.iter().enumerate() {
                        let sample = *channel as f32 / float_u8_max;
                        pre_buf[j][i] = sample;
                    }
                }
            }
            SampleFormat::I16 => {
                let float_i16_max = i16::MAX as f32;

                for (i, sample) in chunk.chunks(2 * channels_usize).enumerate() {
                    for (j, channel) in sample.chunks(2).enumerate() {
                        let sample = i16::from_le_bytes(channel.try_into()?) as f32 / float_i16_max;
                        pre_buf[j][i] = sample;
                    }
                }
            }
            SampleFormat::I32 => {
                let float_i32_max = i32::MAX as f32;

                for (i, sample) in chunk.chunks(4 * channels_usize).enumerate() {
                    for (j, channel) in sample.chunks(4).enumerate() {
                        let sample = i32::from_le_bytes(channel.try_into()?) as f32 / float_i32_max;
                        pre_buf[j][i] = sample;
                    }
                }
            }
            SampleFormat::F32 => {
                for (i, sample) in chunk.chunks(4 * channels_usize).enumerate() {
                    for (j, channel) in sample.chunks(4).enumerate() {
                        let sample = f32::from_le_bytes(channel.try_into()?);
                        pre_buf[j][i] = sample;
                    }
                }
            }
            SampleFormat::F64 => {
                for (i, sample) in chunk.chunks(8 * channels_usize).enumerate() {
                    for (j, channel) in sample.chunks(8).enumerate() {
                        let sample = f64::from_le_bytes(channel.try_into()?) as f32;
                        pre_buf[j][i] = sample;
                    }
                }
            }
            _ => return Err(ErrorKind::UnknownSampleFormat.into()),
        }

        let (target_buffer, len) = if let Some(resampler) = &mut resampler {
            let processed = resampler.process_into_buffer(&pre_buf, &mut post_buf, None)?;
            (&mut post_buf, processed.1)
        } else {
            (&mut pre_buf, FRAME_SIZE)
        };

        for channel in target_buffer.iter_mut() {
            mul(&mut channel[..len], output_volume);
        }

        for i in 0..len {
            let multiplier = if position < audio_len {
                let delta = audio_len - position;

                if delta < fade_out {
                    // calculate fade out multiplier
                    delta / fade_basis
                } else if delta > fade_in {
                    // calculate fade in multiplier
                    position / fade_basis
                } else {
                    1_f32 // no fade in or out
                }
            } else {
                0_f32 // the calculated audio_len is too short
            };

            position += 1_f32; // advance the position

            for j in 0..output_channels {
                // this handles when there are more output channels than input channels
                let sample = if j >= channels_usize {
                    target_buffer[0][i]
                } else {
                    target_buffer[j][i]
                };

                out_buf.push(sample * multiplier);
            }

            // take this buffer and send it to the output
            let buffer = mem::take(&mut out_buf);
            processed_sender.send(buffer)?;
        }
    }

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use fast_log::Config;
//     use log::{LevelFilter, Log};
//     use tokio::fs::File;
//     use tokio::io::AsyncReadExt;
//     use tokio::time::sleep;
//
//     #[tokio::test]
//     async fn test_player() {
//         let logger = fast_log::init(
//             Config::new()
//                 .chan_len(Some(100))
//                 .file("tests.log")
//                 .level(LevelFilter::Debug),
//         )
//         .unwrap();
//
//         let mut wav_bytes = Vec::new();
//         let mut wav_file = File::open("../sounds/outgoing.wav").await.unwrap();
//         wav_file.read_to_end(&mut wav_bytes).await.unwrap();
//
//         let player = super::SoundPlayer::new(1_f32);
//         let handle = player.play(wav_bytes).await;
//
//         sleep(std::time::Duration::from_secs(1)).await;
//         handle.cancel();
//         sleep(std::time::Duration::from_secs(1)).await;
//
//         logger.flush();
//     }
// }
