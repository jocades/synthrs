use crate::{env, osc::OscKind};

/// An instrument is just a preset for a runtime [Voice].
pub struct Instrument {
    /// The [crate::env::Shape] of an ADSL [crate::env::Env].
    pub shape: env::Shape,
    /// [OscKind]s and gains to construct an [crate::osc::Osc].
    pub oscs: Vec<(OscKind, f64)>,
    pub lfo: (f64, f64), // freq, depth
}

impl Instrument {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Default)]
pub struct Builder {
    shape: env::Shape,
    oscs: Vec<(OscKind, f64)>,
    lfo: (f64, f64),
}

impl Builder {
    pub fn env(mut self, a: f64, d: f64, s: f64, r: f64) -> Self {
        self.shape = env::Shape {
            attack: a,
            decay: d,
            sustain: s,
            release: r,
        };
        self
    }

    pub fn osc(mut self, kind: OscKind, gain: f64) -> Self {
        self.oscs.push((kind, gain));
        self
    }

    pub fn lfo(mut self, freq: f64, depth: f64) -> Self {
        self.lfo = (freq, depth);
        self
    }

    pub fn build(self) -> Instrument {
        Instrument {
            shape: self.shape,
            oscs: self.oscs,
            lfo: self.lfo,
        }
    }
}
