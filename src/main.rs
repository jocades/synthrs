use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{cmp, thread};

use synth::env::Env;
use synth::kbd::{self, KeyCode, Keyboard};
use synth::osc::{Osc, Waveform};
use synth::preset::{self, Instrument};
use synth::{Engine, Hz};

#[derive(Default)]
struct Voice {
    inst_id: usize,
    /// Wether this voice is _currently_ producing sound
    active: bool,
    /// Midi note 0..128
    note: u8,
    freq: Hz,
    env: Env,
    lfos: Vec<Osc>,
    oscs: Vec<Osc>,
}

enum Event {
    NoteOn(usize, u8),
    NoteOff(usize, u8),
    Trigger(usize),
}

thread_local! {
    static FREQ_MAP: [Hz; 18] = [
        Hz::from_pitch_std(-9),
        Hz::from_pitch_std(-8),
        Hz::from_pitch_std(-7),
        Hz::from_pitch_std(-6),
        Hz::from_pitch_std(-5),
        Hz::from_pitch_std(-4),
        Hz::from_pitch_std(-3),
        Hz::from_pitch_std(-2),
        Hz::from_pitch_std(-1),
        Hz::from_pitch_std(0),
        Hz::from_pitch_std(1),
        Hz::from_pitch_std(2),
        Hz::from_pitch_std(3),
        Hz::from_pitch_std(4),
        Hz::from_pitch_std(5),
        Hz::from_pitch_std(6),
        Hz::from_pitch_std(7),
        Hz::from_pitch_std(8),
    ]
}

struct Synth<const N: usize = 64> {
    voices: [Voice; N],
    instruments: Vec<Instrument>,
    rx: mpsc::Receiver<Event>,
}

impl<const N: usize> Synth<N> {
    fn new(rx: mpsc::Receiver<Event>, instruments: Vec<Instrument>) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice {
                oscs: Vec::with_capacity(5),
                lfos: Vec::with_capacity(5),
                ..Default::default()
            }),
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

    fn init_voice(&mut self, inst: usize, note: Option<u8>) {
        let index = self.find_voice_slot();
        let voice = &mut self.voices[index];

        voice.inst_id = inst;
        voice.active = true;

        let instrument = &self.instruments[inst];

        match instrument.kind {
            preset::Kind::Pitched => {
                voice.note = note.unwrap();
                voice.freq = FREQ_MAP.with(|m| m[voice.note as usize]);
            }
            preset::Kind::Percussive(freq) => {
                voice.freq = freq;
            }
        };

        voice.env = Env::new(instrument.shape);

        voice.lfos.clear();
        for &(waveform, freq, gain) in &instrument.lfos {
            voice
                .lfos
                .push(Osc::new(waveform, freq.into(), SAMPLE_RATE, gain));
        }

        voice.oscs.clear();
        for &(waveform, gain) in &instrument.oscs {
            voice
                .oscs
                .push(Osc::new(waveform, voice.freq, SAMPLE_RATE, gain));
        }
    }

    fn note_on(&mut self, inst: usize, note: u8) {
        self.init_voice(inst, Some(note));
    }

    fn note_off(&mut self, inst: usize, note: u8) {
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.active && v.inst_id == inst && v.note == note)
        {
            v.env.note_off();
        }
    }

    fn trigger(&mut self, inst: usize) {
        self.init_voice(inst, None);
    }

    fn process(&mut self, buf: &mut [f32]) {
        let dt = 1.0 / SAMPLE_RATE;

        for _ in 0..128 {
            let Ok(event) = self.rx.try_recv() else {
                break;
            };
            match event {
                Event::NoteOn(inst, note) => self.note_on(inst, note),
                Event::NoteOff(inst, note) => self.note_off(inst, note),
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

                let lfo = voice.lfos.iter_mut().map(|lfo| lfo.next()).sum::<f64>();

                let sum = voice
                    .oscs
                    .iter_mut()
                    .map(|osc| {
                        osc.mod_freq(lfo);
                        osc.next()
                    })
                    .sum::<f64>();

                // mix += amp * (sum / voice.oscs.len() as f64);
                mix += amp * sum;
            }

            // master gain
            *sample = (0.2 * mix) as f32;
        }
    }
}

