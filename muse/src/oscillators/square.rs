use super::Oscillator;
use rodio::Source;
use std::{f32::consts::PI, time::Duration};

#[derive(Clone, Debug)]
pub struct Square {
    oscillator: Oscillator,
}

impl Square {
    pub fn new(frequency: f32) -> Self {
        Self {
            oscillator: Oscillator::new(frequency),
        }
    }
}

impl Iterator for Square {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.oscillator.next().map(square_wave)
    }
}

impl Source for Square {
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

fn square_wave(value: f32) -> f32 {
    let value = value % (2.0 * PI);
    if value < PI {
        1.0
    } else {
        -1.0
    }
}
