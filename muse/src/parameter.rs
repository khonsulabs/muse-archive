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
