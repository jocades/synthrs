use crate::{env, osc::OscKind};

/// An instrument is just a preset / blueprint for a runtime [Voice].
pub struct Instrument {
    pub shape: env::Shape,
    pub oscs: Vec<OscKind>,
}

impl Instrument {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Default)]
pub struct Builder {
    shape: env::Shape,
    oscs: Vec<OscKind>,
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

    pub fn osc(mut self, kind: OscKind) -> Self {
        self.oscs.push(kind);
        self
    }

    pub fn build(self) -> Instrument {
        Instrument {
            shape: self.shape,
            oscs: self.oscs,
        }
    }
}
