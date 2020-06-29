use crate::parameter::Parameter;
use rodio::Source;
use std::time::Duration;

pub struct Pan {
    pan: Parameter,
    source: Box<dyn Source<Item = f32> + Send + Sync>,
    current_sample: Option<PanFrame>,
}

#[derive(Clone, Copy, Debug)]
struct PanFrame {
    sample: f32,
    pan: f32,
}

impl PanFrame {
    pub fn value(&self) -> f32 {
        self.sample * self.pan
    }

    pub fn second_channel_value(&self) -> f32 {
        self.sample * (1.0 - self.pan)
    }
}

impl Pan {
    pub fn new<T: Source<Item = f32> + Send + Sync + 'static>(
        parameter: Parameter,
        source: T,
    ) -> Self {
        Self {
            pan: parameter,
            source: Box::new(source),
            current_sample: None,
        }
    }

    fn next_frame(&mut self) -> Option<PanFrame> {
        self.source.next().map(|sample| PanFrame {
            sample,
            pan: self.pan.next(self.sample_rate()).unwrap(),
        })
    }
}

impl Iterator for Pan {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        // 2 channel panning, first access stores the value and returns it, the second access consumes the current sample
        if let Some(frame) = self.current_sample {
            self.current_sample = None;
            Some(frame.second_channel_value())
        } else if let Some(frame) = self.next_frame() {
            self.current_sample = Some(frame);
            Some(frame.value())
        } else {
            None
        }
    }
}

impl Source for Pan {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}
