use crate::{note::Note, sampler::PreparedSampler};
use cpal::traits::{DeviceTrait, StreamTrait};
use crossbeam::{
    channel::{bounded, unbounded, Receiver, RecvTimeoutError, Sender},
    sync::ShardedLock,
};
use std::{sync::Arc, time::Duration};
mod cpal_thread;
mod device;
mod sampler_thread;
pub use device::Device;

pub(crate) enum ManagerMessage {
    Append {
        note: Note,
        sampler: PreparedSampler,
        callback: Sender<PlayingHandle>,
    },
}

pub type ManagerHandle = Arc<ShardedLock<Manager>>;

#[derive(Clone, Debug)]
pub struct PlayingHandle(Arc<u64>);

#[derive(Debug)]
struct PlayingSound {
    note: Note,
    handle: PlayingHandle,
    sampler: Arc<ShardedLock<PreparedSampler>>,
}

impl PlayingSound {
    fn still_producing_values(&self) -> bool {
        let sampler = self.sampler.read().expect("Error reading sampler");
        sampler.still_producing_samples
    }
}

// TODO add derivative and impl debug but skip stream
pub struct Manager {
    playing_sounds: Vec<PlayingSound>,
    last_playing_sound_id: u64,
    clock: usize,
    pub(crate) sender: Sender<ManagerMessage>,
}

impl Manager {
    pub(crate) fn open_output_device(
        output_device: cpal::Device,
        format: cpal::StreamConfig,
    ) -> Result<ManagerHandle, anyhow::Error> {
        let (sender, receiver) = unbounded();

        let (sample_sender, sample_receiver) = bounded(1024);

        let thread_format = format.clone();

        let manager = Arc::new(ShardedLock::new(Manager::new(sender)));
        // event_loop.play_stream(output_stream_id.clone())?;

        let manager_for_thread = manager.clone();
        std::thread::Builder::new()
            .name("muse::manager".to_owned())
            .spawn(move || {
                let callback_format = thread_format.clone();
                let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    cpal_thread::copy_samples(&sample_receiver, data, &callback_format).unwrap();
                    // TODO BAD UNWRAP
                };
                let output_stream = output_device
                    .build_output_stream(&thread_format, output_data_fn, error_fn)
                    .unwrap(); // TODO unwrap here is bad

                ManagerThread::new(manager_for_thread, receiver)
                    .main(output_stream)
                    .unwrap_or_default()
            })?;

        let manager_for_thread = manager.clone();
        std::thread::Builder::new()
            .name("muse::sampler".to_owned())
            .spawn(move || sampler_thread::run(manager_for_thread, sample_sender, format))?;

        Ok(manager)
    }

    fn new(sender: Sender<ManagerMessage>) -> Self {
        Self {
            sender,
            playing_sounds: Vec::new(),
            last_playing_sound_id: 0,
            clock: 0,
        }
    }

    fn increment_clock(&mut self) -> usize {
        self.clock = self.clock.wrapping_add(1);
        self.clock
    }
}

fn error_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

struct ManagerThread {
    receiver: Receiver<ManagerMessage>,
    manager: Arc<ShardedLock<Manager>>,
}

impl ManagerThread {
    fn new(manager: Arc<ShardedLock<Manager>>, receiver: Receiver<ManagerMessage>) -> Self {
        Self { manager, receiver }
    }

    fn main(&mut self, stream: cpal::Stream) -> Result<(), RecvTimeoutError> {
        stream.play().unwrap();

        // TODO cpal now supports the concept of pausing when nothing is playing. It'd be great if we could handle that situation.
        loop {
            // Check for new messages
            if let Err(err) = self.handle_incoming_messages() {
                if let RecvTimeoutError::Timeout = err {
                    // Let the loop fall through
                } else {
                    return Err(err);
                }
            }

            self.release_completed_sounds();
        }
    }

    fn handle_incoming_messages(&mut self) -> Result<(), RecvTimeoutError> {
        match self.receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(ManagerMessage::Append {
                note,
                sampler,
                callback,
            }) => {
                let handle = {
                    // Scope this write so that sending the handle across the callback doesn't happen while the lock is still held
                    let mut manager = self
                        .manager
                        .write()
                        .expect("Error locking manager to add sampler");
                    manager.last_playing_sound_id = manager.last_playing_sound_id.wrapping_add(1);

                    let handle = PlayingHandle(Arc::new(manager.last_playing_sound_id));
                    manager.playing_sounds.push(PlayingSound {
                        note,
                        handle: handle.clone(),
                        sampler: Arc::new(ShardedLock::new(sampler)),
                    });
                    handle
                };

                callback.send(handle).unwrap_or_default();

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn release_completed_sounds(&mut self) {
        let mut manager = self.manager.write().expect("Error locking manager");
        manager
            .playing_sounds
            .retain(|s| s.still_producing_values() || Arc::strong_count(&s.handle.0) > 1)
    }
}

pub mod prelude {
    pub use super::{Device, Manager, ManagerHandle};
}
