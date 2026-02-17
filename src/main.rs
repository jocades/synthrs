use std::f64::consts::PI;
use std::ffi::c_void;

#[repr(C)]
struct AudioEngine(());

type AudioCallback = extern "C" fn(user_data: *mut c_void, buffer: *mut f32, frame_count: u32);

unsafe extern "C" {
    fn audio_engine_new(ud: *mut c_void, cb: AudioCallback, sample_rate: f64) -> *mut AudioEngine;
    fn audio_engine_start(engine: *mut AudioEngine);
    fn audio_engine_stop(engine: *mut AudioEngine);
    fn audio_engine_free(engine: *mut AudioEngine);
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

fn main() {
    let mut phase = 0.0f64;
    let freq = 440.0;
    let sr = 44_100.0;
    let inc = 2.0 * PI * freq / sr;

    let engine = Engine::new(sr, move |buf| {
        for s in buf {
            *s = (phase.sin() * 0.2) as f32;
            phase += inc;
            if phase > 2.0 * PI {
                phase -= 2.0 * PI;
            }
        }
    });

    engine.start();
    std::thread::sleep(std::time::Duration::from_secs(2));
    engine.stop();
}
