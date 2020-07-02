use super::OscillatorFunction;

#[derive(Debug)]
pub struct Sine {}

impl OscillatorFunction for Sine {
    fn compute_sample(value: f32) -> f32 {
        value.sin()
    }
}
