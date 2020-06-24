use kurbo::{BezPath, PathEl};
use rodio::Source;
use std::{
    convert::TryFrom,
    sync::{Arc, RwLock},
    time::Duration,
};

#[derive(Debug, Clone, Copy)]
enum EnvelopeStage {
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

#[derive(Debug, Eq, PartialEq)]
pub enum PlayingState {
    Playing,
    Stopping,
    Stopped,
}

#[derive(Default, Debug)]
pub struct EnvelopeCurve {
    segments: Arc<Vec<EnvelopeSegment>>,
}

impl EnvelopeCurve {
    fn instantiate(&self) -> EnvelopeCurveInstance {
        EnvelopeCurveInstance {
            segments: self.segments.clone(),
            segment: None,
            start_frame: None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EnvelopeCurveError {
    #[error("curve must not have any breaks")]
    NonContiguousPath,
    #[error("curve is too complex")]
    TooComplex,
}

impl TryFrom<BezPath> for EnvelopeCurve {
    type Error = EnvelopeCurveError;
    fn try_from(curve: BezPath) -> Result<Self, Self::Error> {
        let mut flattened_path = Vec::new();
        // The tolerance is the max distance between points
        // Initial thought is that we want 1ms resolution,
        // so that's why 0.01 is passed
        curve.flatten(0.01, |path| flattened_path.push(path));

        //
        let mut segments = Vec::new();
        let mut current_location = None;
        for path in flattened_path {
            match path {
                PathEl::MoveTo(point) => {
                    current_location = match current_location {
                        Some(_) => return Err(EnvelopeCurveError::NonContiguousPath),
                        None => Some(point),
                    }
                }
                PathEl::LineTo(point) => {
                    let starting_point = current_location.unwrap_or_default();

                    segments.push(EnvelopeSegment {
                        duration: (point.x - starting_point.x) as f32,
                        start_value: starting_point.y as f32,
                        end_value: point.y as f32,
                    });

                    current_location = Some(point);
                }
                PathEl::ClosePath => {}
                _ => unreachable!("kurbo should only send line segments, not curves"),
            }
        }

        Ok(Self {
            segments: Arc::new(segments),
        })
    }
}

impl TryFrom<Option<BezPath>> for EnvelopeCurve {
    type Error = EnvelopeCurveError;
    fn try_from(curve: Option<BezPath>) -> Result<Self, Self::Error> {
        match curve {
            Some(curve) => Self::try_from(curve),
            None => Ok(Self::default()),
        }
    }
}

pub struct EnvelopeConfiguration {
    pub attack: EnvelopeCurve,
    pub hold: EnvelopeCurve,
    pub decay: EnvelopeCurve,
    pub sustain: f32,
    pub release: EnvelopeCurve,
}

impl EnvelopeConfiguration {
    pub fn asdr(
        attack: Option<BezPath>,
        decay: Option<BezPath>,
        sustain: f32,
        release: Option<BezPath>,
    ) -> Result<Self, EnvelopeCurveError> {
        Self::ahsdr(attack, None, decay, sustain, release)
    }

    pub fn ahsdr(
        attack: Option<BezPath>,
        hold: Option<BezPath>,
        decay: Option<BezPath>,
        sustain: f32,
        release: Option<BezPath>,
    ) -> Result<Self, EnvelopeCurveError> {
        let attack = EnvelopeCurve::try_from(attack)?;
        let hold = EnvelopeCurve::try_from(hold)?;
        let decay = EnvelopeCurve::try_from(decay)?;
        let release = EnvelopeCurve::try_from(release)?;
        Ok(Self {
            attack,
            hold,
            decay,
            sustain,
            release,
        })
    }

    pub fn envelop<T: Source<Item = f32>>(
        &self,
        source: T,
    ) -> (Envelope<T>, Arc<RwLock<PlayingState>>) {
        let is_playing = Arc::new(RwLock::new(PlayingState::Playing));
        let is_playing_handle = is_playing.clone();

        let envelope = Envelope {
            frame: 0,
            state: EnvelopeStage::Attack,

            attack: self.attack.instantiate(),
            hold: self.hold.instantiate(),
            decay: self.decay.instantiate(),
            sustain: self.sustain,
            release: self.release.instantiate(),

            source,
            is_playing,
        };

        (envelope, is_playing_handle)
    }
}

#[derive(Debug, Clone)]
pub struct EnvelopeSegment {
    pub duration: f32,
    pub start_value: f32,
    pub end_value: f32,
}

struct EnvelopeCurveInstance {
    segments: Arc<Vec<EnvelopeSegment>>,
    start_frame: Option<u32>,
    segment: Option<usize>,
}

impl EnvelopeCurveInstance {
    fn advance(&mut self, current_frame: u32, sample_rate: u32) -> Option<f32> {
        let start_frame = match self.start_frame {
            Some(frame) => frame,
            None => {
                self.start_frame = Some(current_frame);
                current_frame
            }
        };

        let current_segment_index = match self.segment {
            Some(segment_index) => segment_index,
            None => {
                if self.segments.is_empty() {
                    return None;
                }

                self.segment = Some(0);
                0
            }
        };

        let segment = &self.segments[current_segment_index];
        let segment_frames = (segment.duration * sample_rate as f32) as u32;
        let relative_frame = current_frame - start_frame;

        if segment_frames <= relative_frame {
            if current_segment_index + 1 >= self.segments.len() {
                // No more segments
                return None;
            }

            self.segment = Some(current_segment_index + 1);
            self.advance(current_frame, sample_rate)
        } else {
            // lerp the value
            let fractional_position = relative_frame as f32 / segment_frames as f32;
            Some(
                segment.start_value
                    + (segment.end_value - segment.start_value) * fractional_position,
            )
        }
    }
}

pub struct Envelope<T> {
    frame: u32,
    state: EnvelopeStage,
    is_playing: Arc<RwLock<PlayingState>>,

    attack: EnvelopeCurveInstance,
    hold: EnvelopeCurveInstance,
    decay: EnvelopeCurveInstance,
    sustain: f32,
    release: EnvelopeCurveInstance,

    pub source: T,
}

impl<T> Envelope<T>
where
    T: Source<Item = f32>,
{
    fn advance_attack(&mut self) -> (EnvelopeStage, Option<f32>) {
        match self.attack.advance(self.frame, self.source.sample_rate()) {
            Some(value) => (EnvelopeStage::Attack, Some(value)),
            None => self.advance_hold(),
        }
    }

    fn advance_hold(&mut self) -> (EnvelopeStage, Option<f32>) {
        match self.hold.advance(self.frame, self.source.sample_rate()) {
            Some(value) => (EnvelopeStage::Hold, Some(value)),
            None => self.advance_decay(),
        }
    }

    fn advance_decay(&mut self) -> (EnvelopeStage, Option<f32>) {
        match self.decay.advance(self.frame, self.source.sample_rate()) {
            Some(value) => (EnvelopeStage::Decay, Some(value)),
            None => self.sustain(),
        }
    }

    fn sustain(&mut self) -> (EnvelopeStage, Option<f32>) {
        (EnvelopeStage::Sustain, Some(self.sustain))
    }

    fn advance_release(&mut self) -> (EnvelopeStage, Option<f32>) {
        match self.release.advance(self.frame, self.source.sample_rate()) {
            Some(value) => (EnvelopeStage::Release, Some(value)),
            None => (EnvelopeStage::Completed, None),
        }
    }

    fn should_stop(&self) -> bool {
        *self.is_playing.read().unwrap() != PlayingState::Playing
    }
}

impl<T> Iterator for Envelope<T>
where
    T: Source<Item = f32>,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.is_playing() && self.should_stop() {
            self.state = EnvelopeStage::Release;
        }

        if let Some(value) = self.source.next() {
            self.frame = self.frame.wrapping_add(1);

            let (new_state, amplitude) = match self.state {
                EnvelopeStage::Attack => self.advance_attack(),
                EnvelopeStage::Hold => self.advance_hold(),
                EnvelopeStage::Decay => self.advance_decay(),
                EnvelopeStage::Sustain => self.sustain(),
                EnvelopeStage::Release => self.advance_release(),
                EnvelopeStage::Completed => (EnvelopeStage::Completed, None),
            };

            self.state = new_state;
            amplitude.map(|a| a * value)
        } else {
            None
        }
    }
}

impl<T> Source for Envelope<T>
where
    T: Source<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}
