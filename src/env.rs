#[derive(Default, PartialEq, Eq)]
enum State {
    #[default]
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

#[derive(Default)]
pub struct Env {
    attack: f64,  // secs
    decay: f64,   // secs
    sustain: f64, // amp
    release: f64, // secs

    pub amp: f64, // 0..1
    state: State,
}

impl Env {
    pub fn new(a: f64, d: f64, s: f64, r: f64) -> Self {
        Self {
            attack: a,
            decay: d,
            sustain: s,
            release: r,
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
                if self.attack > 0.0 {
                    self.amp += dt / self.attack;
                } else {
                    self.amp = 1.0;
                }

                if self.amp >= 1.0 {
                    self.amp = 1.0;
                    self.state = State::Decay;
                }
            }

            State::Decay => {
                if self.decay > 0.0 {
                    self.amp -= dt * (1.0 - self.sustain) / self.decay;
                }

                if self.amp <= self.sustain {
                    self.amp = self.sustain;
                    self.state = State::Sustain;
                }
            }

            State::Sustain => {}

            State::Release => {
                if self.release > 0.0 {
                    self.amp *= (-dt / self.release).exp();
                }

                if self.amp <= 0.0001 {
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
