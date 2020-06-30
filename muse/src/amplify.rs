use crate::{
    parameter::Parameter,
    sampler::{PreparableSampler, PreparedSampler, Sample, Sampler},
};

#[derive(Debug)]
pub struct Amplify {
    amplify: Parameter,
    source: PreparedSampler,
}

impl Amplify {
    pub fn new<T: PreparableSampler>(amplify: Parameter, source: T) -> Self {
        Self {
            amplify,
            source: source.prepare(),
        }
    }
}

impl Sampler for Amplify {
    fn sample(&mut self, sample_rate: u32) -> Option<Sample> {
        if let Some(sample) = self.source.sample(sample_rate) {
            if let Some(amplify) = self.amplify.next(sample_rate) {
                return Some(sample * amplify);
            }
        }

        None
    }
}
