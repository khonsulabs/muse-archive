use crate::{
    envelope::{EnvelopeBuilder, EnvelopeConfiguration, EnvelopeCurve},
    instrument::{
        serialization::{self, OscillatorFunction},
        ControlHandles,
    },
    note::Note,
    parameter,
    sampler::{
        Add, Amplify, Multiply, Oscillator, Pan, PreparableSampler, PreparedSampler, Sawtooth,
        Sine, Square, Triangle, Unison,
    },
};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug)]
pub struct LoadedInstrument<T = ()> {
    output: Node<T>,
}

impl<T> Instantiatable for LoadedInstrument<T>
where
    T: Instantiatable + Clone + std::fmt::Debug + 'static,
{
    fn instantiate(&self, note: &Note, control_handles: &ControlHandles) -> PreparedSampler {
        self.output.instantiate(note, control_handles)
    }
}

#[cfg(feature = "serialization")]
impl<T> LoadedInstrument<T> {
    fn instantiate_envelopes(
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

#[cfg(feature = "serialization")]
impl<T> TryFrom<serialization::Instrument<T>> for LoadedInstrument<T>
where
    T: serialization::NodeInstantiator<T>,
{
    type Error = serialization::Error;
    fn try_from(
        instrument_spec: serialization::Instrument<T>,
    ) -> Result<LoadedInstrument<T>, Self::Error> {
        use serialization::NodeInstantiator;

        let envelopes = Self::instantiate_envelopes(&instrument_spec.envelopes)?;

        let mut context = serialization::Context::new(&envelopes);

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
        // https://github.com/khonsulabs/muse/issues/17
        Ok(LoadedInstrument { output })
    }
}

pub trait Instantiatable: Send + Sync + std::fmt::Debug {
    fn instantiate(&self, note: &Note, controls: &ControlHandles) -> PreparedSampler;
}

impl Instantiatable for () {
    fn instantiate(&self, _note: &Note, _controls: &ControlHandles) -> PreparedSampler {
        unreachable!()
    }
}

#[derive(Debug, Clone)]
pub enum Node<T> {
    Oscillator {
        function: OscillatorFunction,
        frequency: Parameter,
        amplitude: Parameter,
    },
    Unison {
        template: Box<Self>,
        quantity: u8,
        detune: Parameter,
    },
    Amplify {
        value: Parameter,
        input: Box<Self>,
    },
    Multiply {
        inputs: Vec<Self>,
    },
    Add {
        inputs: Vec<Self>,
    },
    Pan {
        value: Parameter,
        input: Box<Self>,
    },
    Custom(T),
}

impl<T> Instantiatable for Node<T>
where
    T: Instantiatable + Clone + 'static,
{
    fn instantiate(&self, note: &Note, controls: &ControlHandles) -> PreparedSampler {
        match self {
            Node::Oscillator {
                frequency,
                function,
                amplitude,
            } => {
                let frequency = frequency.instantiate(controls);
                let amplitude = amplitude.instantiate(controls);

                match function {
                    OscillatorFunction::Sine => {
                        Oscillator::<Sine>::new(frequency, amplitude).prepare()
                    }
                    OscillatorFunction::Sawtooth => {
                        Oscillator::<Sawtooth>::new(frequency, amplitude).prepare()
                    }
                    OscillatorFunction::Square => {
                        Oscillator::<Square>::new(frequency, amplitude).prepare()
                    }
                    OscillatorFunction::Triangle => {
                        Oscillator::<Triangle>::new(frequency, amplitude).prepare()
                    }
                }
            }
            Node::Multiply { inputs } => Multiply::new(
                inputs
                    .iter()
                    .map(|i| i.instantiate(note, controls))
                    .collect(),
            )
            .prepare(),
            Node::Add { inputs } => Add::new(
                inputs
                    .iter()
                    .map(|i| i.instantiate(note, controls))
                    .collect(),
            )
            .prepare(),
            Node::Amplify { value, input } => Amplify::new(
                value.instantiate(controls),
                input.instantiate(note, controls),
            )
            .prepare(),
            Node::Pan { value, input } => Pan::new(
                value.instantiate(controls),
                input.instantiate(note, controls),
            )
            .prepare(),
            Node::Unison {
                template,
                detune,
                quantity,
            } => {
                let samplers = (0..*quantity)
                    .map(|_| template.instantiate(note, controls))
                    .collect();
                Unison::new(detune.instantiate(controls), samplers).prepare()
            }
            Node::Custom(custom) => custom.instantiate(note, controls),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Parameter {
    Value(f32),
    NoteHertz,
    NoteStep,
    NoteVelocity,
    Envelope(EnvelopeConfiguration),
}

impl Parameter {
    pub fn instantiate(&self, controls: &ControlHandles) -> parameter::Parameter {
        match self {
            Parameter::NoteHertz => parameter::Parameter::NoteHertz,
            Parameter::NoteStep => parameter::Parameter::NoteStep,
            Parameter::NoteVelocity => parameter::Parameter::NoteVelocity,
            Parameter::Envelope(config) => config.as_parameter(controls),
            Parameter::Value(value) => parameter::Parameter::Value(*value),
        }
    }
}
