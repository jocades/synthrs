mod engine;
mod kbd;

use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use engine::Engine;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Hz(f64);

impl Hz {
    /// Convert frequency (Hz) to angular velocity (ω).
    #[inline(always)]
    fn w(&self) -> f64 {
        self.0 * 2.0 * PI
    }

    fn from_pitch_std(semitones: i32) -> Self {
        const PITCH_STANDARD: f64 = 440.0;
        const TWELFTH_ROOT_OF_TWO: f64 = 1.0594630943592952646;

        Hz(PITCH_STANDARD * TWELFTH_ROOT_OF_TWO.powi(semitones))
    }
}

#[derive(Default, Clone, Copy)]
struct Note {
    freq: Hz,
    phase: f64,
    pressed: bool,
    alive: bool,
}

impl Note {
    fn from_pitch_std(semitones: i32) -> Self {
        Self {
            freq: Hz::from_pitch_std(semitones),
            ..Default::default()
        }
    }
}

const SAMPLE_RATE: f64 = 44_100.0;

fn main() {
    let piano = [(kbd::Key::A, Note::from_pitch_std(-9))];
    let piano = Arc::new(Mutex::new(piano));

    let engine = {
        let piano = piano.clone();

        Engine::new(SAMPLE_RATE, move |buf| {
            let mut piano = piano.lock().unwrap();

            for sample in buf {
                let mut mix = 0.0f32;

                for (_, note) in piano.iter_mut().filter(|(_, n)| n.alive) {
                    mix += (0.5 * note.phase.sin()) as f32;
                    note.phase += note.freq.w() / SAMPLE_RATE;
                    if note.phase > 2.0 * PI {
                        note.phase -= 2.0 * PI;
                    }
                }

                *sample = mix;
            }
        })
    };

    engine.start();

    loop {
        let mut piano = piano.lock().unwrap();
        for (key, note) in piano.iter_mut() {
            let pressed = kbd::is_key_down(*key);

            if pressed && !note.pressed {
                note.pressed = true;
                note.alive = true;
            }

            if !pressed && note.pressed {
                note.pressed = false;
                note.alive = false;
            }
        }
        drop(piano);

        if kbd::is_key_down(kbd::Key::Q) {
            break;
        }

        thread::sleep(Duration::from_millis(1));
    }

    engine.stop();
}
