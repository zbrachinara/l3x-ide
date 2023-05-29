use std::time::Duration;

use rodio::{OutputStream, Source};
use single_value_channel::{NoReceiverError, Receiver};

use crate::sound::chord::{Chord, PlayState};

impl Chord {
    fn play_with_state(&self, state: &PlayState) -> f32 {
        self.pitches
            .iter()
            .copied()
            .map(|u| ((u.hz() * state.time()).sin() * u.volume, u.volume))
            .reduce(|(wv, vol), (wv_new, vol_new)| (wv + wv_new, vol + vol_new))
            .map(|(wv, vol)| wv / vol)
            .unwrap_or(0.0)
            * self.volume
    }
}

pub struct Updater {
    #[allow(unused)] // needs to exist so that the sound thread is allowed to live
    output: OutputStream,
    output_interface: single_value_channel::Updater<Chord>,
}

impl Default for Updater {
    fn default() -> Self {
        let (output, output_handle) = OutputStream::try_default().unwrap();
        let (output_interface, signals) = pitch_signals();
        output_handle
            .play_raw(signals)
            .expect("Could not begin sound engine");
        Self {
            output,
            output_interface,
        }
    }
}

impl Updater {
    pub fn update(&mut self, chord: Chord) -> Result<(), NoReceiverError<Chord>> {
        self.output_interface.update(chord)
    }
}

pub struct Signal {
    receiver: Receiver<Chord>,
    state: PlayState,
}

impl Iterator for Signal {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.advance();
        Some(self.receiver.latest().play_with_state(&self.state))
    }
}

impl Source for Signal {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.state.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn pitch_signals() -> (single_value_channel::Updater<Chord>, Signal) {
    let (receiver, sender) = single_value_channel::channel_starting_with(Default::default());
    let signal = Signal {
        receiver,
        state: Default::default(),
    };
    (sender, signal)
}
