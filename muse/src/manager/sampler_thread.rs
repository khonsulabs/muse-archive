use super::ManagerHandle;
use crate::sampler::{FrameInfo, Sample, Sampler};
use crossbeam::channel::Sender;

pub fn run(manager: ManagerHandle, sender: Sender<Sample>, format: cpal::Format) {
    loop {
        if let Err(err) = sender.send(next_sample(&manager, &format).clamped()) {
            println!("Error on sampler thread: {}", err);
            break;
        }
    }
}

fn next_sample(manager: &ManagerHandle, format: &cpal::Format) -> Sample {
    let mut manager = manager.write().expect("Error locking manager for sampling");
    let clock = manager.increment_clock();
    let mut combined_sample = Sample::default();
    for sample in manager.playing_sounds.iter_mut().filter_map(|s| {
        let frame = FrameInfo {
            clock,
            sample_rate: format.sample_rate.0,
            note: s.note,
        };
        let sample = s.sampler.sample(&frame);
        if sample.is_none() {
            s.still_producing_values = false;
        }
        sample
    }) {
        combined_sample += sample;
    }
    combined_sample
}
