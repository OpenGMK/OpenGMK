use crate::types::ArraySerde;
use rmp3::{Decoder, Frame, RawDecoder};
use serde::{Deserialize, Serialize};
use std::{alloc, sync::Arc};
use udon::source::{ChannelCount, Sample, SampleRate, Source};

/// This MP3 player is deliberately designed to emulate bugs in GameMaker 8. Specifically:
/// - the sample rate from the first frame is assumed to be the sample rate of the whole file
/// - any frames with a different channel count than the first frame are skipped immediately
#[derive(Clone, Serialize, Deserialize)]
pub struct Mp3Player {
    file: Arc<[u8]>,
    channels: ChannelCount,
    sample_rate: SampleRate,
    length: usize, // Pre-calculated number of samples that will actually be output by GM8
    #[serde(skip, default = "RawDecoderWrap::new")]
    decoder: RawDecoderWrap,
    offset: usize,
    buffer: Box<ArraySerde<rmp3::Sample, { rmp3::MAX_SAMPLES_PER_FRAME }>>,
    buffer_off: usize,
    buffer_len: usize,
}

struct RawDecoderWrap(RawDecoder);

impl RawDecoderWrap {
    fn new() -> Self {
        Self(RawDecoder::new())
    }
}

impl Clone for RawDecoderWrap {
    fn clone(&self) -> Self {
        Self(RawDecoder::new())
    }
}

pub enum Error {
    InvalidFile,
    InvalidDetails,
    NoDetails,
}

impl Mp3Player {
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

            let buffer = unsafe {
                let layout = alloc::Layout::new::<ArraySerde<rmp3::Sample, { rmp3::MAX_SAMPLES_PER_FRAME }>>();
                let alloc = alloc::alloc(layout);
                if alloc.is_null() {
                    panic!("failed to allocate mp3 decoder buffer");
                }
                Box::from_raw(alloc.cast())
            };

            Ok(Self {
                file: file.into(),
                channels: ChannelCount::new(channels).ok_or(Error::InvalidDetails)?,
                sample_rate: SampleRate::new(sample_rate).ok_or(Error::InvalidDetails)?,
                decoder: RawDecoderWrap(RawDecoder::new()),
                length,
                offset: 0,
                buffer,
                buffer_off: 0,
                buffer_len: 0,
            })
        } else {
            Err(Error::NoDetails)
        }
    }

    /// The number of samples which will actually be played out. Divide by sample rate to get length in seconds.
    #[inline(always)]
    pub fn length(&self) -> usize {
        self.length
    }

    fn flush(&mut self, output: &mut [Sample]) -> usize {
        // get the biggest slice that can be copied directly into `output`
        let mut buffer = &self.buffer[self.buffer_off..self.buffer_off + self.buffer_len];
        if buffer.len() > output.len() {
            buffer = &buffer[..output.len()];
        }

        // copy samples into `output`
        (&mut output[..buffer.len()]).copy_from_slice(buffer);

        // adjust internal state
        self.buffer_len -= buffer.len();
        if self.buffer_len > 0 {
            self.buffer_off += buffer.len();
        } else {
            self.buffer_off = 0;
        }

        // return samples written
        buffer.len()
    }

    fn refill(&mut self) -> bool {
        loop {
            match self.decoder.0.next(&self.file[self.offset..], &mut self.buffer) {
                Some((rmp3::Frame::Audio(audio), bytes_consumed)) => {
                    self.offset += bytes_consumed;
                    if self.channels.get() == audio.channels() {
                        self.buffer_off = 0;
                        self.buffer_len = usize::from(audio.channels()) * audio.sample_count();
                        break true
                    }
                },
                Some((rmp3::Frame::Other(_), bytes_consumed)) => self.offset += bytes_consumed,
                None => {
                    self.buffer_off = 0;
                    self.buffer_len = 0;
                    break false
                },
            }
        }
    }
}

impl Source for Mp3Player {
    #[inline(always)]
    fn channel_count(&self) -> ChannelCount {
        self.channels
    }

    #[inline(always)]
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn write_samples(&mut self, mut buffer: &mut [Sample]) -> usize {
        let mut samples_written = 0usize;
        loop {
            // 😳
            let flushed = self.flush(buffer);
            if buffer.len() == flushed {
                break samples_written + flushed
            }
            buffer = &mut buffer[flushed..];
            samples_written += flushed;

            // 🥤
            if self.buffer_len == 0 {
                if !self.refill() {
                    break samples_written
                }
            }
        }
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.buffer_off = 0;
        self.buffer_len = 0;
        self.offset = 0;
    }
}

fn details(mut decoder: Decoder) -> Option<(u16, u32)> {
    while let Some(frame) = decoder.peek() {
        if let Frame::Audio(audio) = frame {
            return Some((audio.channels(), audio.sample_rate()))
        } else {
            decoder.skip();
        }
    }
    None
}
