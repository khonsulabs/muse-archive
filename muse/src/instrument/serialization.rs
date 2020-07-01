use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod node_instantiator;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("envelope not found {0}")]
    EnvelopeNotFound(String),
    #[error("node not found {0}")]
    NodeNotFound(String),
    #[error("reference cycle found between these nodes: {0:?}")]
    RecursiveNodeDependencies(Vec<String>),
    #[error("error with envelope curve: {0}")]
    EnvelopeCurveError(#[from] crate::envelope::EnvelopeCurveError),
    #[error("error loading node {0:?}")]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instrument<T = ()> {
    pub name: String,
    pub envelopes: HashMap<String, Envelope>,
    pub nodes: HashMap<String, Node<T>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Envelope {
    pub attack: Option<EnvelopeCurve>,
    pub hold: Option<EnvelopeCurve>,
    pub decay: Option<EnvelopeCurve>,
    pub sustain: Option<EnvelopeCurve>,
    pub release: Option<EnvelopeCurve>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EnvelopeCurve {
    Milliseconds(u32),
    Sustain(f32),
    // TODO add bezier curves
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Parameter {
    Value(f32),
    NoteVelocity,
    NoteHertz,
    NoteStep,
    Envelope(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Node<T> {
    Oscillator {
        function: OscillatorFunction,
        frequency: Parameter,
        amplitude: Parameter,
    },
    Amplify {
        value: Parameter,
        input: String,
    },
    Multiply {
        inputs: Vec<String>,
    },
    Add {
        inputs: Vec<String>,
    },
    Pan {
        value: Parameter,
        input: String,
    },
    Custom(T),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OscillatorFunction {
    Sawtooth,
    Sine,
    Square,
    Triangle,
}
