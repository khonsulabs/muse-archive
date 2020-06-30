use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instrument<T = ()> {
    pub name: String,
    pub envelopes: HashMap<String, Envelope>,
    pub nodes: HashMap<String, Node<T>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Envelope {
    pub attack: Option<Parameter>,
    pub hold: Option<Parameter>,
    pub decay: Option<Parameter>,
    pub sustain: Option<Parameter>,
    pub release: Option<Parameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Parameter {
    Milliseconds(u32),
    Percent(f32),
    NoteVelocity,
    NoteHertz,
    NoteStep,
    Envelope(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Node<T> {
    Oscillator(Oscillator),
    Amplify(Amplify),
    Add(Add),
    Custom(T),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Oscillator {
    pub function: OscillatorFunction,
    pub frequency: Parameter,
    pub amplitude: Parameter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OscillatorFunction {
    Sawtooth,
    Sine,
    Square,
    Triangle,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Amplify {
    pub value: Parameter,
    pub input: Parameter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Add {
    pub inputs: Vec<String>,
}
