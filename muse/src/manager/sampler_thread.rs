use super::ManagerHandle;
use crate::sampler::{FrameInfo, PreparedSampler, Sample, Sampler};
use crossbeam::{
    channel::{unbounded, Receiver, Sender},
    sync::ShardedLock,
};
use std::sync::Arc;

fn desired_threads() -> usize {
    // TOo many threads causes high CPU usage when it's really not necessary
    // For a machine that reports 5 cores or less, we want to use 2 threads
    // For machines in the sweet spot of 4-8 cores, we'll use half the number of CPUs
    // Any machines with more than 8 CPUs will just use 4 threads.
    // TODO Should this be configurable?
    (num_cpus::get() / 2).min(2).max(4)
}

pub fn run(manager: ManagerHandle, sender: Sender<Sample>, format: cpal::Format) {
    let (sample_sender, sample_receiver) = unbounded();
    let (result_sender, result_receiver) = unbounded();

    let thread_count = desired_threads();
    (0..thread_count).for_each(|_| {
        let sample_receiver = sample_receiver.clone();
        let result_sender = result_sender.clone();
        std::thread::spawn(|| sampler_thread_main(sample_receiver, result_sender));
    });
    let mut thread = SamplerThread {
        manager,
        sender,
        format,
        sample_sender,
        result_receiver,
    };

    thread.run();
}

struct SamplerThread {
    sample_sender: Sender<(FrameInfo, Arc<ShardedLock<PreparedSampler>>)>,
    result_receiver: Receiver<Option<Sample>>,
    manager: ManagerHandle,
    sender: Sender<Sample>,
    format: cpal::Format,
}

impl SamplerThread {
    fn run(&mut self) {
        loop {
            let sample = self.next_sample();
            if let Err(err) = self.sender.send(sample) {
                println!("Error on sampler thread: {}", err);
                break;
            }
        }
    }

    fn next_sample(&mut self) -> Sample {
        let mut manager = self
            .manager
            .write()
            .expect("Error locking manager for sampling");
        let clock = manager.increment_clock();
        for sound in manager.playing_sounds.iter() {
            let frame = FrameInfo {
                clock,
                sample_rate: self.format.sample_rate.0,
                note: sound.note,
            };
            let _ = self.sample_sender.send((frame, sound.sampler.clone()));
        }

        (0..manager.playing_sounds.len())
            .filter_map(|_| self.result_receiver.recv().ok().flatten())
            .sum()
    }
}

fn sampler_thread_main(
    samplers: Receiver<(FrameInfo, Arc<ShardedLock<PreparedSampler>>)>,
    results: Sender<Option<Sample>>,
) {
    while let Ok((frame, sampler)) = samplers.recv() {
        let sample = {
            let mut sampler = sampler.write().expect("Error locking sampler");
            sampler.sample(&frame)
        };

        if results.send(sample).is_err() {
            break;
        }
    }
}
