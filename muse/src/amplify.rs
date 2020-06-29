use crate::{
    parameter::Parameter,
    sampler::{Sample, Sampler},
};

#[derive(Debug)]
pub struct Amplify {
    amplify: Parameter,
    source: Box<dyn Sampler + Send + Sync>,
}

impl Amplify {
    pub fn new<T: Sampler + Send + Sync + 'static>(amplify: Parameter, source: T) -> Self {
        Self {
            amplify,
            source: Box::new(source),
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
