use amuse::playback::{
    choir::Choir,
    voice::{NoteDuration, VoiceBuilder},
};
use muse::{
    node::LoadedInstrument,
    prelude::{serialization, InstrumentController, PreparedSampler, ToneGenerator},
    Note,
};
use std::{convert::TryInto, error::Error};
pub struct TestInstrument {
    basic_synth: LoadedInstrument,
}

impl ToneGenerator for TestInstrument {
    type CustomNodes = ();

    fn generate_tone(
        &mut self,
        note: Note,
        control: &mut InstrumentController<Self>,
    ) -> Result<PreparedSampler, anyhow::Error> {
        Ok(control.instantiate(&self.basic_synth, note)?)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let instrument: LoadedInstrument<()> =
        ron::from_str::<serialization::Instrument>(include_str!("support/basic_synth.ron"))?
            .try_into()?;
    let voice = VoiceBuilder::new(instrument)
        .poly(|p| {
            p.part(|p| p.play(Note::new(64., 80)).hold_for(NoteDuration::whole()))
                .part(|p| p.play(Note::new(60., 80)).hold_for(NoteDuration::whole()))
        })
        .play(Note::new(64., 80))
        .hold_for(NoteDuration::whole())
        .release()
        .play(Note::new(62., 80))
        .hold_for(NoteDuration::whole().dotted())
        .release()
        .play(Note::new(60., 80))
        .hold_for(NoteDuration::whole())
        .build();
    let choir = Choir::new(vec![voice]);
    choir.play()?;

    Ok(())
}
