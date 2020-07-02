use crate::{
    envelope::{EnvelopeBuilder, EnvelopeConfiguration, EnvelopeCurve},
    instrument::{
        serialization::{self, OscillatorFunction},
        ControlHandles,
    },
    note::Note,
    parameter,
    sampler::{
        add::Add,
        amplify::Amplify,
        multiply::Multiply,
        oscillator::{Oscillator, Sawtooth, Sine, Square, Triangle},
        pan::Pan,
        PreparableSampler, PreparedSampler,
    },
};
use std::{collections::HashMap, convert::TryFrom};

pub struct LoadedInstrument<T = ()> {
    output: Node<T>,
}

impl<T> Instantiatable for LoadedInstrument<T>
where
    T: Instantiatable,
{
    fn instantiate(&self, note: &Note, control_handles: &mut ControlHandles) -> PreparedSampler {
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
    T: serialization::node_instantiator::NodeInstantiator<T>,
{
    type Error = serialization::Error;
    fn try_from(
        instrument_spec: serialization::Instrument<T>,
    ) -> Result<LoadedInstrument<T>, Self::Error> {
        use serialization::node_instantiator::NodeInstantiator;

        let envelopes = Self::instantiate_envelopes(&instrument_spec.envelopes)?;

        let mut context = serialization::node_instantiator::Context::new(&envelopes);

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

pub trait Instantiatable {
    fn instantiate(&self, note: &Note, controls: &mut ControlHandles) -> PreparedSampler;
}

impl Instantiatable for () {
    fn instantiate(&self, _note: &Note, _controls: &mut ControlHandles) -> PreparedSampler {
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
    T: Instantiatable,
{
    fn instantiate(&self, note: &Note, controls: &mut ControlHandles) -> PreparedSampler {
        match self {
            Node::Oscillator {
                frequency,
                function,
                amplitude,
            } => {
                let frequency = frequency.instantiate(note, controls);
                let amplitude = amplitude.instantiate(note, controls);

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
                value.instantiate(note, controls),
                input.instantiate(note, controls),
            )
            .prepare(),
            Node::Pan { value, input } => Pan::new(
                value.instantiate(note, controls),
                input.instantiate(note, controls),
            )
            .prepare(),
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
    pub fn instantiate(&self, note: &Note, controls: &mut ControlHandles) -> parameter::Parameter {
        match self {
            Parameter::NoteHertz => parameter::Parameter::Value(note.hertz()),
            Parameter::NoteStep => parameter::Parameter::Value(note.step as f32),
            Parameter::NoteVelocity => parameter::Parameter::Value(note.velocity()),
            Parameter::Envelope(config) => config.as_parameter(controls),
            Parameter::Value(value) => parameter::Parameter::Value(*value),
        }
    }
}
