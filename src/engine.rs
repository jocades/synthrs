use std::ffi::c_void;

#[repr(C)]
struct AudioEngine(());

type AudioCallback = extern "C" fn(user_data: *mut c_void, buffer: *mut f32, frame_count: u32);

unsafe extern "C" {
    fn audio_engine_new(ud: *mut c_void, cb: AudioCallback, sr: f64) -> *mut AudioEngine;
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

pub struct Engine<F> {
    inner: *mut AudioEngine,
    _process: Box<F>,
}

impl<F> Engine<F>
where
    F: FnMut(&mut [f32]) + Send + 'static,
{
    pub fn new(sample_rate: f64, process: F) -> Self {
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

    pub fn start(&self) {
        unsafe { audio_engine_start(self.inner) };
    }

    pub fn stop(&self) {
        unsafe { audio_engine_stop(self.inner) };
    }
}

impl<T> Drop for Engine<T> {
    fn drop(&mut self) {
        unsafe { audio_engine_free(self.inner) };
    }
}
