use crate::{Hz, env, osc::OscKind};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Kind {
    #[default]
    Pitched,
    Percussive(Hz),
}

/// An instrument is just a preset for a runtime [Voice].
pub struct Instrument {
    pub kind: Kind,
    /// The [crate::env::Shape] of an ADSL [crate::env::Env].
    pub shape: env::Shape,
    /// [OscKind]s and gains to construct an [crate::osc::Osc].
    pub oscs: Vec<(OscKind, f64)>,
    pub lfos: Vec<(f64, f64)>, // freq, depth
}

impl Instrument {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Default)]
pub struct Builder {
    kind: Kind,
    shape: env::Shape,
    oscs: Vec<(OscKind, f64)>,
    lfos: Vec<(f64, f64)>,
}

impl Builder {
    pub fn pitched(mut self) -> Self {
        self.kind = Kind::Pitched;
        self
    }

    pub fn percussive(mut self, freq: impl Into<Hz>) -> Self {
        self.kind = Kind::Percussive(freq.into());
        self.oneshot()
    }

    pub fn env(mut self, a: f64, d: f64, s: f64, r: f64) -> Self {
        self.shape = env::Shape {
            attack: a,
            decay: d,
            sustain: s,
            release: r,
            hold: true,
        };
        self
    }

    pub fn oneshot(mut self) -> Self {
        self.shape.hold = false;
        self
    }

    pub fn osc(mut self, kind: OscKind, gain: f64) -> Self {
        self.oscs.push((kind, gain));
        self
    }

    pub fn lfo(mut self, freq: f64, depth: f64) -> Self {
        self.lfos.push((freq, depth));
        self
    }

    pub fn build(self) -> Instrument {
        Instrument {
            kind: self.kind,
            shape: self.shape,
            oscs: self.oscs,
            lfos: self.lfos,
        }
    }
}

// freq: 60.0
pub fn kick() -> Instrument {
    Instrument::builder()
        .percussive(60.0)
        .osc(OscKind::Sine, 1.0)
        .env(0.001, 0.15, 0.0, 0.0)
        .build()
}

// freq: 180.0
pub fn snare() -> Instrument {
    Instrument::builder()
        .percussive(180.0)
        .osc(OscKind::Noise, 0.2)
        .osc(OscKind::Sine, 0.5)
        .env(0.001, 0.12, 0.0, 0.0)
        .build()
}

pub fn hihat() -> Instrument {
    Instrument::builder()
        .percussive(0.0)
        .osc(OscKind::Noise, 0.4)
        .env(0.001, 0.03, 0.0, 0.0)
        .build()
}
