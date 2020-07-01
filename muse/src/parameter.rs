use crate::envelope::Envelope;

#[derive(Debug, Clone)]
pub enum Parameter {
    Value(f32),
    Envelope(Box<Envelope>),
}

impl Parameter {
    pub fn next(&mut self, sample_rate: u32, clock: usize) -> Option<f32> {
        match self {
            Self::Value(value) => Some(*value),
            Self::Envelope(envelope) => envelope.next(sample_rate, clock),
        }
    }
}

#[cfg(feature = "serialization")]
use crate::instrument::serialization::{
    node_instantiator::Context, Error, Parameter as ParameterSpec,
};

#[cfg(feature = "serialization")]
impl Parameter {
    pub fn from_serialization(
        spec: &ParameterSpec,
        context: &mut Context<'_>,
    ) -> Result<Self, Error> {
        let parameter = match spec {
            ParameterSpec::NoteHertz => Parameter::Value(context.note.hertz()),
            ParameterSpec::NoteStep => Parameter::Value(context.note.step as f32),
            ParameterSpec::NoteVelocity => Parameter::Value(context.note.velocity as f32),
            ParameterSpec::Envelope(envelope_name) => {
                let envelope = context.envelope(envelope_name)?;
                envelope.as_parameter(context.controls)
            }
            ParameterSpec::Value(value) => Parameter::Value(*value),
        };

        Ok(parameter)
    }
}
