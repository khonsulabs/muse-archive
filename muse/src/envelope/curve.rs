use kurbo::{BezPath, PathEl};
use std::{convert::TryFrom, sync::Arc};

#[derive(Default, Debug)]
pub struct EnvelopeCurve {
    segments: Arc<Vec<EnvelopeSegment>>,
}

impl EnvelopeCurve {
    pub fn instantiate(&self) -> EnvelopeCurveInstance {
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

#[derive(Debug, Clone)]
pub struct EnvelopeSegment {
    pub duration: f32,
    pub start_value: f32,
    pub end_value: f32,
}

pub struct EnvelopeCurveInstance {
    segments: Arc<Vec<EnvelopeSegment>>,
    start_frame: Option<u32>,
    segment: Option<usize>,
}

impl EnvelopeCurveInstance {
    pub fn advance(&mut self, current_frame: u32, sample_rate: u32) -> Option<f32> {
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
