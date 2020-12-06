use muse::{
    node::Instantiatable,
    prelude::{ToneGenerator, VirtualInstrument},
    Note,
};

use crate::playback::voice::Voice;

use super::voice::{NoteDuration, VoiceCommand, VoiceSequence};

pub struct Choir<T>
where
    T: ToneGenerator + Clone + Instantiatable,
{
    voices: Vec<Voice<T>>,
}

impl<T> Choir<T>
where
    T: ToneGenerator + Clone + Instantiatable,
{
    pub fn new(voices: Vec<Voice<T>>) -> Self {
        Self { voices }
    }
}

impl<T> Choir<T>
where
    T: ToneGenerator + Clone + Instantiatable + 'static,
{
    pub fn play(&self) -> anyhow::Result<()> {
        let mut current_beat = NoteDuration::default();
        let beats_per_minute = 60f32;
        let mut voices = self
            .voices
            .iter()
            .filter_map(|voice| {
                let instrument =
                    VirtualInstrument::new_with_default_output(voice.instrument.clone()).ok()?;
                Some(ChoirVoice {
                    state: SequenceState::default(),
                    instrument,
                    voice,
                })
            })
            .collect::<Vec<ChoirVoice<T>>>();

        loop {
            println!("Playing beat {:?}", current_beat);
            let next_beat = voices.iter_mut().filter_map(|v| v.play(current_beat)).min();
            if let Some(next_beat) = next_beat {
                let remaining = (next_beat - current_beat).duration(beats_per_minute);
                spin_sleep::sleep(remaining);
                current_beat = next_beat;
            } else {
                break;
            }
        }

        Ok(())
    }
}

pub struct ChoirVoice<'a, T>
where
    T: ToneGenerator + Clone + Instantiatable,
{
    voice: &'a Voice<T>,
    instrument: VirtualInstrument<T>,
    state: SequenceState,
}

impl<'a, T> ChoirVoice<'a, T>
where
    T: ToneGenerator + Clone + Instantiatable + 'static,
{
    pub fn play(&mut self, current_beat: NoteDuration) -> Option<NoteDuration> {
        Self::play_sequence(
            &mut self.instrument,
            &self.voice.sequence,
            &mut self.state,
            current_beat,
        )
    }

    fn play_sequence(
        instrument: &mut VirtualInstrument<T>,
        sequence: &VoiceSequence,
        state: &mut SequenceState,
        current_beat: NoteDuration,
    ) -> Option<NoteDuration> {
        // If we're a poly step, we need to make sure we fully process its sequence before moving to the next step
        if let Some(current_step) = state.current_step {
            if let Some(poly_states) = &mut state.poly_state {
                if let VoiceCommand::Poly(sequences) = &sequence.steps[current_step].command {
                    let mut next_beat = None;
                    for (state, sequence) in poly_states.states.iter_mut().zip(sequences) {
                        if let Some(sequence_next_beat) =
                            Self::play_sequence(instrument, sequence, state, current_beat)
                        {
                            if next_beat.is_none() || next_beat.unwrap() < sequence_next_beat {
                                next_beat = Some(sequence_next_beat);
                            }
                        }
                    }

                    if next_beat.is_some() {
                        return next_beat;
                    }
                }
            }
        }

        // Move onto the next step
        let next_step_index = state.current_step.map(|s| s + 1).unwrap_or_default();
        if next_step_index >= sequence.steps.len() {
            return None;
        }

        let step = &sequence.steps[next_step_index];
        if step.beat <= current_beat {
            state.current_step = Some(next_step_index);
            match &step.command {
                VoiceCommand::Play(note) => {
                    instrument.play_note(*note).unwrap();
                    state.playing_note = Some(*note);
                }

                VoiceCommand::Release => {
                    if let Some(note) = state.playing_note {
                        instrument.stop_note(note.step() as u8);
                    }
                }
                VoiceCommand::Poly(parts) => {
                    let mut states = Vec::with_capacity(parts.len());
                    states.resize_with(parts.len(), Default::default);
                    state.poly_state = Some(PolyState { states })
                }
            }
        }

        Some(step.beat)
    }
}

#[derive(Default)]
pub struct SequenceState {
    current_step: Option<usize>,
    playing_note: Option<Note>,
    poly_state: Option<PolyState>,
}

pub struct PolyState {
    states: Vec<SequenceState>,
}
