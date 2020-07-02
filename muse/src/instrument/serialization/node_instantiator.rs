use crate::{
    envelope::EnvelopeConfiguration,
    instrument::{
        loaded,
        serialization::{self, Error, Node},
    },
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context<'a, T> {
    envelopes: &'a HashMap<String, EnvelopeConfiguration>,
    nodes: HashMap<String, loaded::Node<T>>,
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

    pub fn node_reference(&mut self, name: &str) -> Result<loaded::Node<T>, Error> {
        if let Some(sampler) = self.nodes.remove(name) {
            Ok(sampler)
        } else {
            Err(Error::NodeNotFound(name.to_owned()))
        }
    }

    pub fn node_references(&mut self, names: &[String]) -> Result<Vec<loaded::Node<T>>, Error> {
        names
            .iter()
            .map(|i| self.node_reference(i))
            .collect::<Result<Vec<_>, _>>()
    }

    pub(crate) fn node_instantiated(&mut self, name: &str, sampler: loaded::Node<T>) {
        self.nodes.insert(name.to_owned(), sampler);
    }

    pub fn load_parameter(
        &mut self,
        parameter: &serialization::Parameter,
    ) -> Result<loaded::Parameter, Error> {
        let parameter = match parameter {
            serialization::Parameter::Envelope(envelope) => {
                loaded::Parameter::Envelope(self.envelope(envelope)?)
            }

            serialization::Parameter::Value(value) => loaded::Parameter::Value(*value),
            serialization::Parameter::NoteHertz => loaded::Parameter::NoteHertz,
            serialization::Parameter::NoteStep => loaded::Parameter::NoteStep,
            serialization::Parameter::NoteVelocity => loaded::Parameter::NoteVelocity,
        };

        Ok(parameter)
    }
}

pub trait NodeInstantiator<T> {
    fn instantiate_node(
        &self,
        context: &mut Context<'_, T>,
    ) -> Result<loaded::Node<T>, anyhow::Error>;
}

impl<T> NodeInstantiator<T> for () {
    fn instantiate_node(
        &self,
        _context: &mut Context<'_, T>,
    ) -> Result<loaded::Node<T>, anyhow::Error> {
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
    ) -> Result<loaded::Node<T>, anyhow::Error> {
        match self {
            Node::Oscillator {
                function,
                frequency,
                amplitude,
            } => Ok(loaded::Node::Oscillator {
                function: *function,
                frequency: context.load_parameter(frequency)?,
                amplitude: context.load_parameter(amplitude)?,
            }),
            Node::Multiply { inputs } => {
                Ok(loaded::Node::Multiply {
                    inputs: context.node_references(inputs)?,
                })
                //Ok(Multiply::new(context.node_references(inputs)?).prepare())
            }
            Node::Amplify { value, input } => Ok(loaded::Node::Amplify {
                value: context.load_parameter(value)?,
                input: Box::new(context.node_reference(input)?),
            }),
            // Ok(Amplify::new(
            //     Parameter::from_serialization(value, context)?,
            //     context.node_reference(input)?,
            // )
            // .prepare()),
            Node::Add { inputs } => Ok(loaded::Node::Add {
                inputs: context.node_references(inputs)?,
            }),
            Node::Pan { value, input } => Ok(loaded::Node::Pan {
                value: context.load_parameter(value)?,
                input: Box::new(context.node_reference(input)?),
            }),
            // Ok(Pan::new(
            //     Parameter::from_serialization(value, context)?,
            //     context.node_reference(input)?,
            // )
            // .prepare()),
            Node::Custom(custom) => custom.instantiate_node(context),
        }
    }
}
