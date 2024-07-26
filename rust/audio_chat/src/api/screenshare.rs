use std::fmt::Display;
use std::process::{ExitStatus, Output, Stdio};
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use crate::api::audio_chat::{Capabilities, RecordingConfig};
use libp2p::futures::{AsyncReadExt as ReadExt, AsyncWriteExt as WriteExt};
use libp2p::Stream;
use log::{error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::select;
use tokio::sync::Notify;

use crate::api::error::{Error, ErrorKind};

type Result<T> = std::result::Result<T, Error>;

const BUFFER_SIZE: usize = 512;
// TODO this is windows only i think
const CREATION_FLAGS: u32 = 0x08000000;

impl Capabilities {
    pub(crate) async fn new() -> Self {
        let codec_regex = Regex::new("V....[D.] ([^= ]+)\\s+(.+)").unwrap();

        let encoders_result = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-encoders")
            .creation_flags(CREATION_FLAGS)
            .output()
            .await;

        let decoders_result = Command::new("ffplay")
            .arg("-hide_banner")
            .arg("-decoders")
            .creation_flags(CREATION_FLAGS)
            .output()
            .await;

        match (encoders_result, decoders_result) {
            (Ok(encoders_output), Ok(decoders_output)) => {
                let encoders = parse_codecs(encoders_output, &codec_regex)
                    .into_iter()
                    .filter_map(|codec| Encoder::from_str(&codec).ok())
                    .collect();
                let decoders = parse_codecs(decoders_output, &codec_regex)
                    .into_iter()
                    .filter_map(|codec| Decoder::from_str(&codec).ok())
                    .collect();

                Self {
                    available: true,
                    encoders,
                    // TODO verify decoders here
                    decoders,
                    devices: Device::devices(),
                }
            }
            _ => Self {
                available: false,
                encoders: Vec::new(),
                decoders: Vec::new(),
                devices: Device::devices(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Device {
    DirectShow,
    GdiGrab,
    DesktopDuplication,
    AVFoundation(Vec<String>),
    X11Grab,
}

impl Device {
    #[cfg(target_os = "windows")]
    fn devices() -> Vec<Self> {
        vec![Self::DesktopDuplication, Self::GdiGrab, Self::DirectShow]
    }

    #[cfg(target_os = "macos")]
    fn devices() -> Vec<Self> {
        let devices_output = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-f")
            .arg("avfoundation")
            .arg("-list_devices")
            .arg("true")
            .arg("-i")
            .arg("\"\"")
            .output()
            .await;

        // TODO parse the output and use it for devices

        vec![Self::AVFoundation]
    }

    #[cfg(target_os = "linux")]
    fn devices() -> Vec<Self> {
        vec![Self::X11Grab]
    }

    fn to_args(&self, encoder: Encoder) -> Vec<&str> {
        // TODO figure out a way to only add the video size for encoders if needed
        match self {
            Self::DesktopDuplication => match encoder {
                Encoder::H264Nvenc | Encoder::H264Qsv => vec![
                    "-init_hw_device",
                    "d3d11va",
                    "-filter_complex",
                    "ddagrab=video_size=1920x1080",
                ],
                Encoder::HevcNvenc | Encoder::Av1Nvenc => {
                    vec!["-init_hw_device", "d3d11va", "-filter_complex", "ddagrab=0"]
                }
                _ => vec![
                    "-init_hw_device",
                    "d3d11va",
                    "-filter_complex",
                    "ddagrab=0,hwdownload,format=bgra",
                ],
            },
            Self::GdiGrab => match encoder {
                Encoder::H264Nvenc | Encoder::H264Qsv => vec![
                    "-f",
                    "gdigrab",
                    "-framerate",
                    "30",
                    "-video_size",
                    "1920x1080",
                    "-i",
                    "desktop",
                ],
                _ => vec!["-f", "gdigrab", "-framerate", "30", "-i", "desktop"],
            },
            _ => todo!(),
        }
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectShow => write!(f, "DirectShow"),
            Self::GdiGrab => write!(f, "GDI Grab"),
            Self::DesktopDuplication => write!(f, "Desktop Duplication"),
            Self::AVFoundation(devices) => write!(f, "AVFoundation: {:?}", devices),
            Self::X11Grab => write!(f, "X11 Grab"),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) enum Encoder {
    Libx264,
    H264Nvenc,
    H264Amf,
    H264Qsv,
    H264Vaapi,
    Libx265,
    HevcNvenc,
    HevcAmf,
    HevcQsv,
    HevcVaapi,
    Av1Nvenc,
    Av1Amf,
    Av1Qsv,
    Av1Vaapi,
}

impl Into<&'static str> for Encoder {
    fn into(self) -> &'static str {
        match self {
            Self::Libx264 => "libx264",
            Self::H264Nvenc => "h264_nvenc",
            Self::H264Amf => "h264_amf",
            Self::H264Qsv => "h264_qsv",
            Self::H264Vaapi => "h264_vaapi",
            Self::Libx265 => "libx265",
            Self::HevcNvenc => "hevc_nvenc",
            Self::HevcAmf => "hevc_amf",
            Self::HevcQsv => "hevc_qsv",
            Self::HevcVaapi => "hevc_vaapi",
            Self::Av1Nvenc => "av1_nvenc",
            Self::Av1Amf => "av1_amf",
            Self::Av1Qsv => "av1_qsv",
            Self::Av1Vaapi => "av1_vaapi",
        }
    }
}

impl Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}

impl FromStr for Encoder {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "libx264" => Ok(Self::Libx264),
            "h264_nvenc" => Ok(Self::H264Nvenc),
            "h264_amf" => Ok(Self::H264Amf),
            "h264_qsv" => Ok(Self::H264Qsv),
            "h264_vaapi" => Ok(Self::H264Vaapi),
            "libx265" => Ok(Self::Libx265),
            "hevc_nvenc" => Ok(Self::HevcNvenc),
            "hevc_amf" => Ok(Self::HevcAmf),
            "hevc_qsv" => Ok(Self::HevcQsv),
            "hevc_vaapi" => Ok(Self::HevcVaapi),
            "av1_nvenc" => Ok(Self::Av1Nvenc),
            "av1_amf" => Ok(Self::Av1Amf),
            "av1_qsv" => Ok(Self::Av1Qsv),
            "av1_vaapi" => Ok(Self::Av1Vaapi),
            _ => Err(()),
        }
    }
}

impl Encoder {
    /// returns the valid decoders for this encoder in preferred order
    fn decoders(&self) -> Vec<Decoder> {
        match self {
            Self::Libx264 | Self::H264Nvenc | Self::H264Amf | Self::H264Qsv | Self::H264Vaapi => {
                vec![Decoder::H264Cuvid, Decoder::H264Qsv, Decoder::H264]
            }
            Self::Libx265 | Self::HevcNvenc | Self::HevcAmf | Self::HevcQsv | Self::HevcVaapi => {
                vec![Decoder::HevcCuvid, Decoder::HevcQsv, Decoder::Hevc]
            }
            Self::Av1Nvenc | Self::Av1Amf | Self::Av1Qsv | Self::Av1Vaapi => {
                vec![Decoder::Av1Cuvid, Decoder::Av1Qsv]
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Decoder {
    H264,
    H264Cuvid,
    H264Qsv,
    Hevc,
    HevcCuvid,
    HevcQsv,
    Av1Cuvid,
    Av1Qsv,
}

impl Into<&'static str> for Decoder {
    fn into(self) -> &'static str {
        match self {
            Self::H264 => "h264",
            Self::H264Cuvid => "h264_cuvid",
            Self::Hevc => "hevc",
            Self::HevcCuvid => "hevc_cuvid",
            Self::H264Qsv => "h264_qsv",
            Self::HevcQsv => "hevc_qsv",
            Self::Av1Cuvid => "av1_cuvid",
            Self::Av1Qsv => "av1_qsv",
        }
    }
}

impl FromStr for Decoder {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "h264" => Ok(Self::H264),
            "h264_cuvid" => Ok(Self::H264Cuvid),
            "h264_qsv" => Ok(Self::H264Qsv),
            "hevc" => Ok(Self::Hevc),
            "hevc_cuvid" => Ok(Self::HevcCuvid),
            "hevc_qsv" => Ok(Self::HevcQsv),
            "av1_cuvid" => Ok(Self::Av1Cuvid),
            "av1_qsv" => Ok(Self::Av1Qsv),
            _ => Err(()),
        }
    }
}
impl RecordingConfig {
    fn make_command(&self, test: bool) -> Command {
        let mut command = Command::new("ffmpeg");
        command.args(self.device.to_args(self.encoder));

        // sets the video size if specified
        if let Some(height) = self.height {
            command.arg("-vf");
            command.arg(format!("trunc(oh*a/2)*2:{}", height));
        }

        if test {
            command.arg("-frames:v");
            command.arg("1");
        }

        command.args(&[
            "-c:v",
            self.encoder.into(),
            "-delay",
            "0",
            "-b:v",
            self.bitrate.to_string().as_str(),
            "-bufsize",
            "1M",
            "-f",
            "mpegts",
            "-",
        ]);

        command
    }

    pub(crate) async fn test_config(&self) -> Result<ExitStatus> {
        let mut child = self
            .make_command(true)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATION_FLAGS)
            .spawn()?;

        child.wait().await.map_err(Into::into)
    }
}

struct PlaybackConfig {
    decoder: Decoder,
}

impl PlaybackConfig {
    fn make_command(&self) -> Command {
        let mut command = Command::new("ffplay");

        command.args(&["-vcodec", self.decoder.into(), "-f", "mpegts", "-i", "-"]);

        command
    }
}

pub(crate) async fn record(
    mut stream: Stream,
    stop: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
    config: RecordingConfig,
) -> Result<()> {
    warn!("Starting screen recording with config: {:?}", config);

    let mut child = config
        .make_command(false)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(CREATION_FLAGS)
        .spawn()?;

    let mut stdout = child.stdout.take().expect("Failed to capture stdout");

    let future = async {
        let mut frame = [0u8; BUFFER_SIZE];

        while let Ok(read) = stdout.read(&mut frame).await {
            if read == 0 {
                break;
            }

            bandwidth.fetch_add(read, Relaxed);
            if let Err(error) = WriteExt::write(&mut stream, &frame[..read]).await {
                error!("Failed to write frame to ffmpeg {}", error);
                break;
            }
        }
    };

    select! {
        _ = future => {
            stop.notify_waiters();
            info!("Recording finished");
        }
        _ = stop.notified() => {
            info!("Recording stopped");
        }
    }

    _ = child.kill().await;
    Ok(())
}

pub(crate) async fn playback(
    mut stream: Stream,
    stop: Arc<Notify>,
    bandwidth: Arc<AtomicUsize>,
    encoder: String,
    width: u32,
    height: u32,
) -> Result<()> {
    info!("Starting screen playback");
    let encoder = Encoder::from_str(&encoder).map_err(|_| ErrorKind::InvalidEncoder)?;
    let decoders = encoder.decoders();

    // TODO intelligently chose a decoder instead of using the first one
    let config = PlaybackConfig {
        decoder: decoders.into_iter().next().unwrap(),
    };

    let mut child = config
        .make_command()
        .args(&[
            "-x",
            &width.to_string(),
            "-y",
            &height.to_string(),
            "-flags",
            "low_delay",
            "-analyzeduration",
            "1",
            // TODO -framedrop
            "-window_title",
            "Audio Chat Screenshare",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATION_FLAGS)
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to capture stdin");

    let future = async {
        let mut buffer = [0u8; BUFFER_SIZE];

        while let Ok(read) = ReadExt::read(&mut stream, &mut buffer).await {
            if read == 0 {
                break;
            }

            bandwidth.fetch_add(read, Relaxed);
            if let Err(error) = stdin.write(&buffer[..read]).await {
                error!("Failed to write frame to ffmpeg {}", error);
                break;
            }
        }
    };

    select! {
        _ = future => {
            info!("Playback finished");
        }
        _ = stop.notified() => {
            info!("Playback stopped");
        }
    }

    _ = child.kill().await;
    Ok(())
}

fn parse_codecs(output: Output, regex: &Regex) -> Vec<String> {
    let output_str = String::from_utf8_lossy(&output.stdout);

    regex
        .captures_iter(&output_str)
        .filter_map(|cap| cap.get(1))
        .map(|cap| cap.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capabilities() {
        let capabilities = Capabilities::new().await;
        debug!("{:?}", capabilities);

        let encoders = [
            Encoder::H264Nvenc,
            Encoder::HevcNvenc,
            Encoder::Av1Nvenc,
            Encoder::Av1Qsv,
            Encoder::Av1Vaapi,
            Encoder::Av1Amf,
            Encoder::H264Amf,
            Encoder::H264Qsv,
            Encoder::H264Vaapi,
            Encoder::Libx264,
            Encoder::Libx265,
            Encoder::HevcAmf,
            Encoder::HevcQsv,
            Encoder::HevcVaapi,
        ];

        for encoder in encoders {
            let config = RecordingConfig {
                encoder,
                device: Device::DesktopDuplication,
                bitrate: 2_000_000,
                height: None,
                framerate: 30,
            };

            let status = config.test_config().await;
            debug!("{:?}: {:?}", encoder, status);
        }
    }
}
