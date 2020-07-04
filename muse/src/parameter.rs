use crate::{envelope::Envelope, sampler::FrameInfo};

#[derive(Debug, Clone)]
pub enum Parameter {
    Value(f32),
    Envelope(Box<Envelope>),
    NoteHertz,
    NoteVelocity,
    NoteStep,
}

impl Parameter {
    pub fn next(&mut self, frame: &FrameInfo) -> Option<f32> {
        match self {
            Self::Value(value) => Some(*value),
            Self::Envelope(envelope) => envelope.next(frame),
            Self::NoteHertz => Some(frame.note.hertz()),
            Self::NoteStep => Some(frame.note.step() as f32),
            Self::NoteVelocity => Some(frame.note.velocity_percent()),
        }
    }
}
