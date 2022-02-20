use super::SoundParams;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use udon::source::{ChannelCount, Sample, SampleRate, Source};

const INIT_CAPACITY: usize = 16;

/// An audio mixer compatible with udon and based on udon's built-in Mixer type, but designed specifically for GM8.
pub struct Mixer {
    channels: ChannelCount,
    sample_rate: SampleRate,
    sources: Vec<(Box<dyn Source + Send + 'static>, Arc<SoundParams>, i32)>,
    exclusive_source: Option<(Box<dyn Source + Send + 'static>, i32)>,
    global_volume: Arc<AtomicU32>,
    input_buffer: Vec<Sample>,
    receiver: Receiver<Command>,
}

enum Command {
    Add { source: Box<dyn Source + Send + 'static>, params: Arc<SoundParams>, id: i32 },
    AddExclusive { source: Box<dyn Source + Send + 'static>, id: i32 },
    Stop(i32),
    StopAll,
}

/// Returned from Mixer::new(), and permanently associated with the Mixer created alongside it.
/// Used for dynamically adding sounds to the Mixer with `handle.add()`
pub struct MixerHandle(Sender<Command>);

/// Error type for Mixer calls
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// Indicates that something could not be sent to the Mixer via a MixerHandle.
    /// This usually happens because the Mixer no longer exists.
    SendError,
}

impl Mixer {
    pub fn new(sample_rate: SampleRate, channels: ChannelCount, global_volume: Arc<AtomicU32>) -> (Self, MixerHandle) {
        let (sender, receiver) = mpsc::channel();
        (
            Self {
                channels,
                sample_rate,
                sources: Vec::with_capacity(INIT_CAPACITY),
                exclusive_source: None,
                global_volume,
                input_buffer: Vec::new(),
                receiver,
            },
            MixerHandle(sender),
        )
    }
}

impl Source for Mixer {
    fn write_samples(&mut self, buffer: &mut [Sample]) -> usize {
        // Check for new incoming commands
        while let Ok(cmd) = self.receiver.try_recv() {
            match cmd {
                Command::Add { source, params, id } => self.sources.push((source, params, id)),
                Command::AddExclusive { source, id } => self.exclusive_source = Some((source, id)),
                Command::Stop(id) => {
                    self.sources.retain(|(_, _, x)| *x != id);
                    if let Some((_, x)) = &self.exclusive_source {
                        if *x == id {
                            self.exclusive_source = None;
                        }
                    }
                },
                Command::StopAll => {
                    self.sources.clear();
                    self.exclusive_source = None;
                },
            }
        }

        if let Some((source, _)) = &mut self.exclusive_source {
            let count = source.write_samples(buffer);
            if buffer.len() != count {
                buffer[count..].iter_mut().for_each(|x| *x = 0.0);
                self.exclusive_source = None;
            }
        } else {
            buffer.iter_mut().for_each(|x| *x = 0.0);
        }

        let input_buffer = &mut self.input_buffer;
        input_buffer.resize_with(buffer.len(), Default::default);
        let global_volume = f32::from_bits(self.global_volume.load(Ordering::Acquire));

        RetainMut::retain_mut(&mut self.sources, |(source, params, _)| {
            let volume = f32::from_bits(params.volume.load(Ordering::Acquire));
            let count = source.write_samples(input_buffer);

            for (in_sample, out_sample) in input_buffer.iter().take(count).copied().zip(buffer.iter_mut()) {
                *out_sample += in_sample * volume * global_volume;
            }

            count == input_buffer.len()
        });

        buffer.len()
    }

    fn channel_count(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn reset(&mut self) {
        self.sources.clear()
    }
}

impl MixerHandle {
    /// Adds a sound to be mixed, along with its ID and atomic params
    pub fn add(&self, source: impl Source + Send + 'static, params: Arc<SoundParams>, id: i32) -> Result<(), Error> {
        let command = Command::Add { source: Box::new(source), params, id };
        self.0.send(command).map_err(|_| Error::SendError)
    }

    /// Adds an exclusive sound
    pub fn add_exclusive(&self, source: impl Source + Send + 'static, id: i32) -> Result<(), Error> {
        let command = Command::AddExclusive { source: Box::new(source), id };
        self.0.send(command).map_err(|_| Error::SendError)
    }

    /// Stops all sounds with a certain ID
    pub fn stop(&self, id: i32) -> Result<(), Error> {
        self.0.send(Command::Stop(id)).map_err(|_| Error::SendError)
    }

    /// Stops all sounds
    pub fn stop_all(&self) -> Result<(), Error> {
        self.0.send(Command::StopAll).map_err(|_| Error::SendError)
    }
}

trait RetainMut<T> {
    fn retain_mut(&mut self, f: impl FnMut(&mut T) -> bool);
}

impl<T> RetainMut<T> for Vec<T> {
    fn retain_mut(&mut self, mut f: impl FnMut(&mut T) -> bool) {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
}
