use crossbeam::channel::Receiver;
use iced_native::{futures, subscription::Recipe};
use midir::{Ignore, MidiInput, MidiInputConnection};

pub enum MIDIControl {
    RefreshInputList,
    SetInput(String),
}

pub struct MIDI {
    control: Receiver<MIDIControl>,
    midi_in: MidiInput,
}

impl MIDI {
    pub fn new(control: Receiver<MIDIControl>) -> Result<Self, midir::InitError> {
        let mut midi = Self {
            control,
            midi_in: MidiInput::new("amuse")?,
        };
        midi.midi_in.ignore(Ignore::None);
        Ok(midi)
    }

    pub fn set_current_input(&mut self, index: usize) {
        if let Some(Some(name)) = &self
            .available_inputs
            .as_ref()
            .map(|names| names.get(index).map(|n| n.to_owned()))
        {
            if let Some(port) = self
                .midi_in
                .ports()
                .iter()
                .find(|p| &self.midi_in.port_name(p).unwrap_or_default() == name)
            {
                if let Some(existing_input) = self.current_input.as_ref() {
                    existing_input.close();
                }
                self.current_input = Some(
                    self.midi_in
                        .connect(port, name, move |a, b, _| {}, ())
                        .unwrap(),
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MIDIMessage {
    InputsListed(Vec<String>),
}

pub enum State {
    New,
    Initialized,
    Connected(MidiInputConnection<()>),
}

impl<H, I> Recipe<H, I> for MIDI
where
    H: std::hash::Hasher,
{
    type Output = MIDIMessage;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(State::New, |state| async move {
            // TODO Read control events...
            match state {
                State::New => {
                    let inputs = self
                        .midi_in
                        .ports()
                        .iter()
                        .filter_map(|p| self.midi_in.port_name(p).ok())
                        .collect();
                    (MIDIMessage::InputsListed(inputs), State::Initialized)
                }
                State::Initialized => {
                    
                }
                State::Connected(connection) => {}
            }
        }))
    }
}
