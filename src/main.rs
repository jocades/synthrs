use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{cmp, thread};

use synth::env::Env;
use synth::kbd::{self, KeyCode, Keyboard};
use synth::osc::{Lfo, Osc, OscKind};
use synth::preset::{self, Instrument};
use synth::{Engine, Hz};

#[derive(Default)]
struct Voice {
    inst_id: usize,
    active: bool,
    keycode: Option<KeyCode>,
    freq: Hz,
    env: Env,
    lfos: Vec<Lfo>,
    oscs: Vec<Osc>,
}

enum Event {
    NoteOn(usize, KeyCode, Hz),
    NoteOff(usize, KeyCode),
    Trigger(usize),
}

struct Synth<const N: usize = 32> {
    voices: [Voice; N],
    instruments: Vec<Instrument>,
    rx: mpsc::Receiver<Event>,
}

impl<const N: usize> Synth<N> {
    fn new(rx: mpsc::Receiver<Event>, instruments: Vec<Instrument>) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::default()),
            instruments,
            rx,
        }
    }

    fn find_voice_slot(&self) -> usize {
        // Try to find a free voice first
        if let Some(idx) = self.voices.iter().position(|v| !v.active) {
            return idx;
        }

        // If none free, steal the one with lowest amplitude
        self.voices
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.env
                    .amp
                    .partial_cmp(&b.env.amp)
                    .unwrap_or(cmp::Ordering::Equal)
            })
            .unwrap()
            .0
    }

    fn note_on(&mut self, inst: usize, code: KeyCode, freq: Hz) {
        let index = self.find_voice_slot();
        let voice = &mut self.voices[index];

        voice.inst_id = inst;
        voice.active = true;
        voice.keycode = Some(code);

        let instrument = &self.instruments[inst];

        voice.freq = match instrument.kind {
            preset::Kind::Pitched => freq,
            preset::Kind::Percussive(freq) => freq,
        };

        voice.env = Env::new(instrument.shape);

        voice.lfos.clear();
        for &(rate, depth) in &instrument.lfos {
            voice.lfos.push(Lfo::new(rate, SAMPLE_RATE, depth));
        }

        voice.oscs.clear();
        for &(kind, gain) in &instrument.oscs {
            voice.oscs.push(Osc::new(kind, freq, SAMPLE_RATE, gain));
        }
    }

    fn note_off(&mut self, inst: usize, code: KeyCode) {
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.active && v.inst_id == inst && v.keycode == Some(code))
        {
            v.env.note_off();
        }
    }

    fn trigger(&mut self, inst: usize) {
        let index = self.find_voice_slot();
        let voice = &mut self.voices[index];

        voice.inst_id = inst;
        voice.active = true;

        let instrument = &self.instruments[inst];

        // assume only drums are trigger for now
        voice.freq = match instrument.kind {
            preset::Kind::Pitched => panic!("can only trigger percussive instruments"),
            preset::Kind::Percussive(freq) => freq,
        };

        voice.env = Env::new(instrument.shape);

        voice.lfos.clear();
        for &(rate, depth) in &instrument.lfos {
            voice.lfos.push(Lfo::new(rate, SAMPLE_RATE, depth));
        }

        voice.oscs.clear();
        for &(kind, gain) in &instrument.oscs {
            voice
                .oscs
                .push(Osc::new(kind, voice.freq, SAMPLE_RATE, gain));
        }
    }

    fn process(&mut self, buf: &mut [f32]) {
        let dt = 1.0 / SAMPLE_RATE;

        for _ in 0..128 {
            let Ok(event) = self.rx.try_recv() else {
                break;
            };
            match event {
                Event::NoteOn(inst, keycode, freq) => self.note_on(inst, keycode, freq),
                Event::NoteOff(inst, keycode) => self.note_off(inst, keycode),
                Event::Trigger(inst) => self.trigger(inst),
            }
        }

        for sample in buf {
            let mut mix = 0.0;

            for voice in self.voices.iter_mut().filter(|v| v.active) {
                let amp = voice.env.next(dt);

                if voice.env.is_finished() {
                    voice.active = false;
                    continue;
                }

                let scale = 1.0 + voice.lfos.iter_mut().map(|lfo| lfo.next()).sum::<f64>();

                let sum = voice
                    .oscs
                    .iter_mut()
                    .map(|osc| osc.next(scale))
                    .sum::<f64>();

                // mix += amp * (sum / voice.oscs.len() as f64);
                mix += amp * sum;
            }

            // master gain
            *sample = (0.2 * mix) as f32;
        }
    }
}

struct Sequencer {
    bpm: f64,
}

impl Sequencer {
    fn new(bpm: f64) -> Self {
        Self { bpm }
    }

    fn update(&mut self) {}
}

const SAMPLE_RATE: f64 = 44_100.0;

fn main() {
    let (tx, rx) = mpsc::channel();

    let instrument = Instrument::builder()
        .lfo(3.0, 0.02)
        .osc(OscKind::Sine, 1.0)
        .osc(OscKind::Saw, 0.2)
        .env(0.005, 0.1, 0.8, 0.2)
        .build();

    let instruments = vec![instrument, preset::kick(), preset::snare(), preset::hihat()];

    let mut synth = Synth::<32>::new(rx, instruments);
    let engine = Engine::new(SAMPLE_RATE, move |buf| synth.process(buf));

    engine.start();

    let mut keyboard = Keyboard::new();

    // 4x4
    let bpm = 100.0;
    let beat_duration = Duration::from_secs_f64(60.0 / bpm / 4.0);

    let mut last_beat = Instant::now();
    let mut current_beat = 0usize;

    let pat1 = "x...x...x...x...".as_bytes();
    let pat2 = ".xx...xx.xx...xx".as_bytes();

    let drums = [
        kbd::Key {
            code: KeyCode::Z,
            freq: Hz(60.0),
            pressed: false,
        },
        kbd::Key {
            code: KeyCode::X,
            freq: Hz(180.0),
            pressed: false,
        },
        kbd::Key {
            code: KeyCode::C,
            freq: Hz(100.0),
            pressed: false,
        },
    ];

    loop {
        if Instant::now().duration_since(last_beat) >= beat_duration {
            if pat1[current_beat] == b'x' {
                _ = tx.send(Event::Trigger(2))
            }
            last_beat += beat_duration;
            current_beat = (current_beat + 1) % 16;
        }

        for key in keyboard.keys.iter_mut() {
            let down = kbd::is_key_down(key.code);

            if down && !key.pressed {
                key.pressed = true;
                _ = tx.send(Event::NoteOn(0, key.code, key.freq));
            }

            if !down && key.pressed {
                key.pressed = false;
                _ = tx.send(Event::NoteOff(0, key.code));
            }
        }

        if kbd::is_key_down(KeyCode::Q) {
            break;
        }

        thread::sleep(Duration::from_millis(2));
    }

    engine.stop();
}
