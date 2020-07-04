use crate::{
    envelope::EnvelopeConfiguration,
    instrument::serialization::{self, Error, Node},
    node,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context<'a, T> {
    envelopes: &'a HashMap<String, EnvelopeConfiguration>,
    nodes: HashMap<String, node::Node<T>>,
}

impl<'a, T> Context<'a, T> {
    pub(crate) fn new(envelopes: &'a HashMap<String, EnvelopeConfiguration>) -> Self {
        Self {
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

    pub fn node_reference(&mut self, name: &str) -> Result<node::Node<T>, Error> {
        if let Some(sampler) = self.nodes.remove(name) {
            Ok(sampler)
        } else {
            Err(Error::NodeNotFound(name.to_owned()))
        }
    }

    pub fn node_references(&mut self, names: &[String]) -> Result<Vec<node::Node<T>>, Error> {
        names
            .iter()
            .map(|i| self.node_reference(i))
            .collect::<Result<Vec<_>, _>>()
    }

    pub(crate) fn node_instantiated(&mut self, name: &str, sampler: node::Node<T>) {
        self.nodes.insert(name.to_owned(), sampler);
    }

    pub fn load_parameter(
        &mut self,
        parameter: &serialization::Parameter,
    ) -> Result<node::Parameter, Error> {
        let parameter = match parameter {
            serialization::Parameter::Envelope(envelope) => {
                node::Parameter::Envelope(self.envelope(envelope)?)
            }

            serialization::Parameter::Value(value) => node::Parameter::Value(*value),
            serialization::Parameter::NoteHertz => node::Parameter::NoteHertz,
            serialization::Parameter::NoteStep => node::Parameter::NoteStep,
            serialization::Parameter::NoteVelocity => node::Parameter::NoteVelocity,
        };

        Ok(parameter)
    }
}

pub trait NodeInstantiator<T> {
    fn instantiate_node(
        &self,
        context: &mut Context<'_, T>,
    ) -> Result<node::Node<T>, anyhow::Error>;
}

impl<T> NodeInstantiator<T> for () {
    fn instantiate_node(
        &self,
        _context: &mut Context<'_, T>,
    ) -> Result<node::Node<T>, anyhow::Error> {
        unreachable!("muse should never reach this code if you have () as the type on Node<>")
    }
}

impl<T> NodeInstantiator<T> for Node<T>
where
    T: NodeInstantiator<T>,
{
    fn instantiate_node(
        &self,
        context: &mut Context<'_, T>,
    ) -> Result<node::Node<T>, anyhow::Error> {
        match self {
            Node::Oscillator {
                function,
                frequency,
                amplitude,
            } => Ok(node::Node::Oscillator {
                function: *function,
                frequency: context.load_parameter(frequency)?,
                amplitude: context.load_parameter(amplitude)?,
            }),
            Node::Multiply { inputs } => Ok(node::Node::Multiply {
                inputs: context.node_references(inputs)?,
            }),
            Node::Amplify { value, input } => Ok(node::Node::Amplify {
                value: context.load_parameter(value)?,
                input: Box::new(context.node_reference(input)?),
            }),
            Node::Add { inputs } => Ok(node::Node::Add {
                inputs: context.node_references(inputs)?,
            }),
            Node::Pan { value, input } => Ok(node::Node::Pan {
                value: context.load_parameter(value)?,
                input: Box::new(context.node_reference(input)?),
            }),
            Node::Unison {
                quantity,
                detune,
                input,
            } => Ok(node::Node::Unison {
                quantity: *quantity,
                detune: context.load_parameter(detune)?,
                template: Box::new(context.node_reference(input)?),
            }),
            Node::Custom(custom) => custom.instantiate_node(context),
        }
    }
}
