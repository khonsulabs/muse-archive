use crossbeam::channel::{unbounded, Receiver, Sender};
use midir::{Ignore, MidiInput, MidiInputConnection};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

mod message;
pub use message::{ChannelMessage, Controller, Message};

static INPUT_MANAGER: Lazy<Arc<RwLock<Option<InputManager>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

pub fn open_input() -> Receiver<Message> {
    open_named_input("amuse")
}

pub fn open_named_input<S: ToString>(app_name: S) -> Receiver<Message> {
    let app_name = app_name.to_string();

    let mut manager = INPUT_MANAGER
        .write()
        .expect("Error locking global input manager");
    if let Some(manager) = manager.as_ref() {
        return manager.receiver.clone();
    }

    let (sender, receiver) = unbounded();

    *manager = Some(InputManager {
        receiver: receiver.clone(),
    });

    std::thread::spawn(move || {
        input_thread(app_name, sender).expect("Error processing MIDI input")
    });

    receiver
}

type InputConnections = HashMap<String, MidiInputConnection<Sender<Message>>>;

pub struct InputManager {
    receiver: Receiver<Message>,
}

fn input_thread(app_name: String, sender: Sender<Message>) -> Result<(), anyhow::Error> {
    let mut connected_ports = InputConnections::new();
    loop {
        // Find any ports to connect to
        let _ = connect_to_available_inputs(&app_name, &sender, &mut connected_ports);

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn connect_to_available_inputs(
    app_name: &str,
    sender: &Sender<Message>,
    open_ports: &mut InputConnections,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = MidiInput::new(app_name)?;
    input.ignore(Ignore::None);

    // Identify ports that can be opened
    // Keep track of ports that error when they are opened
    let port_names_to_try = {
        input
            .ports()
            .into_iter()
            .filter_map(|p| input.port_name(&p).ok().map(|name| name))
            .filter(|name| !open_ports.contains_key(name))
            .collect::<Vec<_>>()
    };

    for port_name in port_names_to_try {
        connect_to_input(app_name, port_name, &sender, open_ports)?;
    }

    Ok(())
}

fn connect_to_input(
    app_name: &str,
    port_name: String,
    sender: &Sender<Message>,
    open_ports: &mut InputConnections,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = MidiInput::new(app_name)?;
    input.ignore(Ignore::None);
    if let Some(port) = input
        .ports()
        .into_iter()
        .find(|p| input.port_name(&p).unwrap_or_default() == port_name)
    {
        let connection = input.connect(
            &port,
            &app_name,
            move |_, message, sender| handle_message(message, sender),
            sender.clone(),
        )?;

        open_ports.insert(port_name, connection);
    }

    Ok(())
}

fn handle_message(message: &[u8], sender: &Sender<Message>) {
    if !message.is_empty() {
        let message = Message::from(message);
        sender.send(message).unwrap();
    }
}
