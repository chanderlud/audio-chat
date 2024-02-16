use std::mem;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Host, SampleFormat};
use flutter_rust_bridge::spawn;
use flutter_rust_bridge::{frb, spawn_blocking_with};
use kanal::{bounded_async, Sender};
use log::{debug, error};
use nnnoiseless::FRAME_SIZE;
use rubato::Resampler;
use tokio::select;
use tokio::sync::Notify;

use crate::api::audio_chat::{
    db_to_multiplier, get_output_device, mul, resampler_factory, DeviceName, SendStream,
};
use crate::api::error::Error;
use crate::api::items::AudioHeader;
use crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER;

#[frb(opaque)]
pub struct SoundPlayer {
    /// A multiplier applied to sound effects
    output_volume: Arc<AtomicF32>,

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

    pub async fn play(&self, bytes: Vec<u8>) -> SoundHandle {
        let cancel = Arc::new(Notify::new());
        let output_volume = self.output_volume.clone();
        let host = self.host.clone();
        let cancel_clone = cancel.clone();
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
}

#[frb(opaque)]
pub struct SoundHandle {
    cancel: Arc<Notify>,
}

impl SoundHandle {
    #[frb(sync)]
    pub fn cancel(&self) {
        self.cancel.notify_waiters();
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
    let output_device = get_output_device(&output_device, &host).await?;
    let output_config = output_device.default_output_config()?;

    let spec = AudioHeader::from(&bytes[0..44]);
    debug!("Audio header: {:?}", spec);

    let sample_format = SampleFormat::I16; // TODO get the sample format from the header
    let sample_size = sample_format.sample_size();

    let ratio = output_config.sample_rate().0 as f64 / spec.sample_rate as f64;

    let (processed_sender, processed_receiver) = bounded_async::<Vec<f32>>(1_000);

    let output_channels = output_config.channels() as usize;
    let sync_receiver = processed_receiver.to_sync();

    let output_stream = SendStream {
        stream: output_device.build_output_stream(
            &output_config.into(),
            move |output: &mut [f32], _: &_| {
                for frame in output.chunks_mut(output_channels) {
                    let samples = sync_receiver.recv().unwrap();
                    frame.copy_from_slice(&samples);
                }
            },
            move |err| {
                error!("Error in output stream: {}", err);
            },
            None,
        )?,
    };

    output_stream.stream.play()?;

    let processor_handle = std::thread::spawn(move || {
        processor(
            bytes,
            sample_format,
            spec,
            sample_size,
            output_volume,
            processed_sender.to_sync(),
            output_channels,
            ratio,
        )
    });

    let processor_future = spawn_blocking_with(
        move || processor_handle.join(),
        FLUTTER_RUST_BRIDGE_HANDLER.thread_pool(),
    );

    select! {
        _ = cancel.notified() => {
            debug!("Sound cancelled");
        }
        result = processor_future => {
            debug!("Sound finished {:?}", result);
        }
    }

    Ok(())
}

/// Processes the WAV data
fn processor(
    bytes: Vec<u8>,
    sample_format: SampleFormat,
    spec: AudioHeader,
    sample_size: usize,
    output_volume: Arc<AtomicF32>,
    processed_sender: Sender<Vec<f32>>,
    output_channels: usize,
    ratio: f64,
) -> Result<(), Error> {
    // rubato requires 10 extra bytes in the output buffer as a safety margin
    let post_len = (FRAME_SIZE as f64 * ratio + 10.0) as usize;

    // the output for the resampler
    let mut post_buf = vec![vec![0_f32; post_len]; spec.channels as usize];
    // the input for the resampler
    let mut pre_buf = vec![vec![0_f32; FRAME_SIZE]; spec.channels as usize];

    let mut resampler = resampler_factory(ratio, spec.channels as usize)?;

    for chunk in bytes[44..].chunks(FRAME_SIZE * sample_size * spec.channels as usize) {
        match sample_format {
            SampleFormat::I16 => {
                for (i, sample) in chunk.chunks(2 * spec.channels as usize).enumerate() {
                    for (j, channel) in sample.chunks(2).enumerate() {
                        let sample =
                            i16::from_le_bytes([channel[0], channel[1]]) as f32 / i16::MAX as f32;
                        pre_buf[j][i] = sample;
                    }
                }
            }
            _ => unimplemented!(),
        }

        let output_volume = output_volume.load(Relaxed);

        let (target_buffer, len) = if let Some(resampler) = &mut resampler {
            let processed = resampler.process_into_buffer(&pre_buf, &mut post_buf, None)?;
            (&mut post_buf, processed.1)
        } else {
            (&mut pre_buf, FRAME_SIZE)
        };

        for channel in target_buffer.iter_mut() {
            mul(&mut channel[..len], output_volume);
        }

        let mut buffer = Vec::with_capacity(output_channels);

        for i in 0..len {
            for j in 0..output_channels {
                // this handles when there are more output channels than input channels
                if j > spec.channels as usize {
                    buffer.push(target_buffer[0][i]);
                } else {
                    buffer.push(target_buffer[j][i]);
                }
            }

            // take this buffer and send it to the output
            let buffer = mem::take(&mut buffer);
            processed_sender.send(buffer)?;
        }
    }

    Ok(())
}
