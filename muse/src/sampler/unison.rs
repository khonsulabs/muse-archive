use crate::{
    note::Note,
    parameter::Parameter,
    sampler::{FrameInfo, PreparedSampler, Sample, Sampler},
};

#[derive(Debug)]
pub struct Unison {
    detune: Parameter,
    samplers: Vec<PreparedSampler>,
}

impl Unison {
    pub fn new(detune: Parameter, samplers: Vec<PreparedSampler>) -> Self {
        Self { detune, samplers }
    }
}

impl Sampler for Unison {
    fn sample(&mut self, frame: &FrameInfo) -> Option<Sample> {
        if let Some(detune) = self.detune.next(frame) {
            let steps = self.samplers.len() as f32 - 1.;
            let detune_step = if steps > 0. { detune / steps } else { 0.0 };
            let pitch_floor = frame.note.step() - detune_step * steps;
            let samples = self
                .samplers
                .iter_mut()
                .enumerate()
                .filter_map(|(i, sampler)| {
                    let frame = frame.with_note(Note::new(
                        pitch_floor + detune_step * i as f32,
                        frame.note.velocity(),
                    ));
                    sampler.sample(&frame)
                })
                .collect::<Vec<_>>();

            if !samples.is_empty() {
                let sample_count = samples.len();
                return Some(
                    samples
                        .into_iter()
                        .fold(Sample::default(), |output, s| {
                            output + (s / sample_count as f32)
                        })
                        .clamped(),
                );
            }
        }

        None
    }
}
