use crate::{
    parameter::Parameter,
    sampler::{FrameInfo, PreparableSampler, PreparedSampler, Sample, Sampler},
};

#[derive(Debug)]
pub struct Pan {
    pan: Parameter,
    source: PreparedSampler,
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
    fn sample(&mut self, frame: &FrameInfo) -> Option<Sample> {
        if let Some(sample) = self.source.sample(frame) {
            if let Some(pan) = self.pan.next(frame) {
                return Some(Sample {
                    left: sample.left * (1. - pan),
                    right: sample.right * pan,
                });
            }
        }

        None
    }
}
