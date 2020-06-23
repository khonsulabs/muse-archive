use super::Oscillator;
use rodio::Source;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Sine {
    oscillator: Oscillator,
}

impl Sine {
    pub fn new(frequency: f32) -> Self {
        Self {
            oscillator: Oscillator::new(frequency),
        }
    }
}

impl Iterator for Sine {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.oscillator.next().map(|v: f32| v.sin())
    }
}

impl Source for Sine {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        Oscillator::sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
