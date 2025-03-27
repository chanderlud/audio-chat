use libp2p::StreamProtocol;
use nnnoiseless::FRAME_SIZE;
use rubato::{SincInterpolationParameters, SincInterpolationType, WindowFunction};
use std::mem;
use std::time::Duration;

/// The number of bytes in a single network audio frame
pub(crate) const TRANSFER_BUFFER_SIZE: usize = FRAME_SIZE * mem::size_of::<i16>();
/// Parameters used for resampling throughout the application
pub(crate) const RESAMPLER_PARAMETERS: SincInterpolationParameters = SincInterpolationParameters {
    sinc_len: 256,
    f_cutoff: 0.95,
    interpolation: SincInterpolationType::Linear,
    oversampling_factor: 256,
    window: WindowFunction::BlackmanHarris2,
};
/// A timeout used when initializing the call
pub(crate) const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
/// A timeout used to detect temporary network issues
pub(crate) const TIMEOUT_DURATION: Duration = Duration::from_millis(100);
/// the number of frames to hold in a channel
pub(crate) const CHANNEL_SIZE: usize = 2_400;
/// the protocol identifier for audio chat
pub(crate) const CHAT_PROTOCOL: StreamProtocol = StreamProtocol::new("/audio-chat/0.0.1");
pub(crate) const ROOM_PROTOCOL: StreamProtocol = StreamProtocol::new("/audio-chat-room/0.0.1");
#[cfg(target_family = "wasm")]
pub(crate) const SILENCE: [f32; FRAME_SIZE] = [0_f32; FRAME_SIZE];
