mod mp3;

use serde::{Serialize, Deserialize};
use udon::{
    cycle::Cycle,
    mixer::{Mixer, MixerHandle},
    rechanneler::Rechanneler,
    resampler::Resampler,
    session::{Api, Session},
    source::{ChannelCount, SampleRate},
    wav::WavPlayer
};

use self::mp3::Mp3Player;

#[derive(Clone, Serialize, Deserialize)]
pub struct Mp3Handle(Mp3Player);

#[derive(Clone, Serialize, Deserialize)]
pub struct WavHandle(WavPlayer);

pub struct AudioManager {
    mixer_handle: MixerHandle,
    mixer_channel_count: ChannelCount,
    mixer_sample_rate: SampleRate,
    do_output: bool,
}

impl AudioManager {
    pub fn new(do_output: bool) -> Self {
        let session = Session::new(Api::Wasapi).unwrap();
        let device = session.default_output_device().unwrap();
        let sample_rate = device.sample_rate();
        let channel_count = device.channel_count();
        let (mixer, mixer_handle) = Mixer::new(sample_rate, channel_count);

        std::thread::spawn(move || {
            let stream = session.open_output_stream(device).unwrap();
            stream.play(mixer).unwrap();
        });

        Self {
            mixer_handle,
            mixer_channel_count: channel_count,
            mixer_sample_rate: sample_rate,
            do_output,
        }
    }

    pub fn add_mp3(&mut self, file: Box<[u8]>) -> Option<Mp3Handle> {
        Mp3Player::new(file).map(Mp3Handle).ok()
    }

    pub fn add_wav(&mut self, file: Box<[u8]>) -> Option<WavHandle> {
        WavPlayer::new(file).map(WavHandle).ok()
    }

    pub fn play_mp3(&self, handle: &Mp3Handle) {
        if self.do_output {
            let _ = self.mixer_handle.add(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            );
        }
    }

    pub fn play_wav(&self, handle: &WavHandle) {
        if self.do_output {
            let _ = self.mixer_handle.add(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            );
        }
    }

    pub fn loop_mp3(&self, handle: &Mp3Handle) {
        if self.do_output {
            let _ = self.mixer_handle.add(Cycle::new(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            ));
        }
    }

    pub fn loop_wav(&self, handle: &WavHandle) {
        if self.do_output {
            let _ = self.mixer_handle.add(Cycle::new(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            ));
        }
    }
}
