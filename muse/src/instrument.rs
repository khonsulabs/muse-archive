use crate::{
    device::Device, envelope::PlayingState, manager::PlayingHandle, note::Note,
    sampler::PreparedSampler,
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct GeneratedTone<T> {
    pub source: T,
    pub control: Arc<RwLock<PlayingState>>,
}

pub type ControlHandles = Vec<Arc<RwLock<PlayingState>>>;

pub trait ToneGenerator: Sized {
    fn generate_tone(
        note: Note,
        controls: &mut ControlHandles,
    ) -> Result<PreparedSampler, anyhow::Error>;
}

pub struct VirtualInstrument<T> {
    playing_notes: Vec<PlayingNote>,
    device: Device,
    sustain: bool,
    _phantom: std::marker::PhantomData<T>,
}

pub struct PlayingNote {
    note: Note,
    handle: Option<PlayingHandle>,
    controls: Vec<Arc<RwLock<PlayingState>>>,
}

impl PlayingNote {
    fn is_playing(&self) -> bool {
        for control in self.controls.iter() {
            let value = control.read().unwrap();
            if let PlayingState::Playing = *value {
                return true;
            }
        }

        false
    }

    fn stop(&self) {
        for control in self.controls.iter() {
            let mut value = control.write().unwrap();
            *value = PlayingState::Stopping;
        }
    }

    fn sustain(&self) {
        for control in self.controls.iter() {
            let mut value = control.write().unwrap();
            *value = PlayingState::Sustaining;
        }
    }
}

impl Drop for PlayingNote {
    fn drop(&mut self) {
        self.stop();

        let handle = std::mem::take(&mut self.handle);
        let controls = std::mem::take(&mut self.controls);

        std::thread::spawn(move || loop {
            {
                let all_stopped = controls
                    .iter()
                    .map(|control| {
                        let value = control.read().unwrap();
                        *value
                    })
                    .all(|state| state == PlayingState::Stopped);
                if all_stopped {
                    println!("Sound stopping");
                    drop(handle);
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        });
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Loudness {
    Fortissimo,
    MezzoForte,
    Pianissimo,
}

impl<T> Default for VirtualInstrument<T>
where
    T: ToneGenerator,
{
    fn default() -> Self {
        let device = Device::default_output().expect("No default audio output device");
        Self::new(device)
    }
}

impl<T> VirtualInstrument<T>
where
    T: ToneGenerator,
{
    pub fn new(device: Device) -> Self {
        Self {
            device,
            playing_notes: Vec::new(),
            sustain: false,
            _phantom: std::marker::PhantomData::default(),
        }
    }

    pub fn play_note(&mut self, note: Note) -> Result<(), anyhow::Error> {
        // We need to re-tone the note, so we'll get rid of the existing notes
        self.playing_notes.retain(|n| n.note.step != note.step);

        let mut controls = ControlHandles::new();
        let source = T::generate_tone(note, &mut controls)?;
        let handle = Some(self.device.play(source)?);

        self.playing_notes.push(PlayingNote {
            note,
            handle,
            controls,
        });

        Ok(())
    }

    pub fn stop_note(&mut self, step: u8) {
        if self.sustain {
            // For sustain, we need ot keep the notes playing, but mark that the key isn't pressed
            // so that when the pedal is released, the note isn't filtered out.
            if let Some(existing_note) = self
                .playing_notes
                .iter_mut()
                .find(|pn| pn.note.step == step)
            {
                existing_note.sustain();
            }
        } else {
            self.playing_notes.retain(|pn| pn.note.step != step);
        }
    }

    pub fn set_sustain(&mut self, active: bool) {
        self.sustain = active;

        if !active {
            self.playing_notes.retain(|n| n.is_playing());
        }
    }
}
