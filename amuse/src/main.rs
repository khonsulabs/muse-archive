use muse::instrument;
use std::error::Error;
use std::io::{stdin, stdout, Write};

use midir::{Ignore, MidiInput};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

const STATUS_MESSAGE_NOTE_OFF: u8 = 0x80;
const STATUS_MESSAGE_NOTE_ON: u8 = 0x90;
const STATUS_MESSAGE_CONTROL: u8 = 0xb0;
const CONTROLLER_NUMBER_SUSTAIN: u8 = 0x40;

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    let mut instrument = instrument::VirtualInstrument::default();

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
        move |stamp, message, _| {
            if !message.is_empty() {
                println!("{}: {:x?} (len = {})", stamp, message, message.len());
                match message[0] & 0xF0 {
                    STATUS_MESSAGE_NOTE_ON => instrument.play_note(message[1], message[2]).unwrap(),
                    STATUS_MESSAGE_NOTE_OFF => instrument.stop_note(message[1]),
                    STATUS_MESSAGE_CONTROL => match message[1] {
                        CONTROLLER_NUMBER_SUSTAIN => instrument.set_sustain(message[2] > 0x40),
                        _ => {}
                    },
                    _ => {}
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
