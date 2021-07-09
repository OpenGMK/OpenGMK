use rmp3::{Decoder, Frame, RawDecoder};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use udon::source::{ChannelCount, SampleRate, Sample, Source};

/// This MP3 player is deliberately designed to emulate bugs in GameMaker 8. Specifically:
/// - the sample rate from the first frame is assumed to be the sample rate of the whole file
/// - any frames with a different channel count than the first frame are skipped immediately
#[derive(Clone, Serialize, Deserialize)]
pub struct MP3Player {
    file: Arc<[u8]>,
    channels: ChannelCount,
    sample_rate: SampleRate,
    length: usize, // Pre-calculated number of samples that will actually be output by GM8
    decoder: RawDecoder,
    offset: usize,
    buffer: [Sample; rmp3::MAX_SAMPLES_PER_FRAME],
}

pub enum Error {
    InvalidFile,
    InvalidDetails,
    NoDetails,
}

impl MP3Player {
    pub fn new(file: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let file = file.into();
        if let Some((channels, sample_rate)) = details(Decoder::new(&file)) {
            let mut length = 0;
            let mut decoder = Decoder::new(&file);
            while let Some(frame) = decoder.next() {
                if let Frame::Audio(audio) = frame {
                    if audio.channels() == channels {
                        length += audio.sample_count();
                    }
                }
            }

            let buffer = Vec::with_capacity(rmp3::MAX_SAMPLES_PER_FRAME);
            unsafe { buffer.set_len(rmp3::MAX_SAMPLES_PER_FRAME); }

            Ok(Self {
                file: file.into(),
                channels: ChannelCount::new(channels).ok_or(Error::InvalidDetails)?,
                sample_rate: SampleRate::new(sample_rate).ok_or(Error::InvalidDetails)?,
                decoder: RawDecoder::new(),
                length,
                offset: 0,
                buffer,
            })
        }
        else {
            Err(Error::NoDetails)
        }
    }

    /// The number of samples which will actually be played out. Divide by sample rate to get length in seconds.
    #[inline(always)]
    pub fn length(&self) -> usize {
        self.length
    }
}

impl Source for MP3Player {
    #[inline(always)]
    fn channel_count(&self) -> ChannelCount {
        self.channels
    }

    #[inline(always)]
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn write_samples(&mut self, buffer: &mut [Sample]) -> usize {
        if let Some((frame, bytes_consumed)) = self.decoder.next(&self.file[self.offset], &mut self.buffer) {
            self.offset += bytes_consumed;
            buffer
        }
        buffer.len()
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.offset = 0;
    }
}

fn details(mut decoder: Decoder) -> Option<(u16, u32)> {
    while let Some(frame) = decoder.peek() {
        if let Frame::Audio(audio) = frame {
            return Some((audio.channels(), audio.sample_rate()));
        } else {
            decoder.skip();
        }
    }
    None
}