#[allow(unused)]
struct Sequencer {
    bpm: f64,
    beats: u8,
    sub_beats: u8,
    beat_duration: Duration,
    total_beats: usize,
    current_beat: usize,
    last_time: Instant,
    // channels: Vec<(usize, Vec<u8>)>,
    channels: Vec<(usize, u16)>,
    tx: mpsc::Sender<Event>,
}

impl Sequencer {
    fn new(bpm: f64, beats: u8, sub_beats: u8, tx: mpsc::Sender<Event>) -> Self {
        Self {
            bpm,
            beats,
            sub_beats,
            beat_duration: Duration::from_secs_f64(60.0 / bpm / sub_beats as f64),
            current_beat: 0,
            last_time: Instant::now(),
            channels: Vec::new(),
            total_beats: (beats * sub_beats) as usize,
            tx,
        }
    }

    fn update(&mut self) {
        if Instant::now().duration_since(self.last_time) >= self.beat_duration {
            // print!("current_beat = {}", self.current_beat);

            for &(ch, mask) in &self.channels {
                if mask & (1 << self.current_beat) != 0 {
                    // print!(" *");
                    _ = self.tx.send(Event::Trigger(ch));
                }
            }
            // println!();
            self.last_time += self.beat_duration;
            self.current_beat = (self.current_beat + 1) % self.total_beats;
        }
    }

    fn add_channel(&mut self, inst_id: usize, pat: &str) {
        assert_eq!(pat.len(), self.total_beats);

        let mut mask = 0u16;
        for (i, chr) in pat.char_indices() {
            if chr == 'x' {
                mask |= 1 << i;
            }
        }

        // println!("{inst_id} {pat} {mask:016b} {}", mask.count_ones());
        self.channels.push((inst_id, mask));
    }
}

const SAMPLE_RATE: f64 = 44_100.0;

fn main() {
    let (tx, rx) = mpsc::channel();

    let instrument = Instrument::builder()
        .lfo(Waveform::Sine, 3.0, 0.02)
        .osc(Waveform::Sine, 1.0)
        .osc(Waveform::Saw, 0.2)
        .env(0.002, 0.1, 0.8, 0.2)
        .build();

    let instruments = vec![instrument, preset::kick(), preset::snare(), preset::hihat()];

    let mut synth = Synth::<32>::new(rx, instruments);
    let engine = Engine::new(SAMPLE_RATE, move |buf| synth.process(buf));

    engine.start();

    let mut keyboard = Keyboard::new();

    let mut seq = Sequencer::new(60.0, 4, 4, tx.clone());
    // seq.add_channel(1, "x...x...x...x...");
    // seq.add_channel(2, ".xxx.xxx.xxx.xxx");
    // seq.add_channel(3, "x.x.x.x.x.x.x.x.");

    seq.add_channel(3, "x...x...x...x...");
    seq.add_channel(1, ".xxx.xxx.xxx.xxx");

    ratatui::init();

    loop {
        seq.update();

        for (note, key) in keyboard.keys.iter_mut().enumerate() {
            let down = kbd::is_key_down(key.code);

            if down && !key.pressed {
                key.pressed = true;
                _ = tx.send(Event::NoteOn(0, note as u8));
            }

            if !down && key.pressed {
                key.pressed = false;
                _ = tx.send(Event::NoteOff(0, note as u8));
            }
        }

        if kbd::is_key_down(KeyCode::Q) {
            break;
        }

        thread::sleep(Duration::from_millis(2));
    }

    engine.stop();

    ratatui::restore();
}
