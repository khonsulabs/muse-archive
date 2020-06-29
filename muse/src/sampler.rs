#[derive(Clone, Debug, Default)]
pub struct Sample {
    pub left: f32,
    pub right: f32,
}

pub trait Sampler: Send + Sync + std::fmt::Debug {
    fn sample(&mut self, sample_rate: u32) -> Option<Sample>;
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
