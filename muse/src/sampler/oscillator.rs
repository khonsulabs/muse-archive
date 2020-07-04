use crate::sampler::{FrameInfo, Sample, Sampler};
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

#[derive(Debug)]
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
    T: OscillatorFunction + 'static,
{
    fn sample(&mut self, frame: &FrameInfo) -> Option<Sample> {
        let current_sample = frame.clock as f32 / frame.sample_rate as f32;
        let frequency = self.frequency.next(frame)?;
        let value = current_sample * frequency * 2.0 * std::f32::consts::PI;
        let value = value % (2.0 * PI);
        let sample = T::compute_sample(value);

        self.amplitude.next(frame).map(|amplification| Sample {
            left: amplification * sample / 2.0,
            right: amplification * sample / 2.0,
        })
    }
}
