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
    frequency: f32,
    amplitude: Parameter,
    current_sample: Option<usize>,
    _of: std::marker::PhantomData<T>,
}

impl<T> Oscillator<T> {
    pub fn new(frequency: f32, amplitude: Parameter) -> Self {
        Self {
            frequency,
            amplitude,
            current_sample: None,
            _of: std::marker::PhantomData::default(),
        }
    }

    fn initial_sample(sample_rate: u32) -> usize {
        let now = Instant::now();
        let subsec = now
            .checked_duration_since(*STARTUP_INSTANT)
            .unwrap_or_default()
            .subsec_nanos() as f32
            / 1_000_000_000.0;
        let current_sample = subsec * sample_rate as f32;
        current_sample as usize
    }

    fn next_sample(&mut self, sample_rate: u32) -> usize {
        let new_sample = match self.current_sample {
            Some(existing) => existing.wrapping_add(1),
            None => Self::initial_sample(sample_rate),
        };
        self.current_sample = Some(new_sample);
        new_sample
    }
}

pub trait OscillatorFunction: Send + Sync + std::fmt::Debug {
    fn compute_sample(value: f32) -> f32;
}

impl<T> Sampler for Oscillator<T>
where
    T: OscillatorFunction,
{
    fn sample(&mut self, sample_rate: u32) -> Option<Sample> {
        let current_sample = self.next_sample(sample_rate) as f32;
        let value =
            self.frequency * 2.0 * std::f32::consts::PI * current_sample / sample_rate as f32;
        let value = value % (2.0 * PI);
        let sample = T::compute_sample(value);

        self.amplitude
            .next(sample_rate)
            .map(|amplification| Sample {
                left: amplification * sample / 2.0,
                right: amplification * sample / 2.0,
            })
    }
}
