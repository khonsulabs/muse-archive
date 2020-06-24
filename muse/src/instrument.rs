use crate::{
    envelope::{Envelope, EnvelopeConfiguration, PlayingState},
    oscillators::*,
};
use kurbo::{BezPath, Point};
use rodio::source::Source;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct VirtualInstrument {
    playing_notes: Vec<PlayingNote>,
    device: rodio::Device,
    sustain: bool,
}

pub struct PlayingNote {
    pitch: u8,
    _velocity: u8,
    sink: Option<rodio::Sink>,
    is_playing: Arc<RwLock<PlayingState>>,
}

impl PlayingNote {
    fn is_playing(&self) -> bool {
        let value = self.is_playing.read().unwrap();
        if let PlayingState::Playing = *value {
            true
        } else {
            false
        }
    }

    fn stop(&self) {
        let mut value = self.is_playing.write().unwrap();
        *value = PlayingState::Stopping;
    }
}

impl Drop for PlayingNote {
    fn drop(&mut self) {
        self.stop();

        let sink = std::mem::replace(&mut self.sink, None);
        let is_playing = self.is_playing.clone();

        std::thread::spawn(move || loop {
            {
                let value = is_playing.read().unwrap();
                if let PlayingState::Stopped = *value {
                    sink.unwrap().stop();
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(10))
        });
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Loudness {
    Fortissimo,
    MezzoForte,
    Pianissimo,
}

impl Default for VirtualInstrument {
    fn default() -> Self {
        let device = rodio::default_output_device().expect("No default audio output device");
        Self::new(device)
    }
}

impl VirtualInstrument {
    pub fn new(device: rodio::Device) -> Self {
        Self {
            device,
            playing_notes: Vec::new(),
            sustain: false,
        }
    }
    pub fn play_note(&mut self, pitch: u8, velocity: u8) -> Result<(), anyhow::Error> {
        // We need to re-tone the note, so we'll get rid of the existing notes
        self.playing_notes.retain(|n| n.pitch != pitch);

        let (source, is_playing) = self.generate_note(pitch, velocity)?;
        let sink = rodio::Sink::new(&self.device);
        sink.append(source);
        self.playing_notes.push(PlayingNote {
            pitch,
            _velocity: velocity,
            sink: Some(sink),
            is_playing,
        });

        Ok(())
    }

    pub fn stop_note(&mut self, pitch: u8) {
        if self.sustain {
            // For sustain, we need ot keep the notes playing, but mark that the key isn't pressed
            // so that when the pedal is released, the note isn't filtered out.
            if let Some(existing_note) = self
                .playing_notes
                .iter_mut()
                .find(|note| note.pitch == pitch)
            {
                existing_note.stop();
            }
        } else {
            self.playing_notes.retain(|note| note.pitch != pitch);
        }
    }

    pub fn set_sustain(&mut self, active: bool) {
        self.sustain = active;

        if !active {
            self.playing_notes.retain(|n| n.is_playing());
        }
    }

    fn generate_note(
        &mut self,
        midi_pitch: u8,
        velocity: u8,
    ) -> Result<
        (
            rodio::source::Amplify<Envelope<Oscillator<Sawtooth>>>,
            Arc<RwLock<PlayingState>>,
        ),
        anyhow::Error,
    > {
        // A4 = 440hz, A4 = 69
        let frequency = pitch_calc::calc::hz_from_step(midi_pitch as f32);
        println!(
            "Playing {}hz, {:?}",
            frequency,
            pitch_calc::calc::letter_octave_from_step(midi_pitch as f32)
        );
        let wave = Oscillator::new(frequency);
        let mut attack = BezPath::new();
        attack.line_to(Point::new(1.0, 1.0));
        let mut decay = BezPath::new();
        decay.move_to(Point::new(0.0, 1.0));
        decay.line_to(Point::new(0.5, 0.5));
        let mut release = BezPath::new();
        release.move_to(Point::new(0.0, 0.8));
        release.line_to(Point::new(0.3, 0.0));

        let envelope_config =
            EnvelopeConfiguration::asdr(Some(attack), Some(decay), 0.5, Some(release))?;

        let (envelope, is_playing_handle) = envelope_config.envelop(wave);

        Ok((
            envelope.amplify(velocity as f32 / 127.0 * 0.3),
            is_playing_handle,
        ))
    }
}
