use muse::{
    instrument::{
        serialization, InstrumentController, LoadedInstrument, ToneGenerator, VirtualInstrument,
    },
    note::Note,
    sampler::PreparedSampler,
};

use std::{
    convert::TryInto,
    error::Error,
    io::{stdin, stdout, Write},
};

use midir::{Ignore, MidiInput};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Start,
    TimingClock,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,

    Vendor {
        vendor_id: u8,
        payload: Vec<u8>,
    },

    Channel {
        channel: u8,
        message: ChannelMessage,
    },

    Unsupported(Vec<u8>),
}

impl From<&[u8]> for Message {
    fn from(bytes: &[u8]) -> Self {
        if bytes.is_empty() || bytes[0] < 0x80 {
            return Message::Unsupported(bytes.into());
        }

        if bytes[0] < 0xF0 {
            // Channel message
            let status = bytes[0] & 0xF0;
            let channel = bytes[0] & 0x0F;
            Message::Channel {
                channel,
                message: match status {
                    0x80 => ChannelMessage::NoteOff {
                        key: bytes[1],
                        velocity: bytes[2],
                    },
                    0x90 => ChannelMessage::NoteOn {
                        key: bytes[1],
                        velocity: bytes[2],
                    },
                    0xA0 => ChannelMessage::PolyphonicKeyPressure {
                        key: bytes[1],
                        pressure: bytes[2],
                    },
                    0xB0 => ChannelMessage::ControlChange {
                        controller: bytes[1].into(),
                        value: bytes[2],
                    },
                    0xC0 => ChannelMessage::ProgramChange { program: bytes[1] },
                    0xD0 => ChannelMessage::ChannelPressure { pressure: bytes[1] },
                    0xE0 => ChannelMessage::PitchBend {
                        // Byte 1 is the least significant byte (7 bits though)
                        // Byte 2 is the most significant (also 7 bits)
                        // Value of 8192 (64 << 7) is no bend
                        amount: ((bytes[2] as i16) << 7 | bytes[1] as i16) - 8192,
                    },
                    _ => unreachable!(),
                },
            }
        } else {
            match bytes[0] {
                0xF0 => Message::Vendor {
                    vendor_id: bytes[1],
                    payload: bytes.iter().skip(2).cloned().collect(),
                },
                _ => Message::Unsupported(bytes.into()),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMessage {
    AllSoundOff,
    ResetAllControllers,
    LocalControlOff,
    LocalControlOn,
    AllNotesOff,
    NoteOff { key: u8, velocity: u8 },
    NoteOn { key: u8, velocity: u8 },
    ProgramChange { program: u8 },
    ControlChange { controller: Controller, value: u8 },
    PolyphonicKeyPressure { key: u8, pressure: u8 },
    ChannelPressure { pressure: u8 },
    PitchBend { amount: i16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Controller {
    Undefined(u8),
    ModulationWheel,
    BreathController,
    FootController,
    PortamentoTime,
    DataEntrySlider,
    MainVolume,
    Balance,
    Pan,
    ExpressionController,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    Damper,
    Portamento,
    Sostenuto,
    SoftPedal,
}

impl From<u8> for Controller {
    fn from(value: u8) -> Self {
        match value {
            1 => Controller::ModulationWheel,
            2 => Controller::BreathController,
            4 => Controller::FootController,
            5 => Controller::PortamentoTime,
            6 => Controller::DataEntrySlider,
            7 => Controller::MainVolume,
            8 => Controller::Balance,
            10 => Controller::Pan,
            11 => Controller::ExpressionController,
            16 => Controller::GeneralPurpose1,
            17 => Controller::GeneralPurpose2,
            18 => Controller::GeneralPurpose3,
            19 => Controller::GeneralPurpose4,
            64 => Controller::Damper,
            65 => Controller::Portamento,
            66 => Controller::Sostenuto,
            67 => Controller::SoftPedal,
            undefined => Controller::Undefined(undefined),
        }
    }
}

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

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut instrument = VirtualInstrument::new_with_default_output(TestInstrument {
        basic_synth: ron::from_str::<serialization::Instrument>(include_str!(
            "support/basic_synth.ron"
        ))?
        .try_into()?,
    })?;

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, message, _| {
            if !message.is_empty() {
                let message = Message::from(message);
                if let Message::Channel { channel, message } = &message {
                    if channel == &0 {
                        match message {
                            ChannelMessage::NoteOff { key, .. } => instrument.stop_note(*key),
                            ChannelMessage::NoteOn { key, velocity } => {
                                instrument.play_note(Note::new(*key, *velocity)).unwrap()
                            }
                            ChannelMessage::ControlChange { controller, value } => match controller
                            {
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
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
