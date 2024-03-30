use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired, AudioStatus};
use sdl2::Sdl;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;
    
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Beep {
    device: AudioDevice<SquareWave>,
}

impl Beep {
    pub fn new(sdl_context: &Sdl) -> Self {
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                SquareWave {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.2,
                }
            })
            .unwrap();

        Beep { device }
    }

    pub fn play(&self) {
        if self.device.status() == AudioStatus::Stopped
            || self.device.status() == AudioStatus::Paused
        {
            self.device.resume();
        }
    }

    pub fn pause(&self) {
        self.device.pause();
    }
}

