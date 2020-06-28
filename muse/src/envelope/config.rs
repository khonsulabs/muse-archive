// TODO BAD JON
use super::{curve::*, Envelope, EnvelopeStage, PlayingState};
use crate::{instrument::ControlHandles, parameter::Parameter};
use kurbo::BezPath;
use std::{
    convert::TryFrom,
    sync::{Arc, RwLock},
    time::Duration,
};

#[derive(Debug, Default)]
pub struct EnvelopeBuilder {
    pub attack: Option<EnvelopeCurve>,
    pub hold: Option<EnvelopeCurve>,
    pub decay: Option<EnvelopeCurve>,
    pub sustain: Option<EnvelopeCurve>,
    pub release: Option<EnvelopeCurve>,
}

impl EnvelopeBuilder {
    pub fn attack(mut self, attack: EnvelopeCurve) -> Self {
        self.attack = Some(attack);
        self
    }

    pub fn hold(mut self, hold: EnvelopeCurve) -> Self {
        self.hold = Some(hold);
        self
    }

    pub fn decay(mut self, decay: EnvelopeCurve) -> Self {
        self.decay = Some(decay);
        self
    }

    pub fn sustain(mut self, sustain: EnvelopeCurve) -> Self {
        self.sustain = Some(sustain);
        self
    }

    pub fn release(mut self, release: EnvelopeCurve) -> Self {
        self.release = Some(release);
        self
    }

    fn flatten_timed_curve(
        curve: Option<EnvelopeCurve>,
        start_value: f32,
        end_value: f32,
    ) -> Result<FlattenedCurve, EnvelopeCurveError> {
        match curve {
            Some(curve) => match curve {
                EnvelopeCurve::Curve(flattened_curve) => Ok(flattened_curve),
                EnvelopeCurve::Sustain(_) => Err(EnvelopeCurveError::InvalidCurveType),
                EnvelopeCurve::Timed(duration) => Ok(EnvelopeSegment {
                    start_value,
                    end_value,
                    duration: duration.as_secs_f32(),
                }
                .into()),
            },
            None => Ok(Default::default()),
        }
    }

    fn flatten_sustain_curve(
        curve: Option<EnvelopeCurve>,
        default_magnitude: f32,
    ) -> Result<FlattenedCurve, EnvelopeCurveError> {
        match curve {
            Some(curve) => match curve {
                EnvelopeCurve::Curve(flattened_curve) => Ok(flattened_curve),
                EnvelopeCurve::Timed(_) => Err(EnvelopeCurveError::InvalidCurveType),
                EnvelopeCurve::Sustain(magnitude) => Ok(FlattenedCurve::sustain(magnitude)),
            },
            None => Ok(FlattenedCurve::sustain(default_magnitude)),
        }
    }

    pub fn build(self) -> Result<EnvelopeConfiguration, EnvelopeCurveError> {
        // Attack goes to 1.0,
        //  start: 0
        //  end: 1
        // hold has no default number, just holds attack_end
        //  start: attack.end
        //  end: attack.end
        // decay has no default number, transitions from hold_end to sustain_start
        //  start: hold.end
        //  end: sustain.start
        // sustain has a value (time is invalid), or if none is specified, it's decay_end
        //  start: value (or curve start)
        //  end: value (or curve end)
        // release has a end of 0
        //  start: sustain.end
        //  end: 0

        let attack = Self::flatten_timed_curve(self.attack, 0.0, 1.0)?;
        let attack_end = attack.terminal_value().unwrap_or(1.0);
        let sustain = Self::flatten_sustain_curve(self.sustain, attack_end)?;
        let hold = Self::flatten_timed_curve(self.hold, attack_end, attack_end)?;

        let decay = Self::flatten_timed_curve(
            self.decay,
            hold.terminal_value().unwrap_or(attack_end),
            sustain.start_value().unwrap(),
        )?;

        let release =
            Self::flatten_timed_curve(self.release, sustain.terminal_value().unwrap(), 0.0)?;

        Ok(EnvelopeConfiguration {
            attack,
            hold,
            decay,
            sustain,
            release,
        })
    }
}

