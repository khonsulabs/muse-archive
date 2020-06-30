use crate::sampler::{PreparedSampler, Sample, Sampler};

#[derive(Debug)]
pub struct Add {
    sources: Vec<PreparedSampler>,
}

impl Add {
    pub fn new(sources: Vec<PreparedSampler>) -> Self {
        Self { sources }
    }
}

impl Sampler for Add {
    fn sample(&mut self, sample_rate: u32, clock: usize) -> Option<Sample> {
        let mut result: Option<Sample> = None;
        for sample in self
            .sources
            .iter_mut()
            .filter_map(|s| s.sample(sample_rate, clock))
        {
            result = result
                .map(|mut existing| {
                    existing += sample;
                    existing
                })
                .or(Some(sample))
        }
        result
    }
}
