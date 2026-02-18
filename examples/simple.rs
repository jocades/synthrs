use std::{f64::consts::PI, thread, time::Duration};
use synth::Engine;

const SAMPLE_RATE: f64 = 44_100.0;

fn main() {
    let amp = 0.5;
    let freq = 440.0; // Hz
    let mut phase = 0.0f64; // Rad

    let engine = Engine::new(SAMPLE_RATE, move |buf| {
        for sample in buf {
            *sample = (amp * phase.sin()) as f32;
            phase += freq * 2.0 * PI / SAMPLE_RATE;
        }
    });

    engine.start();
    thread::sleep(Duration::from_secs(2));
    engine.stop();
}