#[derive(Default, Clone)]
pub struct EnvelopeConfiguration {
    pub attack: FlattenedCurve,
    pub hold: FlattenedCurve,
    pub decay: FlattenedCurve,
    pub sustain: FlattenedCurve,
    pub release: FlattenedCurve,
}

impl EnvelopeConfiguration {
    // pub fn envelop<T: Source<Item = f32>>(
    //     &self,
    //     source: T,
    // ) -> (Envelope<T>, Arc<RwLock<PlayingState>>) {
    //     let is_playing = Arc::new(RwLock::new(PlayingState::Playing));
    //     let is_playing_handle = is_playing.clone();

    //     let envelope = Envelope {
    //         frame: 0,
    //         state: EnvelopeStage::Attack,
    //         last_value: None,

    //         attack: self.attack.instantiate(),
    //         hold: self.hold.instantiate(),
    //         decay: self.decay.instantiate(),
    //         sustain: self.sustain.instantiate(),
    //         release: self.release.instantiate(),

    //         source,
    //         is_playing,
    //     };

    //     (envelope, is_playing_handle)
    // }

    pub fn as_parameter(&self, controls: &mut ControlHandles) -> Parameter {
        let is_playing = Arc::new(RwLock::new(PlayingState::Playing));
        controls.push(is_playing.clone());

        let envelope = Envelope {
            frame: 0,
            state: EnvelopeStage::Attack,
            last_value: None,

            attack: self.attack.instantiate(),
            hold: self.hold.instantiate(),
            decay: self.decay.instantiate(),
            sustain: self.sustain.instantiate(),
            release: self.release.instantiate(),

            is_playing,
        };

        Parameter::Envelope(Box::new(envelope))
    }
}

pub use kurbo::Point;

#[derive(Debug)]
pub enum EnvelopeCurve {
    /// A curve representing one or more line segments
    Curve(FlattenedCurve),
    /// A flat curve that lasts for a specific duration (for ahd + r)
    Timed(Duration),
    /// A flat curve that holds for an infinite duration at a specified magnitude
    Sustain(f32),
}

impl EnvelopeCurve {
    // pub fn extents(&self, default_min: f32, default_max: f32) -> (f32, f32) {
    //     match self {
    //         Self::Curve(curve) => (
    //             curve.segments.get(0).map(|s| s.start_value),
    //             curve.segments.last().map(|s| s.end_value),
    //         ),
    //         Self::Timed(_) => (None, None),
    //         Self::Sustain()
    //     }
    // }
    pub fn terminal_value(&self, carryover_value: f32) -> f32 {
        match self {
            Self::Curve(curve) => curve
                .segments
                .last()
                .map(|s| s.end_value)
                .unwrap_or(carryover_value),
            _ => carryover_value,
        }
    }
}

#[derive(Default)]
pub struct CurveBuilder {
    path: BezPath,
}

impl CurveBuilder {
    pub fn move_to(mut self, seconds: f32, magnitude: f32) -> Result<Self, EnvelopeCurveError> {
        if self.path.elements().is_empty() {
            self.path
                .move_to(Point::new(seconds as f64, magnitude as f64));
            Ok(self)
        } else {
            Err(EnvelopeCurveError::NonContiguousPath)
        }
    }

    pub fn line_to(mut self, seconds: f32, magnitude: f32) -> Self {
        self.path
            .line_to(Point::new(seconds as f64, magnitude as f64));
        self
    }

    pub fn curve_to(
        mut self,
        seconds: f32,
        magnitude: f32,
        start_control: Point,
        end_control: Point,
    ) -> Self {
        self.path.curve_to(
            start_control,
            end_control,
            Point::new(seconds as f64, magnitude as f64),
        );
        self
    }

    pub fn build(self) -> Result<EnvelopeCurve, EnvelopeCurveError> {
        Ok(EnvelopeCurve::Curve(FlattenedCurve::try_from(self.path)?))
    }
}
