use std::sync::{Arc, RwLock};

mod config;
mod curve;
pub use config::{CurveBuilder, EnvelopeBuilder, EnvelopeConfiguration, EnvelopeCurve, Point};
pub use curve::{EnvelopeCurveError, FlattenedCurve};
// TODO BAD JON
use curve::*;

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
    is_playing: Arc<RwLock<PlayingState>>,
    last_value: Option<f32>,

    attack: EnvelopeCurveInstance,
    hold: EnvelopeCurveInstance,
    decay: EnvelopeCurveInstance,
    sustain: EnvelopeCurveInstance,
    release: EnvelopeCurveInstance,
}

impl Envelope {
    fn advance_attack(&mut self, sample_rate: u32, clock: usize) -> (EnvelopeStage, Option<f32>) {
        match self.attack.advance(clock, sample_rate) {
            Some(value) => (EnvelopeStage::Attack, Some(value)),
            None => {
                println!("Advancing to hold");
                self.advance_hold(sample_rate, clock)
            }
        }
    }

    fn stop_if_needed_or<F: Fn(&mut Self, u32, usize) -> (EnvelopeStage, Option<f32>)>(
        &mut self,
        f: F,
        sample_rate: u32,
        clock: usize,
    ) -> (EnvelopeStage, Option<f32>) {
        if self.should_stop() {
            println!("Skipping to release");
            self.advance_release(sample_rate, clock)
        } else {
            f(self, sample_rate, clock)
        }
    }

    fn advance_hold(&mut self, sample_rate: u32, clock: usize) -> (EnvelopeStage, Option<f32>) {
        match self.hold.advance(clock, sample_rate) {
            Some(value) => (EnvelopeStage::Hold, Some(value)),
            None => {
                println!("Advancing to decay");
                self.stop_if_needed_or(Self::advance_decay, sample_rate, clock)
            }
        }
    }

    fn advance_decay(&mut self, sample_rate: u32, clock: usize) -> (EnvelopeStage, Option<f32>) {
        match self.decay.advance(clock, sample_rate) {
            Some(value) => (EnvelopeStage::Decay, Some(value)),
            None => {
                println!("Advancing to sustain");
                self.stop_if_needed_or(Self::sustain, sample_rate, clock)
            }
        }
    }

    fn sustain(&mut self, sample_rate: u32, clock: usize) -> (EnvelopeStage, Option<f32>) {
        match self.sustain.advance(clock, sample_rate) {
            Some(value) => (EnvelopeStage::Decay, Some(value)),
            None => (
                EnvelopeStage::Sustain,
                self.sustain.terminal_value().or_else(|| self.last_value),
            ),
        }
    }

    fn advance_release(&mut self, sample_rate: u32, clock: usize) -> (EnvelopeStage, Option<f32>) {
        if self.release.is_at_start() {
            println!("Releasing {:?}", self.last_value);
            if let Some(last_value) = self.last_value {
                self.release.descend_to(last_value, sample_rate);
            }
        }
        match self.release.advance(clock, sample_rate) {
            Some(value) => (EnvelopeStage::Release, Some(value)),
            None => self.stop(),
        }
    }

    fn should_stop(&self) -> bool {
        match *self.is_playing.read().unwrap() {
            PlayingState::Playing | PlayingState::Sustaining => false,
            _ => true,
        }
    }

    fn stop(&self) -> (EnvelopeStage, Option<f32>) {
        let mut control = self.is_playing.write().unwrap();
        *control = PlayingState::Stopped;
        (EnvelopeStage::Completed, None)
    }

    pub fn next(&mut self, sample_rate: u32, clock: usize) -> Option<f32> {
        let (new_state, amplitude) = match self.state {
            EnvelopeStage::Attack => self.advance_attack(sample_rate, clock),
            EnvelopeStage::Hold => self.stop_if_needed_or(Self::advance_hold, sample_rate, clock),
            EnvelopeStage::Decay => self.stop_if_needed_or(Self::advance_decay, sample_rate, clock),
            EnvelopeStage::Sustain => self.stop_if_needed_or(Self::sustain, sample_rate, clock),
            EnvelopeStage::Release => self.advance_release(sample_rate, clock),
            EnvelopeStage::Completed => self.stop(),
        };

        self.state = new_state;
        self.last_value = amplitude;
        amplitude
    }
}
