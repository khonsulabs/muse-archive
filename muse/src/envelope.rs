use rodio::Source;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

mod config;
mod curve;
mod stage;
pub use config::EnvelopeConfiguration;
use curve::*;
use stage::EnvelopeStage;

#[derive(Debug, Eq, PartialEq)]
pub enum PlayingState {
    Playing,
    Stopping,
    Stopped,
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

    fn stop_if_needed_or<F: Fn(&mut Self) -> (EnvelopeStage, Option<f32>)>(
        &mut self,
        f: F,
    ) -> (EnvelopeStage, Option<f32>) {
        if self.should_stop() {
            self.advance_release()
        } else {
            f(self)
        }
    }

    fn advance_hold(&mut self) -> (EnvelopeStage, Option<f32>) {
        match self.hold.advance(self.frame, self.source.sample_rate()) {
            Some(value) => (EnvelopeStage::Hold, Some(value)),
            None => self.stop_if_needed_or(Self::advance_decay),
        }
    }

    fn advance_decay(&mut self) -> (EnvelopeStage, Option<f32>) {
        match self.decay.advance(self.frame, self.source.sample_rate()) {
            Some(value) => (EnvelopeStage::Decay, Some(value)),
            None => self.stop_if_needed_or(Self::sustain),
        }
    }

    fn sustain(&mut self) -> (EnvelopeStage, Option<f32>) {
        (EnvelopeStage::Sustain, Some(self.sustain))
    }

    fn advance_release(&mut self) -> (EnvelopeStage, Option<f32>) {
        match self.release.advance(self.frame, self.source.sample_rate()) {
            Some(value) => (EnvelopeStage::Release, Some(value)),
            None => self.stop(),
        }
    }

    fn should_stop(&self) -> bool {
        *self.is_playing.read().unwrap() != PlayingState::Playing
    }

    fn stop(&self) -> (EnvelopeStage, Option<f32>) {
        let mut control = self.is_playing.write().unwrap();
        *control = PlayingState::Stopped;
        (EnvelopeStage::Completed, None)
    }
}

impl<T> Iterator for Envelope<T>
where
    T: Source<Item = f32>,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.source.next() {
            self.frame = self.frame.wrapping_add(1);

            let (new_state, amplitude) = match self.state {
                EnvelopeStage::Attack => self.advance_attack(),
                EnvelopeStage::Hold => self.stop_if_needed_or(Self::advance_hold),
                EnvelopeStage::Decay => self.stop_if_needed_or(Self::advance_decay),
                EnvelopeStage::Sustain => self.stop_if_needed_or(Self::sustain),
                EnvelopeStage::Release => self.advance_release(),
                EnvelopeStage::Completed => self.stop(),
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
