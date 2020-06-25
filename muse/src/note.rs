pub use pitch_calc::{Letter, Octave, Step};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub step: u8,
    pub velocity: u8,
}

impl Note {
    pub fn new(step: u8, velocity: u8) -> Self {
        Self { step, velocity }
    }
    pub fn hertz(&self) -> f32 {
        pitch_calc::hz_from_step(self.step as f32)
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (letter, octave) = pitch_calc::letter_octave_from_step(self.step as f32);
        f.write_fmt(format_args!("{:?}{}({})", letter, octave, self.velocity))
    }
}
