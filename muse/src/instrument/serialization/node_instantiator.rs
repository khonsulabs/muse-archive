use crate::{
    envelope::EnvelopeConfiguration,
    instrument::{
        serialization::{Error, Node, OscillatorFunction},
        ControlHandles,
    },
    note::Note,
    parameter::Parameter,
    sampler::{
        amplify::Amplify,
        multiply::Multiply,
        oscillator::{Oscillator, Sawtooth, Sine, Square, Triangle},
        PreparableSampler, PreparedSampler,
    },
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context<'a> {
    pub note: &'a Note,
    pub controls: &'a mut ControlHandles,
    envelopes: &'a HashMap<String, EnvelopeConfiguration>,
    nodes: HashMap<String, PreparedSampler>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        note: &'a Note,
        controls: &'a mut ControlHandles,
        envelopes: &'a HashMap<String, EnvelopeConfiguration>,
    ) -> Self {
        Self {
            note,
            controls,
            envelopes,
            nodes: HashMap::new(),
        }
    }

    pub fn envelope(&self, name: &str) -> Result<EnvelopeConfiguration, Error> {
        self.envelopes
            .get(name)
            .cloned()
            .ok_or_else(|| Error::NodeNotFound(name.to_owned()))
    }

    pub fn node_reference(&mut self, name: &str) -> Result<PreparedSampler, Error> {
        if let Some(sampler) = self.nodes.remove(name) {
            Ok(sampler)
        } else {
            Err(Error::NodeNotFound(name.to_owned()))
        }
    }

    pub(crate) fn node_instantiated(&mut self, name: &str, sampler: PreparedSampler) {
        self.nodes.insert(name.to_owned(), sampler);
    }
}

pub trait NodeInstantiator {
    fn instantiate_node(&self, context: &mut Context<'_>)
        -> Result<PreparedSampler, anyhow::Error>;
}

impl NodeInstantiator for () {
    fn instantiate_node(
        &self,
        _context: &mut Context<'_>,
    ) -> Result<PreparedSampler, anyhow::Error> {
        unreachable!("muse should never reach this code if you have () as the type on Node<>")
    }
}

impl<T> NodeInstantiator for Node<T>
where
    T: NodeInstantiator,
{
    #[inline]
    fn instantiate_node(
        &self,
        context: &mut Context<'_>,
    ) -> Result<PreparedSampler, anyhow::Error> {
        match self {
            Node::Oscillator {
                function,
                frequency,
                amplitude,
            } => {
                let frequency = Parameter::from_serialization(frequency, context)?;
                let amplitude = Parameter::from_serialization(amplitude, context)?;

                let sampler = match function {
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
                };

                Ok(sampler)
            }
            Node::Multiply { inputs } => Ok(Multiply::new(
                inputs
                    .iter()
                    .map(|i| context.node_reference(i))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .prepare()),
            Node::Amplify { value, input } => Ok(Amplify::new(
                Parameter::from_serialization(value, context)?,
                context.node_reference(input)?,
            )
            .prepare()),
            Node::Custom(custom) => custom.instantiate_node(context),
        }
    }
}
