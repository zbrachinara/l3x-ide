use std::time::Duration;

use rodio::Source;
use single_value_channel::{Receiver, Updater};

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

pub fn pitch_signals() -> (Updater<Chord>, Signal) {
    let (receiver, sender) = single_value_channel::channel_starting_with(Default::default());
    let signal = Signal {
        receiver,
        state: Default::default(),
    };
    (sender, signal)
}
