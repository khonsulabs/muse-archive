use super::OscillatorFunction;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Square {}

impl OscillatorFunction for Square {
    fn compute_sample(value: f32) -> f32 {
        if value < PI {
            1.0
        } else {
            -1.0
        }
    }
}
