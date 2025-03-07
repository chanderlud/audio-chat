use crate::api::audio_chat::ProcessorMessage;
use bytes::Bytes;
use kanal::{Receiver, Sender};
use log::info;
use sea_codec::decoder::SeaDecoder;
use sea_codec::encoder::{EncoderSettings, SeaEncoder};
use std::io::{Read, Result, Write};

struct ChannelReader {
    receiver: Receiver<ProcessorMessage>,
    buffer: Bytes,
    offset: usize, // tracks how much of the buffer has been read
}

impl Read for ChannelReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // fetch new data if buffer is empty
        if self.offset >= self.buffer.len() {
            match self.receiver.recv() {
                Ok(ProcessorMessage::Data(bytes)) => {
                    self.buffer = bytes;
                    self.offset = 0;
                }
                Err(_) => return Ok(0), // channel closed
                Ok(ProcessorMessage::Silence) => return Ok(0), // unreachable
            }
        }

        // determine how much to read
        let len = std::cmp::min(buf.len(), self.buffer.len() - self.offset);
        buf[..len].copy_from_slice(&self.buffer[self.offset..self.offset + len]);

        // update read position
        self.offset += len;

        Ok(len)
    }
}

struct ChannelWriter {
    sender: Sender<ProcessorMessage>,
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // clone and send data as a `Bytes` object
        self.sender
            .send(ProcessorMessage::Data(Bytes::copy_from_slice(buf)))
            .map(|_| buf.len())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "channel closed"))
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

pub(crate) fn encoder(
    receiver: Receiver<ProcessorMessage>,
    sender: Sender<ProcessorMessage>,
    sample_rate: u32,
    vbr: bool,
    residual_bits: f32,
) {
    let reader = ChannelReader {
        receiver,
        buffer: Default::default(),
        offset: 0,
    };

    let writer = ChannelWriter { sender };

    let settings = EncoderSettings {
        frames_per_chunk: 480,
        scale_factor_frames: 20,
        residual_bits,
        vbr,
        ..Default::default()
    };

    let mut encoder = SeaEncoder::new(1, sample_rate, None, settings, reader, writer).unwrap();

    while encoder.encode_frame().is_ok() {}

    info!("Encoder finished");
}

pub(crate) fn decoder(receiver: Receiver<ProcessorMessage>, sender: Sender<ProcessorMessage>) {
    let reader = ChannelReader {
        receiver,
        buffer: Default::default(),
        offset: 0,
    };

    let writer = ChannelWriter { sender };

    let mut decoder = SeaDecoder::new(reader, writer).unwrap();

    while decoder.decode_frame().is_ok_and(|d| d) {}

    info!("Decoder finished");
}
