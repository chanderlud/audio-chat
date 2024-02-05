use cpal::SupportedStreamConfig;

include!(concat!(env!("OUT_DIR"), "/audio_chat.items.rs"));

impl Hello {
    pub(crate) fn new(port: u16) -> Self {
        Self { port: port as u32 }
    }
}

impl From<&SupportedStreamConfig> for InputConfig {
    fn from(value: &SupportedStreamConfig) -> Self {
        Self {
            channels: value.channels() as u32,
            sample_rate: value.sample_rate().0,
            sample_format: value.sample_format().to_string(),
        }
    }
}
