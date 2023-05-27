use crate::sound::chord::Chord;

extern "C" {
    fn wasm_sound_drop_all();
    fn wasm_sound_play_a();
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
            if chord.pitches.is_empty() {
                unsafe { wasm_sound_drop_all() }
            } else {
                unsafe { wasm_sound_play_a() }
            }
        }
        Ok(())
    }
}
