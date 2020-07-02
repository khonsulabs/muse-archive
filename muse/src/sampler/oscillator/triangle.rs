use super::OscillatorFunction;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Triangle {}

impl OscillatorFunction for Triangle {
    fn compute_sample(value: f32) -> f32 {
        PI - (value - PI).abs()
    }
}
