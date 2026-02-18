use std::f64::consts::PI;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[repr(C)]
struct AudioEngine(());

type AudioCallback = extern "C" fn(user_data: *mut c_void, buffer: *mut f32, frame_count: u32);

unsafe extern "C" {
    fn audio_engine_new(ud: *mut c_void, cb: AudioCallback, sample_rate: f64) -> *mut AudioEngine;
    fn audio_engine_start(engine: *mut AudioEngine);
    fn audio_engine_stop(engine: *mut AudioEngine);
    fn audio_engine_free(engine: *mut AudioEngine);
}

#[inline]
fn is_key_down(key: KeyboardKey) -> bool {
    unsafe extern "C" {
        #[link_name = "is_key_down"]
        fn c_is_key_down(keycode: u16) -> bool;
    }
    unsafe { c_is_key_down(key as u16) }
}

extern "C" fn trampoline<F>(user_data: *mut c_void, buf: *mut f32, frame_count: u32)
where
    F: FnMut(&mut [f32]) + Send + 'static,
{
    unsafe {
        let process = user_data as *mut F;
        let buf = std::slice::from_raw_parts_mut(buf, frame_count as usize);

        (*process)(buf);
    }
}

struct Engine<F> {
    inner: *mut AudioEngine,
    _process: Box<F>,
}

impl<F> Engine<F>
where
    F: FnMut(&mut [f32]) + Send + 'static,
{
    fn new(sample_rate: f64, process: F) -> Self {
        let mut process = Box::new(process);

        let inner = unsafe {
            audio_engine_new(
                process.as_mut() as *mut _ as *mut c_void,
                trampoline::<F>,
                sample_rate,
            )
        };

        Self {
            inner,
            _process: process,
        }
    }

    fn start(&self) {
        unsafe { audio_engine_start(self.inner) };
    }

    fn stop(&self) {
        unsafe { audio_engine_stop(self.inner) };
    }
}

impl<T> Drop for Engine<T> {
    fn drop(&mut self) {
        unsafe { audio_engine_free(self.inner) };
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardKey {
    Q = 12,
    W = 13,
    E = 14,
    R = 15,
    T = 17,
    Y = 16,
    U = 32,
    I = 34,
    O = 31,
    P = 35,
    LBracket = 33,
    RBracket = 30,
    BSlash = 42,

    A = 0,
    S = 1,
    D = 2,
    F = 3,
    G = 5,
    H = 4,
    J = 38,
    K = 40,
    L = 37,
    Semi = 41,
    Quote = 39,
    Enter = 36,
    Home = 115,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Hz(f64);

impl Hz {
    fn from_pitch_std(semitones: i32) -> Self {
        const PITCH_STANDARD: f64 = 440.0;
        const TWELFTH_ROOT_OF_TWO: f64 = 1.0594630943592952646;

        Hz(PITCH_STANDARD * TWELFTH_ROOT_OF_TWO.powi(semitones))
    }

    /// Convert frequency (Hz) to angular velocity (ω).
    #[inline(always)]
    fn w(&self) -> f64 {
        self.0 * 2.0 * PI
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
    let piano = [(KeyboardKey::A, Note::from_pitch_std(-9))];
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
            let pressed = is_key_down(*key);

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

        if is_key_down(KeyboardKey::Q) {
            break;
        }

        thread::sleep(Duration::from_millis(1));
    }

    engine.stop();
}
