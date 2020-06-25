use super::{curve::*, Envelope, EnvelopeStage, PlayingState};
use kurbo::BezPath;
use rodio::Source;
use std::{
    convert::TryFrom,
    sync::{Arc, RwLock},
};

pub struct EnvelopeConfiguration {
    pub attack: EnvelopeCurve,
    pub hold: EnvelopeCurve,
    pub decay: EnvelopeCurve,
    pub sustain: EnvelopeCurve,
    pub release: EnvelopeCurve,
}

impl EnvelopeConfiguration {
    pub fn asdr(
        attack: Option<BezPath>,
        decay: Option<BezPath>,
        sustain: Option<BezPath>,
        release: Option<BezPath>,
    ) -> Result<Self, EnvelopeCurveError> {
        Self::ahsdr(attack, None, decay, sustain, release)
    }

    pub fn ahsdr(
        attack: Option<BezPath>,
        hold: Option<BezPath>,
        decay: Option<BezPath>,
        sustain: Option<BezPath>,
        release: Option<BezPath>,
    ) -> Result<Self, EnvelopeCurveError> {
        let attack = EnvelopeCurve::try_from(attack)?;
        let hold = EnvelopeCurve::try_from(hold)?;
        let decay = EnvelopeCurve::try_from(decay)?;
        let sustain = EnvelopeCurve::try_from(sustain)?;
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
            sustain: self.sustain.instantiate(),
            release: self.release.instantiate(),

            source,
            is_playing,
        };

        (envelope, is_playing_handle)
    }
}
