use crate::instrument::ControlHandle;

mod config;
mod curve;
pub use config::{CurveBuilder, EnvelopeBuilder, EnvelopeConfiguration, EnvelopeCurve, Point};
use curve::EnvelopeCurveInstance;
pub use curve::{EnvelopeCurveError, FlattenedCurve};

use crate::sampler::FrameInfo;

#[derive(Debug, Clone, Copy)]
pub enum EnvelopeStage {
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
    Completed,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PlayingState {
    Playing,
    Sustaining,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct Envelope {
    state: EnvelopeStage,
    is_playing: ControlHandle,
    last_value: Option<f32>,

    last_playing_check: usize,

    attack: EnvelopeCurveInstance,
    hold: EnvelopeCurveInstance,
    decay: EnvelopeCurveInstance,
    sustain: EnvelopeCurveInstance,
    release: EnvelopeCurveInstance,
}

impl Envelope {
    fn advance_attack(&mut self, frame: &FrameInfo) -> (EnvelopeStage, Option<f32>) {
        match self.attack.advance(frame) {
            Some(value) => (EnvelopeStage::Attack, Some(value)),
            None => {
                println!("Advancing to hold");
                self.advance_hold(frame)
            }
        }
    }

    fn stop_if_needed_or<F: Fn(&mut Self, &FrameInfo) -> (EnvelopeStage, Option<f32>)>(
        &mut self,
        f: F,
        frame: &FrameInfo,
    ) -> (EnvelopeStage, Option<f32>) {
        if self.should_stop() {
            println!("Skipping to release");
            self.advance_release(frame)
        } else {
            f(self, frame)
        }
    }

    fn advance_hold(&mut self, frame: &FrameInfo) -> (EnvelopeStage, Option<f32>) {
        match self.hold.advance(frame) {
            Some(value) => (EnvelopeStage::Hold, Some(value)),
            None => {
                println!("Advancing to decay");
                self.stop_if_needed_or(Self::advance_decay, frame)
            }
        }
    }

    fn advance_decay(&mut self, frame: &FrameInfo) -> (EnvelopeStage, Option<f32>) {
        match self.decay.advance(frame) {
            Some(value) => (EnvelopeStage::Decay, Some(value)),
            None => {
                println!("Advancing to sustain");
                self.stop_if_needed_or(Self::sustain, frame)
            }
        }
    }

    fn sustain(&mut self, frame: &FrameInfo) -> (EnvelopeStage, Option<f32>) {
        match self.sustain.advance(frame) {
            Some(value) => (EnvelopeStage::Decay, Some(value)),
            None => (
                EnvelopeStage::Sustain,
                self.sustain.terminal_value().or_else(|| self.last_value),
            ),
        }
    }

    fn advance_release(&mut self, frame: &FrameInfo) -> (EnvelopeStage, Option<f32>) {
        if self.release.is_at_start() {
            println!("Releasing {:?}", self.last_value);
            if let Some(last_value) = self.last_value {
                self.release.descend_to(last_value, frame);
            }
        }
        match self.release.advance(frame) {
            Some(value) => (EnvelopeStage::Release, Some(value)),
            None => self.stop(),
        }
    }

    fn should_stop(&mut self) -> bool {
        match self.is_playing.load() {
            PlayingState::Playing | PlayingState::Sustaining => false,
            _ => true,
        }
    }

    fn stop(&self) -> (EnvelopeStage, Option<f32>) {
        self.is_playing.store(PlayingState::Stopped);
        (EnvelopeStage::Completed, None)
    }

    pub fn next(&mut self, frame: &FrameInfo) -> Option<f32> {
        let (new_state, amplitude) = match self.state {
            EnvelopeStage::Attack => self.advance_attack(frame),
            EnvelopeStage::Hold => self.stop_if_needed_or(Self::advance_hold, frame),
            EnvelopeStage::Decay => self.stop_if_needed_or(Self::advance_decay, frame),
            EnvelopeStage::Sustain => self.stop_if_needed_or(Self::sustain, frame),
            EnvelopeStage::Release => self.advance_release(frame),
            EnvelopeStage::Completed => self.stop(),
        };

        self.state = new_state;
        self.last_value = amplitude;
        amplitude
    }
}
