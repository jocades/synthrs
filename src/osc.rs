use crate::{Hz, consts::TAU};

#[derive(Default, Clone, Copy)]
pub enum Waveform {
    #[default]
    Sine,
    Square,
    Triangle,
    Saw,
    Noise,
}

#[derive(Default)]
pub struct Osc {
    waveform: Waveform,
    phase: f64, // 0..1
    base_increment: f64,
    increment: f64,
    gain: f64, // 0..1
}

impl Osc {
    pub fn new(waveform: Waveform, freq: Hz, sr: f64, gain: f64) -> Self {
        let inc = freq.0 / sr;
        Self {
            waveform,
            phase: 0.0,
            increment: inc,
            base_increment: inc,
            gain,
        }
    }

    /// lfo value expected in range [-1, 1] scaled by gain
    pub fn mod_freq(&mut self, lfo: f64) {
        self.increment = self.base_increment * (1.0 + lfo);
    }

    pub fn next(&mut self) -> f64 {
        let out = match self.waveform {
            Waveform::Sine => (self.phase * TAU).sin(),
            Waveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Triangle => 1.0 - 4.0 * (self.phase - 0.5).abs(),
            Waveform::Saw => 2.0 * self.phase - 1.0,
            Waveform::Noise => rand::random_range(-1.0..1.0),
        };

        self.phase = (self.phase + self.increment) % 1.0;

        out * self.gain
    }
}

#[derive(Default)]
pub struct Lfo {
    phase: f64,
    increment: f64,
    depth: f64, // gain
}

impl Lfo {
    pub fn new(freq: f64, sr: f64, depth: f64) -> Self {
        Self {
            phase: 0.0,
            increment: freq / sr,
            depth,
        }
    }

    pub fn next(&mut self) -> f64 {
        let out = (self.phase * TAU).sin();

        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0
        }

        out * self.depth
    }
}
