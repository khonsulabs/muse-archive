use lazy_static::lazy_static;
use rodio::Source;
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

mod sawtooth;
mod sine;
mod square;
mod triangle;
use crate::parameter::Parameter;
pub use sawtooth::Sawtooth;
pub use sine::Sine;
pub use square::Square;
pub use triangle::Triangle;

lazy_static! {
    static ref STARTUP_INSTANT: Instant = Instant::now();
}

#[derive(Debug, Clone)]
pub struct Oscillator<T> {
    frequency: f32,
    amplitude: Parameter,
    current_sample: usize,
    _of: std::marker::PhantomData<T>,
}

impl<T> Oscillator<T> {
    pub fn new(frequency: f32, amplitude: Parameter) -> Self {
        Self {
            frequency,
            amplitude,
            current_sample: Self::initial_sample(),
            _of: std::marker::PhantomData::default(),
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

pub trait OscillatorFunction {
    fn compute_sample(value: f32) -> f32;
}

impl<T> Iterator for Oscillator<T>
where
    T: OscillatorFunction,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.current_sample = self.current_sample.wrapping_add(1);
        let value = self.frequency * 2.0 * std::f32::consts::PI * self.current_sample as f32
            / Self::sample_rate() as f32;
        let value = value % (2.0 * PI);

        self.amplitude
            .next(self.sample_rate())
            .map(|amplification| amplification * T::compute_sample(value))
    }
}

impl<T> Source for Oscillator<T>
where
    T: OscillatorFunction,
{
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        Oscillator::<T>::sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
