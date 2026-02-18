mod engine;
pub use engine::Engine;

pub mod kbd;

pub mod osc;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Hz(pub f64);

impl Hz {
    /// Convert frequency (Hz) to angular velocity (ω).
    #[inline(always)]
    pub fn w(&self) -> f64 {
        self.0 * 2.0 * std::f64::consts::PI
    }

    pub fn from_pitch_std(semitones: i32) -> Self {
        const PITCH_STANDARD: f64 = 440.0;
        const TWELFTH_ROOT_OF_TWO: f64 = 1.0594630943592952646;

        Hz(PITCH_STANDARD * TWELFTH_ROOT_OF_TWO.powi(semitones))
    }
}

impl From<f64> for Hz {
    fn from(x: f64) -> Self {
        Self(x)
    }
}
