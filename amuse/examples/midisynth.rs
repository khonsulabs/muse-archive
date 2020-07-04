use amuse::midi::{ChannelMessage, Controller, Message};
use muse::{
    instrument::{serialization, InstrumentController, ToneGenerator, VirtualInstrument},
    node::LoadedInstrument,
    sampler::PreparedSampler,
    Note,
};

use std::{convert::TryInto, error::Error};

pub struct TestInstrument {
    basic_synth: LoadedInstrument,
}

impl ToneGenerator for TestInstrument {
    type CustomNodes = ();

    fn generate_tone(
        &mut self,
        note: Note,
        control: &mut InstrumentController<Self>,
    ) -> Result<PreparedSampler, anyhow::Error> {
        Ok(control.instantiate(&self.basic_synth, note)?)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut instrument = VirtualInstrument::new_with_default_output(TestInstrument {
        basic_synth: ron::from_str::<serialization::Instrument>(include_str!(
            "support/basic_synth.ron"
        ))?
        .try_into()?,
    })?;

    let messages = amuse::midi::open_named_input("midisynth");

    while let Ok(message) = messages.recv() {
        if let Message::Channel { channel, message } = &message {
            if channel == &0 {
                match message {
                    ChannelMessage::NoteOff { key, .. } => instrument.stop_note(*key),
                    ChannelMessage::NoteOn { key, velocity } => instrument
                        .play_note(Note::new(*key as f32, *velocity))
                        .unwrap(),
                    ChannelMessage::ControlChange { controller, value } => match controller {
                        Controller::Damper => instrument.set_sustain(value > &0x40),
                        _ => println!(
                            "Unrecognized controller changed {:?}, value {}",
                            controller, value
                        ),
                    },
                    unhandled => println!("Unhandled channel message: {:?}", unhandled),
                }
            } else {
                println!(
                    "Ignoring inactive channel ({}) message {:?}",
                    channel, message
                );
            }
        } else {
            println!("Unhandled message {:?}", message)
        }
    }

    Ok(())
}
