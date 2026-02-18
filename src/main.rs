use std::sync::mpsc;
use std::time::Duration;
use std::{cmp, thread};

use synth::env::Env;
use synth::kbd::{self, KeyCode, Keyboard};
use synth::osc::{Osc, OscKind};
use synth::preset::Instrument;
use synth::{Engine, Hz};

#[derive(Default)]
struct Voice {
    /// Invariant: voice.alive -> key.is_some()
    keycode: Option<KeyCode>,
    oscs: Vec<Osc>,
    env: Env,
}

enum Event {
    NoteOn(KeyCode, Hz),
    NoteOff(KeyCode),
}

struct Synth<const N: usize = 32> {
    voices: [Voice; N],
    instrument: Instrument,
    rx: mpsc::Receiver<Event>,
}

impl<const N: usize> Synth<N> {
    fn new(rx: mpsc::Receiver<Event>, instrument: Instrument) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::default()),
            instrument,
            rx,
        }
    }

    fn note_on(&mut self, code: KeyCode, freq: Hz) {
        let voice = if let Some(v) = self.voices.iter_mut().find(|v| v.keycode.is_none()) {
            v
        } else {
            self.voices
                .iter_mut()
                .min_by(|a, b| {
                    a.env
                        .amp
                        .partial_cmp(&b.env.amp)
                        .unwrap_or(cmp::Ordering::Equal)
                })
                .unwrap()
        };

        voice.keycode = Some(code);

        voice.oscs.clear();
        for &(kind, gain) in &self.instrument.oscs {
            voice.oscs.push(Osc::new(kind, freq, SAMPLE_RATE, gain));
        }

        voice.env = Env::new(self.instrument.shape);
    }

    fn note_off(&mut self, code: KeyCode) {
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

            for voice in self.voices.iter_mut().filter(|v| v.keycode.is_some()) {
                let amp = voice.env.next(dt);

                if voice.env.is_finished() {
                    voice.keycode = None;
                    continue;
                }

                let sum = voice.oscs.iter_mut().map(|osc| osc.next()).sum::<f64>();
                let norm = sum / voice.oscs.len() as f64;

                mix += amp * norm;
            }

            // master gain
            mix *= 0.25;

            *sample = mix as f32;
        }
    }
}

const SAMPLE_RATE: f64 = 44_100.0;

fn main() {
    let (tx, rx) = mpsc::channel();

    let instrument = Instrument::builder()
        .osc(OscKind::Sine, 1.0)
        .osc(OscKind::Triangle, 0.25)
        .osc(OscKind::Saw, 0.1)
        .env(0.005, 0.1, 0.8, 0.2)
        .build();

    let mut synth = Synth::<32>::new(rx, instrument);
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

        if kbd::is_key_down(KeyCode::Q) {
            break;
        }

        thread::sleep(Duration::from_millis(2));
    }

    engine.stop();
}
