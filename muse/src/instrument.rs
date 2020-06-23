use crate::oscillators::*;
use rodio::source::Source;

pub struct VirtualInstrument {
    playing_notes: Vec<PlayingNote>,
    device: rodio::Device,
    sustain: bool,
}

pub struct PlayingNote {
    pitch: u8,
    velocity: u8,
    sink: rodio::Sink,
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
        let source = self.generate_note(pitch, velocity)?;
        let sink = rodio::Sink::new(&self.device);
        sink.append(source);
        self.playing_notes.push(PlayingNote {
            pitch,
            velocity,
            sink,
        });

        Ok(())
    }

    pub fn stop_note(&mut self, pitch: u8) {
        if !self.sustain {
            self.playing_notes.retain(|note| {
                if note.pitch == pitch {
                    note.sink.stop();
                    false
                } else {
                    true
                }
            });
        }
    }

    pub fn set_sustain(&mut self, active: bool) {
        self.sustain = active;
        if !active {
            self.playing_notes.iter().for_each(|n| n.sink.stop());
            self.playing_notes.clear();
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
