use crate::{
    parameter::Parameter,
    sampler::{Sample, Sampler},
};

#[derive(Debug)]
pub struct Pan {
    pan: Parameter,
    source: Box<dyn Sampler + Send + Sync>,
}

#[derive(Clone, Debug)]
struct PanFrame {
    sample: Sample,
    pan: f32,
}

impl Pan {
    pub fn new<T: Sampler + Send + Sync + 'static>(parameter: Parameter, source: T) -> Self {
        Self {
            pan: parameter,
            source: Box::new(source),
        }
    }
}

impl Sampler for Pan {
    fn sample(&mut self, sample_rate: u32) -> Option<Sample> {
        if let Some(sample) = self.source.sample(sample_rate) {
            if let Some(pan) = self.pan.next(sample_rate) {
                return Some(Sample {
                    left: sample.left * (1. - pan),
                    right: sample.right * pan,
                });
            }
        }

        None
    }
}
