use crate::{Hz, consts::TAU};

#[derive(Default)]
pub enum Kind {
    #[default]
    Sine,
    Square,
    Triangle,
    Saw,
}

#[derive(Default)]
pub struct Osc {
    phase: f64, // 0..1
    increment: f64,
    kind: Kind,
}

impl Osc {
    pub fn new(freq: Hz, sr: f64, kind: Kind) -> Self {
        Self {
            phase: 0.0,
            increment: freq.0 / sr,
            kind,
        }
    }

    pub fn next(&mut self) -> f64 {
        let p = self.phase;
        let out = match self.kind {
            Kind::Sine => (p * TAU).sin(),
            Kind::Square => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Kind::Triangle => 1.0 - 4.0 * (p - 0.5).abs(),
            Kind::Saw => 2.0 * p - 1.0,
        };

        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0
        }

        out
    }
}
