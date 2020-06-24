use crate::oscillators::*;
use rodio::source::Source;

pub struct VirtualInstrument {
    playing_notes: Vec<PlayingNote>,
    device: rodio::Device,
    sustain: bool,
}

pub struct PlayingNote {
    pitch: u8,
    _velocity: u8,
    _sink: rodio::Sink,
    key_is_pressed: bool,
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

        let source = self.generate_note(pitch, velocity)?;
        let sink = rodio::Sink::new(&self.device);
        sink.append(source);
        self.playing_notes.push(PlayingNote {
            pitch,
            _velocity: velocity,
            _sink: sink,
            key_is_pressed: true,
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
                existing_note.key_is_pressed = false;
            }
        } else {
            self.playing_notes.retain(|note| note.pitch != pitch);
        }
    }

    pub fn set_sustain(&mut self, active: bool) {
        self.sustain = active;

        if !active {
            self.playing_notes.retain(|n| n.key_is_pressed);
        }
    }

    fn generate_note(
        &mut self,
        midi_pitch: u8,
        velocity: u8,
    ) -> Result<rodio::source::Amplify<Oscillator<Sawtooth>>, anyhow::Error> {
        // A4 = 440hz, A4 = 69
        let frequency = pitch_calc::calc::hz_from_step(midi_pitch as f32);
        println!(
            "Playing {}hz, {:?}",
            frequency,
            pitch_calc::calc::letter_octave_from_step(midi_pitch as f32)
        );
        let wave = Oscillator::new(frequency);

        Ok(wave.amplify(velocity as f32 / 127.0 * 0.3))
    }
}
