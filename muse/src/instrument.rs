use crate::{envelope::PlayingState, note::Note};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct GeneratedTone<T> {
    pub source: T,
    pub control: Arc<RwLock<PlayingState>>,
}

pub trait ToneGenerator {
    type Source: rodio::Source<Item = f32> + Send + Sync + 'static;

    fn generate_tone(note: Note) -> Result<GeneratedTone<Self::Source>, anyhow::Error>;
}

pub struct VirtualInstrument<T> {
    playing_notes: Vec<PlayingNote>,
    device: rodio::Device,
    sustain: bool,
    _phantom: std::marker::PhantomData<T>,
}

pub struct PlayingNote {
    note: Note,
    sink: Option<rodio::Sink>,
    control: Arc<RwLock<PlayingState>>,
}

impl PlayingNote {
    fn is_playing(&self) -> bool {
        let value = self.control.read().unwrap();
        if let PlayingState::Playing = *value {
            true
        } else {
            false
        }
    }

    fn stop(&self) {
        let mut value = self.control.write().unwrap();
        *value = PlayingState::Stopping;
    }

    fn sustain(&self) {
        let mut value = self.control.write().unwrap();
        *value = PlayingState::Sustaining;
    }
}

impl Drop for PlayingNote {
    fn drop(&mut self) {
        self.stop();

        let sink = std::mem::replace(&mut self.sink, None);
        let is_playing = self.control.clone();

        std::thread::spawn(move || loop {
            {
                let value = is_playing.read().unwrap();
                if let PlayingState::Stopped = *value {
                    sink.unwrap().stop();
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(10))
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
        let device = rodio::default_output_device().expect("No default audio output device");
        Self::new(device)
    }
}

impl<T> VirtualInstrument<T>
where
    T: ToneGenerator,
{
    pub fn new(device: rodio::Device) -> Self {
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

        let GeneratedTone { source, control } = T::generate_tone(note)?;
        let sink = rodio::Sink::new(&self.device);
        sink.append(source);
        self.playing_notes.push(PlayingNote {
            note,
            sink: Some(sink),
            control,
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
