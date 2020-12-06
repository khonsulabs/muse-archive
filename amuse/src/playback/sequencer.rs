use std::collections::HashMap;

pub struct Sequencer<Timeline> {
    pub tracks: Vec<Track<Timeline>>,
}

pub struct Track<Timeline> {
    pub instrument: muse::instrument::serialization::Instrument,
    pub blocks: HashMap<Timeline, Block>,
}

pub struct Beats {
    pub value: u16,
    pub scale: Scale,
}

pub enum Scale {
    Whole,
    Fraction(u16),
}

pub enum Pitch {}

pub enum Block {
    Rest(Beats),
    Play(Pitch, Beats),
}
