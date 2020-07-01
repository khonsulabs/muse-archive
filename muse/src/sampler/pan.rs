use crate::{
    parameter::Parameter,
    sampler::{PreparableSampler, PreparedSampler, Sample, Sampler},
};

#[derive(Debug)]
pub struct Pan {
    pan: Parameter,
    source: PreparedSampler,
}

#[derive(Clone, Debug)]
struct PanFrame {
    sample: Sample,
    pan: f32,
}

impl Pan {
    pub fn new<T: PreparableSampler>(parameter: Parameter, source: T) -> Self {
        Self {
            pan: parameter,
            source: source.prepare(),
        }
    }
}

impl Sampler for Pan {
    fn sample(&mut self, sample_rate: u32, clock: usize) -> Option<Sample> {
        if let Some(sample) = self.source.sample(sample_rate, clock) {
            if let Some(pan) = self.pan.next(sample_rate, clock) {
                return Some(Sample {
                    left: sample.left * (1. - pan),
                    right: sample.right * pan,
                });
            }
        }

        None
    }
}
