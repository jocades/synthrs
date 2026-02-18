use std::f64::consts::PI;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use synth::kbd::{self, Keyboard};
use synth::{Engine, Hz};

#[derive(Default, PartialEq, Eq)]
enum EnvelopeState {
    #[default]
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

#[derive(Default)]
struct Envelope {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    level: f64,
    state: EnvelopeState,
}

impl Envelope {
    fn new(a: f64, d: f64, s: f64, r: f64) -> Self {
        Self {
            attack: a,
            decay: d,
            sustain: s,
            release: r,
            level: 0.0,
            state: EnvelopeState::Attack,
        }
    }

    fn note_off(&mut self) {
        if self.state != EnvelopeState::Finished {
            self.state = EnvelopeState::Release;
        }
    }

    fn next(&mut self, dt: f64) -> f64 {
        match self.state {
            EnvelopeState::Attack => {
                if self.attack > 0.0 {
                    self.level += dt / self.attack;
                } else {
                    self.level = 1.0;
                }

                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }

            EnvelopeState::Decay => {
                if self.decay > 0.0 {
                    self.level -= dt * (1.0 - self.sustain) / self.decay;
                }

                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.state = EnvelopeState::Sustain;
                }
            }

            EnvelopeState::Sustain => {}

            EnvelopeState::Release => {
                if self.release > 0.0 {
                    self.level *= (-dt / self.release).exp();
                }

                if self.level <= 0.0001 {
                    self.level = 0.0;
                    self.state = EnvelopeState::Finished;
                }
            }

            EnvelopeState::Finished => {}
        }

        self.level
    }

    fn is_finished(&self) -> bool {
        self.state == EnvelopeState::Finished
    }
}

#[derive(Default)]
struct Voice {
    /// Invariant: voice.alive -> key.is_some()
    keycode: Option<kbd::KeyCode>,
    freq: Hz,
    phase: f64,
    env: Envelope,
}

enum Event {
    NoteOn(kbd::KeyCode, Hz),
    NoteOff(kbd::KeyCode),
}

struct Synth<const N: usize = 32> {
    voices: [Voice; N],
    rx: mpsc::Receiver<Event>,
}

impl<const N: usize> Synth<N> {
    fn new(rx: mpsc::Receiver<Event>) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::default()),
            rx,
        }
    }

    fn note_on(&mut self, code: kbd::KeyCode, freq: Hz) {
        let voice = if let Some(v) = self.voices.iter_mut().find(|v| v.keycode.is_none()) {
            v
        } else {
            self.voices
                .iter_mut()
                .min_by(|a, b| a.env.level.partial_cmp(&b.env.level).unwrap())
                .unwrap()
        };

        voice.keycode = Some(code);
        voice.freq = freq;
        voice.phase = 0.0;
        voice.env = Envelope::new(0.1, 0.01, 0.8, 0.2);
    }

    fn note_off(&mut self, code: kbd::KeyCode) {
        for v in self.voices.iter_mut().filter(|v| v.keycode == Some(code)) {
            v.env.note_off();
        }
    }

    fn process(&mut self, buf: &mut [f32]) {
        let dt = 1.0 / SAMPLE_RATE;

        for _ in 0..128 {
            let Ok(event) = self.rx.try_recv() else {
                break;
            };
            match event {
                Event::NoteOn(keycode, freq) => self.note_on(keycode, freq),
                Event::NoteOff(keycode) => self.note_off(keycode),
            }
        }

        for sample in buf {
            let mut mix = 0.0;

            for v in self.voices.iter_mut().filter(|v| v.keycode.is_some()) {
                let amp = v.env.next(dt);

                if v.env.is_finished() {
                    v.keycode = None;
                    continue;
                }

                mix += amp * v.phase.sin();

                v.phase += v.freq.w() * dt;
                if v.phase > 2.0 * PI {
                    v.phase -= 2.0 * PI;
                }
            }

            *sample = (mix * 0.5) as f32;
        }
    }
}

const SAMPLE_RATE: f64 = 44_100.0;

fn main() {
    let (tx, rx) = mpsc::channel();

    let mut synth = Synth::<32>::new(rx);
    let engine = Engine::new(SAMPLE_RATE, move |buf| synth.process(buf));

    engine.start();

    let mut keyboard = Keyboard::new();

    loop {
        for key in keyboard.keys.iter_mut() {
            let down = kbd::is_key_down(key.code);

            if down && !key.pressed {
                key.pressed = true;
                _ = tx.send(Event::NoteOn(key.code, key.freq));
            }

            if !down && key.pressed {
                key.pressed = false;
                _ = tx.send(Event::NoteOff(key.code));
            }
        }

        if kbd::is_key_down(kbd::KeyCode::Q) {
            break;
        }

        thread::sleep(Duration::from_millis(2));
    }

    engine.stop();
}
