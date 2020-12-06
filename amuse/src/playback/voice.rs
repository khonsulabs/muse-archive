use muse::Note;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Voice<T> {
    pub instrument: T,
    pub sequence: VoiceSequence,
}

#[derive(Debug, Clone, Default)]
pub struct VoiceSequence {
    pub steps: Vec<VoiceStep>,
}

impl VoiceSequence {
    pub fn total_beats(&self) -> NoteDuration {
        self.steps.iter().map(|s| s.beat).max().unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct VoiceStep {
    pub beat: NoteDuration,
    pub command: VoiceCommand,
}

#[derive(Debug, Clone)]
pub enum VoiceCommand {
    Play(Note),
    Release,
    Poly(Vec<VoiceSequence>),
}

pub struct VoiceBuilder<T> {
    instrument: T,
    sequence: SequenceBuilder,
}

#[derive(Default, Debug)]
pub struct SequenceBuilder {
    sequence: VoiceSequence,
    needs_release: bool,
    current_beat: NoteDuration,
}

impl<T> Voice<T> {
    pub fn build(instrument: T) -> VoiceBuilder<T> {
        VoiceBuilder::new(instrument)
    }
}

impl<T> VoiceBuilder<T> {
    pub fn new(instrument: T) -> Self {
        VoiceBuilder {
            instrument,
            sequence: Default::default(),
        }
    }

    pub fn build(self) -> Voice<T> {
        Voice {
            instrument: self.instrument,
            sequence: self.sequence.build(),
        }
    }

    pub fn play(mut self, note: Note) -> Self {
        self.sequence = self.sequence.play(note);
        self
    }

    pub fn hold_for(mut self, duration: NoteDuration) -> Self {
        self.sequence = self.sequence.hold_for(duration);
        self
    }

    pub fn release(mut self) -> Self {
        self.sequence = self.sequence.release();
        self
    }

    pub fn poly<F: FnOnce(PolyBuilder) -> PolyBuilder>(mut self, poly_builder: F) -> Self {
        self.sequence = self.sequence.poly(poly_builder);
        self
    }
}

impl SequenceBuilder {
    pub fn play(mut self, note: Note) -> Self {
        self.sequence.steps.push(VoiceStep {
            beat: self.current_beat,
            command: VoiceCommand::Play(note),
        });
        self.needs_release = true;
        self
    }

    pub fn hold_for(mut self, duration: NoteDuration) -> Self {
        self.current_beat += duration;
        self
    }

    pub fn release(mut self) -> Self {
        self.sequence.steps.push(VoiceStep {
            beat: self.current_beat,
            command: VoiceCommand::Release,
        });
        self.needs_release = false;
        self
    }

    pub fn poly<F: FnOnce(PolyBuilder) -> PolyBuilder>(mut self, poly_builder: F) -> Self {
        let PolyBuilder { sequences } = poly_builder(PolyBuilder::default());
        let beats = sequences
            .iter()
            .map(|s| s.total_beats())
            .max()
            .unwrap_or_default();
        self.sequence.steps.push(VoiceStep {
            beat: self.current_beat,
            command: VoiceCommand::Poly(sequences),
        });
        self.current_beat += beats;
        self
    }

    pub fn build(self) -> VoiceSequence {
        if self.needs_release {
            self.release().sequence
        } else {
            self.sequence
        }
    }
}

#[derive(Default)]
pub struct PolyBuilder {
    sequences: Vec<VoiceSequence>,
}

impl PolyBuilder {
    pub fn part<F: FnOnce(SequenceBuilder) -> SequenceBuilder>(mut self, part_builder: F) -> Self {
        let part = part_builder(SequenceBuilder::default());
        self.sequences.push(part.build());
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NoteDuration {
    /// The whole notes value
    pub whole: u32,
    /// The numerator of the fractional part of a beat
    pub sub: u16,
    /// The denominator, ie thirtysecond notes would be 32
    pub quantization: u16,
}

impl NoteDuration {
    pub fn new(whole: u32, sub: u16, quantization: u16) -> Self {
        Self {
            whole,
            sub,
            quantization,
        }
    }

    pub const fn whole() -> Self {
        Self {
            whole: 1,
            sub: 0,
            quantization: 1,
        }
    }

    pub const fn half() -> Self {
        Self {
            whole: 0,
            sub: 1,
            quantization: 2,
        }
    }

    pub const fn quarter() -> Self {
        Self {
            whole: 0,
            sub: 1,
            quantization: 4,
        }
    }

    pub const fn eighth() -> Self {
        Self {
            whole: 0,
            sub: 1,
            quantization: 8,
        }
    }

    pub const fn sixteenth() -> Self {
        Self {
            whole: 0,
            sub: 1,
            quantization: 16,
        }
    }

    pub const fn thirtysecondth() -> Self {
        Self {
            whole: 0,
            sub: 1,
            quantization: 32,
        }
    }

    pub fn dotted(self) -> Self {
        self * NoteDuration::new(3, 0, 1) / NoteDuration::new(2, 0, 1)
    }

    pub fn normalize(mut self) -> Self {
        if self.sub >= self.quantization {
            self.whole += (self.sub / self.quantization) as u32;
            self.sub %= self.quantization;
        }
        loop {
            let gcd = num::integer::gcd(self.sub, self.quantization);
            if gcd > 1 {
                self.sub /= gcd;
                self.quantization /= gcd;
            } else {
                break;
            }
        }
        self
    }

    pub fn to_quantization(mut self, new_quantization: u16) -> Self {
        if self.quantization != new_quantization {
            let ratio = new_quantization as f32 / self.quantization as f32;
            self.sub = (self.sub as f32 * ratio).round() as u16;
            self.quantization = new_quantization;
        }
        self
    }

    fn convert_to_common_quantization(&mut self, other: &mut Self) {
        if self.quantization != other.quantization {
            let quantization = num::integer::lcm(self.quantization, other.quantization);

            *self = self.to_quantization(quantization);
            *other = other.to_quantization(quantization);
        }
    }

    pub fn duration(&self, beats_per_minute: f32) -> Duration {
        let beats = self.whole as f32 + self.sub as f32 / self.quantization as f32;
        let seconds = beats * beats_per_minute / 60.;
        Duration::from_secs_f32(seconds)
    }

    fn quantized_numerator(&self) -> u16 {
        (self.whole * self.quantization as u32 + self.sub as u32) as u16
    }
}

impl Default for NoteDuration {
    fn default() -> Self {
        Self {
            whole: 0,
            sub: 0,
            quantization: 1,
        }
    }
}

impl std::ops::Add for NoteDuration {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.convert_to_common_quantization(&mut rhs);
        self.whole += rhs.whole;
        self.sub += rhs.sub;
        self.normalize()
    }
}

impl std::ops::AddAssign for NoteDuration {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for NoteDuration {
    type Output = Self;

    fn sub(mut self, mut rhs: Self) -> Self::Output {
        self.convert_to_common_quantization(&mut rhs);
        self.whole -= rhs.whole;
        if rhs.sub > self.sub {
            self.whole -= 1;
            self.sub = (self.sub as u32 + self.quantization as u32 - rhs.sub as u32) as u16;
        } else {
            self.sub -= rhs.sub;
        }
        self.normalize()
    }
}

impl std::ops::SubAssign for NoteDuration {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl std::ops::Mul for NoteDuration {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.sub = self.quantized_numerator() * rhs.quantized_numerator();
        self.quantization *= rhs.quantization;
        self.whole = 0;
        self.normalize()
    }
}

impl std::ops::Div for NoteDuration {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self::Output {
        // Cross multiplication
        self.sub = self.quantized_numerator() * rhs.quantization;
        self.quantization *= rhs.quantized_numerator();
        self.whole = 0;
        self.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beat_math() {
        assert_eq!(
            NoteDuration::new(1, 1, 2) + NoteDuration::new(1, 1, 2),
            NoteDuration::new(3, 0, 1)
        );
        assert_eq!(
            NoteDuration::new(1, 1, 2) + NoteDuration::new(1, 1, 8),
            NoteDuration::new(2, 5, 8)
        );
        assert_eq!(
            NoteDuration::new(1, 1, 2) + NoteDuration::new(1, 0, 1),
            NoteDuration::new(2, 1, 2)
        );
        assert_eq!(
            NoteDuration::new(1, 1, 2) - NoteDuration::new(1, 1, 2),
            NoteDuration::new(0, 0, 1)
        );
        assert_eq!(
            NoteDuration::new(1, 1, 2) - NoteDuration::new(0, 1, 8),
            NoteDuration::new(1, 3, 8)
        );
        assert_eq!(
            NoteDuration::new(1, 1, 2) - NoteDuration::new(0, 5, 8),
            NoteDuration::new(0, 7, 8)
        );
        assert_eq!(
            NoteDuration::new(1, 1, 2) / NoteDuration::new(0, 1, 2),
            NoteDuration::new(3, 0, 1)
        );
        assert_eq!(
            NoteDuration::new(1, 1, 2) * NoteDuration::new(0, 1, 2),
            NoteDuration::new(0, 3, 4)
        );
    }
}
