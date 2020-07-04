use crate::{
    parameter::Parameter,
    sampler::{FrameInfo, PreparableSampler, PreparedSampler, Sample, Sampler},
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
    fn sample(&mut self, frame: &FrameInfo) -> Option<Sample> {
        if let Some(sample) = self.source.sample(frame) {
            if let Some(amplify) = self.amplify.next(frame) {
                return Some(sample * amplify);
            }
        }

        None
    }
}
