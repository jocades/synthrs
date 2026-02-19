#[derive(Default, PartialEq, Eq)]
enum State {
    #[default]
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

/// The Shape / Curve of an ADSR Envelope.
#[derive(Clone, Copy)]
pub struct Shape {
    pub attack: f64,  // secs
    pub decay: f64,   // secs
    pub sustain: f64, // amp
    pub release: f64, // secs
    pub hold: bool,   // wether the sound should sustain
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.02,
            sustain: 0.8,
            release: 0.2,
            hold: true,
        }
    }
}

#[derive(Default)]
pub struct Env {
    shape: Shape,
    state: State,
    pub amp: f64, // 0..1
}

impl Env {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            amp: 0.0,
            state: State::Attack,
        }
    }

    pub fn note_off(&mut self) {
        if self.state != State::Finished {
            self.state = State::Release;
        }
    }

    pub fn next(&mut self, dt: f64) -> f64 {
        match self.state {
            State::Attack => {
                if self.shape.attack > 0.0 {
                    self.amp += dt / self.shape.attack;
                } else {
                    self.amp = 1.0;
                }

                if self.amp >= 1.0 {
                    self.amp = 1.0;
                    self.state = State::Decay;
                }
            }

            State::Decay => {
                if self.shape.decay > 0.0 {
                    self.amp -= dt * (1.0 - self.shape.sustain) / self.shape.decay;
                }

                if self.amp <= self.shape.sustain {
                    self.amp = self.shape.sustain;
                    self.state = if self.shape.hold {
                        State::Sustain
                    } else {
                        State::Release
                    };
                }
            }

            State::Sustain => {}

            State::Release => {
                if self.shape.release > 0.0 {
                    self.amp *= (-dt / self.shape.release).exp();
                }

                if self.amp <= 0.001 {
                    self.amp = 0.0;
                    self.state = State::Finished;
                }
            }

            State::Finished => {}
        }

        self.amp
    }

    pub fn is_finished(&self) -> bool {
        self.state == State::Finished
    }
}
