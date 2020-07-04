pub use pitch_calc::{Letter, Octave};

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Note {
    hertz: f32,
    velocity: u8,
}

impl Note {
    pub fn new(step: f32, velocity: u8) -> Self {
        Self {
            hertz: pitch_calc::hz_from_step(step),
            velocity,
        }
    }

    pub fn from_hertz(hertz: f32, velocity: u8) -> Self {
        Self { hertz, velocity }
    }

    pub fn step(&self) -> f32 {
        pitch_calc::step_from_hz(self.hertz())
    }

    pub fn hertz(&self) -> f32 {
        self.hertz
    }

    pub fn velocity(&self) -> u8 {
        self.velocity
    }

    pub fn velocity_percent(&self) -> f32 {
        self.velocity as f32 / 127.
    }

    pub fn letter_octave(&self) -> (Letter, Octave) {
        pitch_calc::letter_octave_from_step(self.step() as f32)
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (letter, octave) = self.letter_octave();
        f.write_fmt(format_args!("{:?}{}({})", letter, octave, self.velocity))
    }
}
