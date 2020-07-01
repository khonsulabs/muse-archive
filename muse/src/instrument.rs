use crate::{
    envelope::{EnvelopeBuilder, EnvelopeConfiguration, EnvelopeCurve, PlayingState},
    manager::{Device, PlayingHandle},
    note::Note,
    sampler::PreparedSampler,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

#[cfg(feature = "serialization")]
pub mod serialization;

pub struct GeneratedTone<T> {
    pub source: T,
    pub control: Arc<RwLock<PlayingState>>,
}

pub type ControlHandles = Vec<Arc<RwLock<PlayingState>>>;

#[derive(Debug)]
pub struct InstrumentController<T> {
    pub control_handles: ControlHandles,
    _tone_generator: std::marker::PhantomData<T>,
}

impl<T> Default for InstrumentController<T> {
    fn default() -> Self {
        Self {
            control_handles: ControlHandles::new(),
            _tone_generator: std::marker::PhantomData::default(),
        }
    }
}

impl<T> InstrumentController<T> {
    #[cfg(feature = "serialization")]
    pub fn instantiate(
        &mut self,
        instrument_spec: &serialization::Instrument,
        note: Note,
    ) -> Result<PreparedSampler, serialization::Error> {
        use serialization::node_instantiator::NodeInstantiator;
        // TODO Creating envelopes should happen once, not over and over.
        let envelopes = self.instantiate_envelopes(&instrument_spec.envelopes)?;
        let mut context = serialization::node_instantiator::Context::new(
            &note,
            &mut self.control_handles,
            &envelopes,
        );

        let mut nodes_to_load = instrument_spec.nodes.iter().collect::<Vec<_>>();
        while !nodes_to_load.is_empty() {
            let initial_len = nodes_to_load.len();
            nodes_to_load.retain(|(name, node)| {
                if let Ok(sampler) = node.instantiate_node(&mut context) {
                    context.node_instantiated(name, sampler);
                    // Handled, so we return false to free it
                    false
                } else {
                    true
                }
            });

            if initial_len == nodes_to_load.len() {
                return Err(serialization::Error::RecursiveNodeDependencies(
                    nodes_to_load.iter().map(|n| n.0.clone()).collect(),
                ));
            }
        }

        let output = context.node_reference("output")?;
        // TODO Do we do anything to warn against unused nodes?
        Ok(output)
    }

    fn instantiate_envelopes(
        &mut self,
        incoming: &HashMap<String, serialization::Envelope>,
    ) -> Result<HashMap<String, EnvelopeConfiguration>, serialization::Error> {
        incoming
            .iter()
            .map(|(name, env)| {
                Ok((
                    name.to_owned(),
                    EnvelopeBuilder {
                        attack: EnvelopeCurve::from_serialization(&env.attack)?,
                        hold: EnvelopeCurve::from_serialization(&env.hold)?,
                        decay: EnvelopeCurve::from_serialization(&env.decay)?,
                        sustain: EnvelopeCurve::from_serialization(&env.sustain)?,
                        release: EnvelopeCurve::from_serialization(&env.release)?,
                    }
                    .build()?,
                ))
            })
            .collect::<Result<_, serialization::Error>>()
    }
}

pub trait ToneGenerator: Sized {
    type CustomNodes;

    fn generate_tone(
        &mut self,
        note: Note,
        control: &mut InstrumentController<Self>,
    ) -> Result<PreparedSampler, anyhow::Error>;
}

pub struct PlayingNote<T> {
    note: Note,
    handle: Option<PlayingHandle>,
    controller: InstrumentController<T>,
}

impl<T> PlayingNote<T> {
    fn is_playing(&self) -> bool {
        for control in self.controller.control_handles.iter() {
            let value = control.read().unwrap();
            if let PlayingState::Playing = *value {
                return true;
            }
        }

        false
    }

    fn stop(&self) {
        for control in self.controller.control_handles.iter() {
            let mut value = control.write().unwrap();
            *value = PlayingState::Stopping;
        }
    }

    fn sustain(&self) {
        for control in self.controller.control_handles.iter() {
            let mut value = control.write().unwrap();
            *value = PlayingState::Sustaining;
        }
    }
}

impl<T> Drop for PlayingNote<T> {
    fn drop(&mut self) {
        self.stop();

        let handle = std::mem::take(&mut self.handle);
        let control_handles = std::mem::take(&mut self.controller.control_handles);

        std::thread::spawn(move || loop {
            {
                let all_stopped = control_handles
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

pub struct VirtualInstrument<T> {
    playing_notes: Vec<PlayingNote<T>>,
    device: Device,
    sustain: bool,
    tone_generator: T,
}

impl<T> VirtualInstrument<T>
where
    T: ToneGenerator,
{
    pub fn new(device: Device, tone_generator: T) -> Self {
        Self {
            device,
            tone_generator,
            playing_notes: Vec::new(),
            sustain: false,
        }
    }

    pub fn new_with_default_output(tone_generator: T) -> Result<Self, anyhow::Error> {
        let device = Device::default_output()?;
        Ok(Self::new(device, tone_generator))
    }

    pub fn play_note(&mut self, note: Note) -> Result<(), anyhow::Error> {
        // We need to re-tone the note, so we'll get rid of the existing notes
        self.playing_notes.retain(|n| n.note.step != note.step);

        let mut controller = InstrumentController::default();
        let source = self.tone_generator.generate_tone(note, &mut controller)?;
        let handle = Some(self.device.play(source)?);

        self.playing_notes.push(PlayingNote {
            note,
            handle,
            controller,
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
