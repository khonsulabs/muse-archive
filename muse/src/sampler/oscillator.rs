use crate::sampler::{Sample, Sampler};
use lazy_static::lazy_static;
use std::{f32::consts::PI, time::Instant};

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
    frequency: Parameter,
    amplitude: Parameter,
    _of: std::marker::PhantomData<T>,
}

impl<T> Oscillator<T> {
    pub fn new(frequency: Parameter, amplitude: Parameter) -> Self {
        Self {
            frequency,
            amplitude,
            _of: std::marker::PhantomData::default(),
        }
    }
}

pub trait OscillatorFunction: Send + Sync + std::fmt::Debug {
    fn compute_sample(value: f32) -> f32;
}

impl<T> Sampler for Oscillator<T>
where
    T: OscillatorFunction,
{
    fn sample(&mut self, sample_rate: u32, clock: usize) -> Option<Sample> {
        let current_sample = clock as f32 / sample_rate as f32;
        let frequency = self.frequency.next(sample_rate, clock)?;
        let value = current_sample * frequency * 2.0 * std::f32::consts::PI;
        let value = value % (2.0 * PI);
        let sample = T::compute_sample(value);

        self.amplitude
            .next(sample_rate, clock)
            .map(|amplification| Sample {
                left: amplification * sample / 2.0,
                right: amplification * sample / 2.0,
            })
    }
}
