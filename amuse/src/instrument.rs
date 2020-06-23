use rodio::source::Source;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub struct VirtualInstrument {
    cached_notes:
        HashMap<u8, HashMap<Loudness, rodio::source::Buffered<rodio::Decoder<BufReader<File>>>>>,
    playing_notes: Vec<PlayingNote>,
    device: rodio::Device,
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

fn load_note(
    midi_pitch: u8,
    loudness: Loudness,
) -> Result<rodio::source::Buffered<rodio::Decoder<BufReader<File>>>, anyhow::Error> {
    let octave = (midi_pitch - 8) / 12;
    let note = midi_pitch - octave * 12 - 8;

    let note = match note {
        0 => "Ab",
        1 => "A",
        2 => "Bb",
        3 => "B",
        4 => "C",
        5 => "Db",
        6 => "D",
        7 => "Eb",
        8 => "E",
        9 => "F",
        10 => "Gb",
        11 => "G",
        _ => unreachable!(),
    };

    let loudness = match loudness {
        Loudness::Fortissimo => "ff",
        Loudness::MezzoForte => "mf",
        Loudness::Pianissimo => "pp",
    };

    println!("Playing note {}{} loudness {}", note, octave, loudness);
    let file = File::open(&format!(
        "/home/ecton/steinway/Piano.{}.{}{}.ogg",
        loudness, note, octave
    ))?;

    Ok(rodio::Decoder::new(BufReader::new(file))?.buffered())
}

impl VirtualInstrument {
    pub fn new() -> Self {
        let device = rodio::default_output_device().unwrap();
        Self {
            device,
            playing_notes: Vec::new(),
            cached_notes: HashMap::new(),
        }
    }

    pub fn play_note(&mut self, pitch: u8, velocity: u8) -> Result<(), anyhow::Error> {
        let source = self.cache_note(pitch, velocity)?;
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
        self.playing_notes.retain(|note| {
            if note.pitch == pitch {
                note.sink.stop();
                false
            } else {
                true
            }
        });
    }

    // fn cache_note(
    //     &mut self,
    //     midi_pitch: u8,
    //     loudness: Loudness,
    // ) -> Result<rodio::source::Buffered<rodio::Decoder<BufReader<File>>>, anyhow::Error> {
    //     let notes_by_loudness = self
    //         .cached_notes
    //         .entry(midi_pitch)
    //         .or_insert_with(HashMap::new);
    //     if let Some(note) = notes_by_loudness.get(&loudness) {
    //         Ok(note.clone())
    //     } else {
    //         let note = load_note(midi_pitch, loudness)?;
    //         notes_by_loudness.insert(loudness, note.clone());
    //         Ok(note)
    //     }
    // }

    fn cache_note(
        &mut self,
        midi_pitch: u8,
        velocity: u8,
    ) -> Result<rodio::source::Amplify<rodio::source::SineWave>, anyhow::Error> {
        // A4 = 440hz, A4 = 69
        let frequency = pitch_calc::calc::hz_from_step(midi_pitch as f32);
        println!(
            "Playing {}hz, {:?}",
            frequency,
            pitch_calc::calc::letter_octave_from_step(midi_pitch as f32)
        );
        let wave = rodio::source::SineWave::new(frequency as u32);

        Ok(wave.amplify(velocity as f32 / 127.0 * 0.6))
    }
}
