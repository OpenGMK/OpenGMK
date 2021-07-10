mod mp3;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
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
pub struct Mp3Handle(Mp3Player, i32);

#[derive(Clone, Serialize, Deserialize)]
pub struct WavHandle(WavPlayer, i32);

pub struct AudioManager {
    mixer_handle: MixerHandle,
    mixer_channel_count: ChannelCount,
    mixer_sample_rate: SampleRate,
    do_output: bool,
    end_times: HashMap<i32, Option<u128>>,
    mp3_end: Option<(i32, Option<u128>)>,
}

impl AudioManager {
    pub fn new(do_output: bool) -> Self {
        // TODO: not all these unwraps
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
            end_times: HashMap::new(),
            mp3_end: None,
        }
    }

    pub fn add_mp3(&mut self, file: Box<[u8]>, sound_id: i32) -> Option<Mp3Handle> {
        Mp3Player::new(file).map(|x| Mp3Handle(x, sound_id)).ok()
    }

    pub fn add_wav(&mut self, file: Box<[u8]>, sound_id: i32) -> Option<WavHandle> {
        WavPlayer::new(file).map(|x| WavHandle(x, sound_id)).ok()
    }

    pub fn play_mp3(&mut self, handle: &Mp3Handle, start_time: u128) {
        let end_time = handle.0.length() as u128 + start_time;
        self.mp3_end = Some((handle.1, Some(end_time)));
        if self.do_output {
            let _ = self.mixer_handle.add(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            );
        }
    }

    pub fn play_wav(&mut self, handle: &WavHandle, start_time: u128) {
        let end_time = handle.0.length() as u128 + start_time;
        self.end_times.insert(handle.1, Some(end_time));
        if self.do_output {
            let _ = self.mixer_handle.add(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            );
        }
    }

    pub fn loop_mp3(&mut self, handle: &Mp3Handle) {
        self.mp3_end = Some((handle.1, None));
        if self.do_output {
            let _ = self.mixer_handle.add(Cycle::new(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            ));
        }
    }

    pub fn loop_wav(&mut self, handle: &WavHandle) {
        self.end_times.insert(handle.1, None);
        if self.do_output {
            let _ = self.mixer_handle.add(Cycle::new(
                Rechanneler::new(Resampler::new(handle.0.clone(), self.mixer_sample_rate), self.mixer_channel_count)
            ));
        }
    }

    pub fn sound_playing(&self, sound_id: i32, current_time: u128) -> bool {
        self.mp3_playing(sound_id, current_time) || self.wav_playing(sound_id, current_time)
    }

    fn mp3_playing(&self, sound_id: i32, current_time: u128) -> bool {
        self.mp3_end.map(|(id, end_time)| id == sound_id && end_time.map(|x| x > current_time).unwrap_or(true)).unwrap_or(false)
    }

    fn wav_playing(&self, sound_id: i32, current_time: u128) -> bool {
        match self.end_times.get(&sound_id) {
            Some(&Some(x)) => x > current_time,
            Some(None) => true,
            None => false,
        }
    }

    // sound_stop should delete an entry from the map if it's a wav, mp3_end to None if it's an mp3
    // sound_stop_all should clear the map and set mp3_end to None
}
