use super::OscillatorFunction;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Sawtooth {}

impl OscillatorFunction for Sawtooth {
    fn compute_sample(value: f32) -> f32 {
        if value <= PI {
            value / PI
        } else {
            ((value - PI) / PI) - 1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sawtooth;
    use crate::sampler::oscillator::OscillatorFunction;
    use approx::assert_ulps_eq;
    use std::f32::consts::PI;

    #[test]
    fn sawtooth_tests() {
        assert_ulps_eq!(Sawtooth::compute_sample(0.0), 0.0);
        assert_ulps_eq!(Sawtooth::compute_sample(PI), 1.0);
        assert_ulps_eq!(Sawtooth::compute_sample(PI + f32::EPSILON), -1.0);
        assert_ulps_eq!(Sawtooth::compute_sample(PI * 2.0), 0.0);
    }
}
