use lazy_static::lazy_static;
use std::time::Instant;

pub mod sine;

lazy_static! {
    static ref STARTUP_INSTANT: Instant = Instant::now();
}

#[derive(Debug, Clone)]
pub struct Oscillator {
    frequency: f32,
    current_sample: usize,
}

impl Oscillator {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            current_sample: Self::initial_sample(),
        }
    }

    fn initial_sample() -> usize {
        let now = Instant::now();
        let subsec = now
            .checked_duration_since(*STARTUP_INSTANT)
            .unwrap_or_default()
            .subsec_nanos() as f32
            / 1_000_000_000.0;
        let current_sample = subsec * Self::sample_rate() as f32;
        current_sample as usize
    }

    pub const fn sample_rate() -> u32 {
        48000
    }
}
impl Iterator for Oscillator {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.current_sample = self.current_sample.wrapping_add(1);

        Some(
            self.frequency * 2.0 * std::f32::consts::PI * self.current_sample as f32
                / Self::sample_rate() as f32,
        )
    }
}
