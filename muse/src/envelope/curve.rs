use crate::sampler::FrameInfo;
use kurbo::{BezPath, PathEl};
use std::{convert::TryFrom, sync::Arc};

#[derive(Default, Debug, Clone)]
pub struct FlattenedCurve {
    pub(crate) segments: Arc<Vec<EnvelopeSegment>>,
}

impl FlattenedCurve {
    pub fn instantiate(&self) -> EnvelopeCurveInstance {
        EnvelopeCurveInstance {
            segments: self.segments.clone(),
            segment: None,
            start_frame: None,
            start_frame_offset: None,
        }
    }

    pub fn terminal_value(&self) -> Option<f32> {
        self.segments.last().map(|s| s.end_value)
    }

    pub fn start_value(&self) -> Option<f32> {
        self.segments.get(0).map(|s| s.start_value)
    }

    pub fn sustain(value: f32) -> Self {
        EnvelopeSegment {
            start_value: value,
            end_value: value,
            duration: 0.0,
        }
        .into()
    }
}

impl From<EnvelopeSegment> for FlattenedCurve {
    fn from(segment: EnvelopeSegment) -> Self {
        Self {
            segments: Arc::new(vec![segment]),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EnvelopeCurveError {
    #[error("curve must not have any breaks")]
    NonContiguousPath,
    #[error("curve is too complex")]
    TooComplex,
    #[error("attempting to use the wrong type of curve")]
    InvalidCurveType,
}

impl TryFrom<BezPath> for FlattenedCurve {
    type Error = EnvelopeCurveError;
    fn try_from(curve: BezPath) -> Result<Self, Self::Error> {
        let mut flattened_path = Vec::new();
        // The tolerance is the max distance between points
        // Initial thought is that we want 1ms resolution,
        // so that's why 0.01 is passed
        curve.flatten(0.01, |path| flattened_path.push(path));

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

impl TryFrom<Option<BezPath>> for FlattenedCurve {
    type Error = EnvelopeCurveError;
    fn try_from(curve: Option<BezPath>) -> Result<Self, Self::Error> {
        match curve {
            Some(curve) => Self::try_from(curve),
            None => Ok(Self::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvelopeSegment {
    pub duration: f32,
    pub start_value: f32,
    pub end_value: f32,
}

impl EnvelopeSegment {
    pub fn frames_for_sample_rate(&self, sample_rate: u32) -> usize {
        (self.duration * sample_rate as f32) as usize
    }
}

#[derive(Debug, Clone)]
pub struct EnvelopeCurveInstance {
    segments: Arc<Vec<EnvelopeSegment>>,
    start_frame: Option<usize>,
    start_frame_offset: Option<usize>,
    segment: Option<usize>,
}

impl EnvelopeCurveInstance {
    pub fn advance(&mut self, frame: &FrameInfo) -> Option<f32> {
        let start_frame = match self.start_frame {
            Some(frame) => frame,
            None => {
                self.start_frame = Some(frame.clock);
                frame.clock
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
        let segment_frames = segment.frames_for_sample_rate(frame.sample_rate);
        let relative_frame =
            frame.clock - start_frame + self.start_frame_offset.unwrap_or_default();

        if segment_frames <= relative_frame {
            if current_segment_index + 1 >= self.segments.len() {
                // No more segments
                return None;
            }

            self.segment = Some(current_segment_index + 1);
            self.advance(frame)
        } else {
            // lerp the value
            let fractional_position = relative_frame as f32 / segment_frames as f32;
            Some(
                segment.start_value
                    + (segment.end_value - segment.start_value) * fractional_position,
            )
        }
    }

    pub fn terminal_value(&self) -> Option<f32> {
        self.segments.last().map(|s| s.end_value)
    }

    /// Solves the curve for a y value of `target_value`. Used for making release seamlessly fade from wherever the current state is
    pub fn descend_to(&mut self, target_value: f32, frame: &FrameInfo) {
        if let Some((index, containing_segment)) =
            self.segments.iter().enumerate().find(|(_, segment)| {
                segment.start_value > target_value && segment.end_value <= target_value
            })
        {
            self.segment = Some(index);

            let segment_value_delta = containing_segment.start_value - containing_segment.end_value;
            let relative_value = containing_segment.start_value - target_value;
            let value_ratio = relative_value / segment_value_delta;
            let segment_frames = containing_segment.frames_for_sample_rate(frame.sample_rate);

            self.start_frame_offset = Some((value_ratio * segment_frames as f32) as usize);
            println!(
                "jump_to: svd {} rv {} vr {}, sf {}, off {:?}",
                segment_value_delta,
                relative_value,
                value_ratio,
                segment_frames,
                self.start_frame_offset
            );
        } else if target_value == 0.0 {
            // Setting the start_frame to 0 is a simple shortcut to making sure the curve is finished
            self.start_frame = Some(0);
            self.segment = Some(self.segments.len() - 1);
        }
    }

    pub fn is_at_start(&self) -> bool {
        self.start_frame.is_none()
    }
}
