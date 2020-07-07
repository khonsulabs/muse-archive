mod add;
mod amplify;
mod max;
mod multiply;
mod oscillator;
mod pan;
mod unison;
use crate::Note;
pub use add::*;
pub use amplify::*;
pub use max::*;
pub use multiply::*;
pub use oscillator::*;
pub use pan::*;
pub use unison::*;

fn clampf(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    pub left: f32,
    pub right: f32,
}

impl Sample {
    pub fn clamped(&self) -> Sample {
        Self {
            left: clampf(self.left, -1., 1.),
            right: clampf(self.right, -1., 1.),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub clock: usize,
    pub sample_rate: u32,
    pub note: Note,
}

impl FrameInfo {
    pub fn with_note(&self, note: Note) -> Self {
        Self {
            clock: self.clock,
            sample_rate: self.sample_rate,
            note,
        }
    }
}

pub trait Sampler: Send + Sync + std::fmt::Debug {
    fn sample(&mut self, frame: &FrameInfo) -> Option<Sample>;
}

#[derive(Debug)]
pub struct PreparedSampler {
    sampler: Box<dyn Sampler + 'static>,
    pub still_producing_samples: bool,
}

impl Sampler for PreparedSampler {
    fn sample(&mut self, frame: &FrameInfo) -> Option<Sample> {
        if self.still_producing_samples {
            match self.sampler.sample(frame) {
                Some(sample) => Some(sample),
                None => {
                    self.still_producing_samples = false;
                    None
                }
            }
        } else {
            None
        }
    }
}

impl PreparedSampler {
    pub fn new<T: Sampler + 'static>(sampler: T) -> Self {
        Self {
            sampler: Box::new(sampler),
            still_producing_samples: true,
        }
    }
}

pub trait PreparableSampler {
    fn prepare(self) -> PreparedSampler;
}

impl<T> PreparableSampler for T
where
    T: Sampler + 'static,
{
    fn prepare(self) -> PreparedSampler {
        PreparedSampler::new(self)
    }
}

impl std::ops::Add<Sample> for Sample {
    type Output = Self;

    fn add(self, rhs: Sample) -> Self::Output {
        Self {
            left: self.left + rhs.left,
            right: self.right + rhs.right,
        }
    }
}

impl std::ops::AddAssign<Sample> for Sample {
    fn add_assign(&mut self, rhs: Sample) {
        self.left += rhs.left;
        self.right += rhs.right;
    }
}

impl std::ops::Mul<f32> for Sample {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            left: self.left * rhs,
            right: self.right * rhs,
        }
    }
}

impl std::ops::MulAssign<f32> for Sample {
    fn mul_assign(&mut self, rhs: f32) {
        self.left *= rhs;
        self.right *= rhs;
    }
}

impl std::ops::Mul<Sample> for Sample {
    type Output = Self;

    fn mul(self, rhs: Sample) -> Self::Output {
        Self {
            left: self.left * rhs.left,
            right: self.right * rhs.right,
        }
    }
}

impl std::ops::MulAssign<Sample> for Sample {
    fn mul_assign(&mut self, rhs: Sample) {
        self.left *= rhs.left;
        self.right *= rhs.right;
    }
}

impl std::ops::Div<f32> for Sample {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            left: self.left / rhs,
            right: self.right / rhs,
        }
    }
}

pub mod prelude {
    pub use super::{
        add::*, amplify::*, max::*, multiply::*, oscillator::*, pan::*, PreparableSampler,
        PreparedSampler, Sample, Sampler,
    };
}
