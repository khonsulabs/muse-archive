#[derive(Debug, Clone, Copy)]
pub enum EnvelopeStage {
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
    Completed,
}

impl EnvelopeStage {
    fn is_playing(&self) -> bool {
        match self {
            EnvelopeStage::Attack
            | EnvelopeStage::Hold
            | EnvelopeStage::Decay
            | EnvelopeStage::Sustain => true,
            _ => false,
        }
    }
}
