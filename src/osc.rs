use crate::{Hz, consts::TAU};

#[derive(Default, Clone, Copy)]
pub enum OscKind {
    #[default]
    Sine,
    Square,
    Triangle,
    Saw,
}

#[derive(Default)]
pub struct Osc {
    kind: OscKind,
    phase: f64, // 0..1
    increment: f64,
    gain: f64, // 0..1
}

impl Osc {
    pub fn new(kind: OscKind, freq: Hz, sr: f64, gain: f64) -> Self {
        Self {
            phase: 0.0,
            increment: freq.0 / sr,
            kind,
            gain,
        }
    }

    pub fn next(&mut self) -> f64 {
        let p = self.phase;
        let out = match self.kind {
            OscKind::Sine => (p * TAU).sin(),
            OscKind::Square => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            OscKind::Triangle => 1.0 - 4.0 * (p - 0.5).abs(),
            OscKind::Saw => 2.0 * p - 1.0,
        };

        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0
        }

        out * self.gain
    }
}
