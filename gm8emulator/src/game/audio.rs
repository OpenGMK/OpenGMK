mod mixer;
mod mp3;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use udon::{
    cycle::Cycle,
    rechanneler::Rechanneler,
    resampler::Resampler,
    session::{Api, Session},
    source::{ChannelCount, SampleRate, Source},
    wav::WavPlayer,
};

use self::{
    mixer::{Mixer, MixerHandle},
    mp3::Mp3Player,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Mp3Handle {
    player: Mp3Player,
    id: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WavHandle {
    player: WavPlayer,
    params: Arc<SoundParams>,
    _use_3d: bool,
    exclusive: bool,
    id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct SoundParams {
    pub volume: AtomicU32,
}

pub struct AudioManager {
    mixer_handle: MixerHandle,
    mixer_channel_count: ChannelCount,
    mixer_sample_rate: SampleRate,
    do_output: bool,
    global_volume: Arc<AtomicU32>,
    end_times: HashMap<i32, Option<u128>>,
    multimedia_end: Option<(i32, Option<u128>)>,
}

impl AudioManager {
    pub fn new(do_output: bool) -> Self {
        // TODO: not all these unwraps
        let session = Session::new(Api::SoundIo).unwrap();
        let device = session.default_output_device().unwrap();
        let sample_rate = device.sample_rate();
        let channel_count = device.channel_count();
        let global_volume = Arc::new(AtomicU32::from(1.0f32.to_bits()));
        let (mixer, mixer_handle) = Mixer::new(sample_rate, channel_count, global_volume.clone());

        std::thread::spawn(move || {
            let stream = session.open_output_stream(device).unwrap();
            stream.play(mixer).unwrap();
        });

        Self {
            mixer_handle,
            mixer_channel_count: channel_count,
            mixer_sample_rate: sample_rate,
            do_output,
            global_volume,
            end_times: HashMap::new(),
            multimedia_end: None,
        }
    }

    pub fn add_mp3(&mut self, file: Box<[u8]>, sound_id: i32) -> Option<Mp3Handle> {
        Mp3Player::new(file).map(|player| Mp3Handle { player, id: sound_id }).ok()
    }

    pub fn add_wav(
        &mut self,
        file: Box<[u8]>,
        sound_id: i32,
        volume: f64,
        use_3d: bool,
        exclusive: bool,
    ) -> Option<WavHandle> {
        WavPlayer::new(file)
            .map(|player| WavHandle {
                player,
                params: Arc::new(SoundParams { volume: AtomicU32::new(make_volume(volume).to_bits()) }),
                _use_3d: use_3d,
                exclusive,
                id: sound_id,
            })
            .ok()
    }

    pub fn play_mp3(&mut self, handle: &Mp3Handle, start_time: u128) {
        let end_time = length_to_ns(
            handle.player.length(),
            handle.player.sample_rate().into(),
            1, // mp3 length() already takes channels into account
        ) + start_time;
        self.multimedia_end = Some((handle.id, Some(end_time)));
        if self.do_output {
            let _ = self.mixer_handle.add_exclusive(
                Rechanneler::new(
                    Resampler::new(handle.player.clone(), self.mixer_sample_rate),
                    self.mixer_channel_count,
                ),
                handle.id,
            );
        }
    }

    pub fn play_wav(&mut self, handle: &WavHandle, start_time: u128) {
        let end_time = length_to_ns(
            handle.player.length(),
            handle.player.sample_rate().into(),
            handle.player.channel_count().into(),
        ) + start_time;
        if handle.exclusive {
            self.multimedia_end = Some((handle.id, Some(end_time)));
        } else if self.end_times.get(&handle.id) != Some(&None) {
            self.end_times.insert(handle.id, Some(end_time));
        }

        if self.do_output {
            if handle.exclusive {
                let _ = self.mixer_handle.add_exclusive(
                    Rechanneler::new(
                        Resampler::new(handle.player.clone(), self.mixer_sample_rate),
                        self.mixer_channel_count,
                    ),
                    handle.id,
                );
            } else {
                let _ = self.mixer_handle.add(
                    Rechanneler::new(
                        Resampler::new(handle.player.clone(), self.mixer_sample_rate),
                        self.mixer_channel_count,
                    ),
                    handle.params.clone(),
                    handle.id,
                );
            }
        }
    }

    pub fn loop_mp3(&mut self, handle: &Mp3Handle) {
        self.multimedia_end = Some((handle.id, None));
        if self.do_output {
            let _ = self.mixer_handle.add_exclusive(
                Cycle::new(Rechanneler::new(
                    Resampler::new(handle.player.clone(), self.mixer_sample_rate),
                    self.mixer_channel_count,
                )),
                handle.id,
            );
        }
    }

    pub fn loop_wav(&mut self, handle: &WavHandle) {
        if handle.exclusive {
            self.multimedia_end = Some((handle.id, None));
        } else {
            self.end_times.insert(handle.id, None);
        }

        if self.do_output {
            if handle.exclusive {
                let _ = self.mixer_handle.add_exclusive(
                    Cycle::new(Rechanneler::new(
                        Resampler::new(handle.player.clone(), self.mixer_sample_rate),
                        self.mixer_channel_count,
                    )),
                    handle.id,
                );
            } else {
                let _ = self.mixer_handle.add(
                    Cycle::new(Rechanneler::new(
                        Resampler::new(handle.player.clone(), self.mixer_sample_rate),
                        self.mixer_channel_count,
                    )),
                    handle.params.clone(),
                    handle.id,
                );
            }
        }
    }

    pub fn stop_sound(&mut self, id: i32) {
        self.end_times.remove(&id);
        if self.multimedia_end.map(|(x, _)| x) == Some(id) {
            self.multimedia_end = None;
        }
        if self.do_output {
            let _ = self.mixer_handle.stop(id);
        }
    }

    pub fn stop_all(&mut self) {
        self.end_times.clear();
        self.multimedia_end = None;
        if self.do_output {
            let _ = self.mixer_handle.stop_all();
        }
    }

    pub fn set_global_volume(&self, vol: f64) {
        self.global_volume.store(make_volume(vol).to_bits(), Ordering::Release)
    }

    pub fn sound_playing(&self, sound_id: i32, current_time: u128) -> bool {
        self.mp3_playing(sound_id, current_time) || self.wav_playing(sound_id, current_time)
    }

    fn mp3_playing(&self, sound_id: i32, current_time: u128) -> bool {
        self.multimedia_end
            .map(|(id, end_time)| id == sound_id && end_time.map(|x| x > current_time).unwrap_or(true))
            .unwrap_or(false)
    }

    fn wav_playing(&self, sound_id: i32, current_time: u128) -> bool {
        match self.end_times.get(&sound_id) {
            Some(&Some(x)) => x > current_time,
            Some(None) => true,
            None => false,
        }
    }

    pub fn state(&self) -> AudioState {
        AudioState {
            global_volume: self.global_volume.clone(),
            end_times: self.end_times.clone(),
            multimedia_end: self.multimedia_end,
        }
    }

    pub fn set_state(&mut self, state: AudioState) {
        self.global_volume = state.global_volume;
        self.end_times = state.end_times;
        self.multimedia_end = state.multimedia_end;
    }
}

impl WavHandle {
    pub fn set_volume(&self, vol: f64) {
        self.params.volume.store(make_volume(vol).to_bits(), Ordering::Release);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioState {
    global_volume: Arc<AtomicU32>,
    end_times: HashMap<i32, Option<u128>>,
    multimedia_end: Option<(i32, Option<u128>)>,
}

fn length_to_ns(sample_count: usize, sample_rate: u32, channels: u16) -> u128 {
    (sample_count as u128 * 1_000_000_000) / (u128::from(sample_rate) * u128::from(channels))
}

// This function takes a volume between 0.0 and 1.0 and converts it to the logarithmic scale used by DirectMusic.
// This is, roughly, the same function used by GM8/DirectMusic.
// Note that the minimum possible output from this function is 0.001. I think that's accurate to GM8.
fn make_volume(vol: f64) -> f32 {
    1000.0f64.powf(vol.clamp(0.0, 1.0) - 1.0) as f32
}
