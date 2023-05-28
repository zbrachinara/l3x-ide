use crate::sound::chord::{Chord, TwelveToneNote};

extern "C" {
    fn wasm_sound_drop_all();
    fn wasm_sound_play(frequency: f32, volume: f32);
}

#[derive(Default)]
pub struct Updater {
    previous: Option<Chord>,
}

impl Updater {
    pub fn update(&mut self, chord: Chord) -> Result<(), ()> {
        let needs_update = self.previous.as_ref().map(|p| p != &chord).unwrap_or(true);

        if needs_update {
            self.previous = Some(chord.clone());
            unsafe { wasm_sound_drop_all() };

            let raw_volume = chord
                .pitches
                .iter()
                .map(|TwelveToneNote { volume, .. }| *volume)
                .sum::<f32>();

            for TwelveToneNote { pitch, volume } in chord.pitches {
                let vol = volume / raw_volume * chord.volume;
                unsafe { wasm_sound_play(pitch.hz(), vol) }
            }
        }
        Ok(())
    }
}
