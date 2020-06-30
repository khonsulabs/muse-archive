pub mod add;
pub mod amplify;
pub mod max;
pub mod multiply;
pub mod oscillator;
pub mod pan;

#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    pub left: f32,
    pub right: f32,
}

pub trait Sampler: Send + Sync + std::fmt::Debug {
    fn sample(&mut self, sample_rate: u32, clock: usize) -> Option<Sample>;
}

#[derive(Debug)]
pub struct PreparedSampler(Box<dyn Sampler + Send + Sync + 'static>);

impl Sampler for PreparedSampler {
    fn sample(&mut self, sample_rate: u32, clock: usize) -> Option<Sample> {
        self.0.sample(sample_rate, clock)
    }
}

impl PreparedSampler {
    pub fn new<T: Sampler + 'static>(sampler: T) -> Self {
        Self(Box::new(sampler))
    }
}

pub trait PreparableSampler {
    fn prepare(self) -> PreparedSampler;
}

impl<T> PreparableSampler for T
where
    T: Sampler + Send + Sync + 'static,
{
    fn prepare(self) -> PreparedSampler {
        PreparedSampler::new(self)
    }
}

// impl<T> PreparableSampler for Box<T>
// where
//     T: Sampler + Send + Sync + 'static,
// {
//     fn prepare(self) -> PreparedSampler {
//         self
//     }
// }

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

pub mod prelude {
    pub use super::{
        add::*, amplify::*, max::*, multiply::*, oscillator::*, pan::*, PreparableSampler,
        PreparedSampler, Sample, Sampler,
    };
}
