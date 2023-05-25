use itertools::{EitherOrBoth, Itertools};
use rodio::Source;
use single_value_channel::{Receiver, Updater};

use std::{cmp::Ordering, ops::Add, time::Duration};

#[allow(dead_code)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TwelveTone {
    #[default]
    CNat = 0,
    CSharp = 1,
    DNat = 2,
    DSharp = 3,
    ENat = 4,
    FNat = 5,
    FSharp = 6,
    GNat = 7,
    GSharp = 8,
    ANat = 9,
    ASharp = 10,
    BNat = 11,
}

impl TwelveTone {
    #![allow(unused, non_upper_case_globals)]
    const CFlat: Self = Self::BNat;
    const DFlat: Self = Self::CSharp;
    const EFlat: Self = Self::DSharp;
    const FFlat: Self = Self::ENat;
    const GFlat: Self = Self::FSharp;
    const AFlat: Self = Self::GSharp;
    const BFlat: Self = Self::ASharp;

    fn hz_at_zero(self) -> f32 {
        match self {
            TwelveTone::CNat => 16.35,
            TwelveTone::CSharp => 17.32,
            TwelveTone::DNat => 18.35,
            TwelveTone::DSharp => 19.45,
            TwelveTone::ENat => 20.60,
            TwelveTone::FNat => 21.83,
            TwelveTone::FSharp => 23.12,
            TwelveTone::GNat => 24.50,
            TwelveTone::GSharp => 25.96,
            TwelveTone::ANat => 27.50,
            TwelveTone::ASharp => 29.14,
            TwelveTone::BNat => 30.87,
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct TwelveTonePitch {
    pub tone: TwelveTone,
    pub octave: i8,
}

impl PartialOrd for TwelveTonePitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.octave.partial_cmp(&other.octave) {
            Some(Ordering::Equal) => self.tone.partial_cmp(&other.tone),
            ord => ord,
        }
    }
}

impl Ord for TwelveTonePitch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl TwelveTonePitch {
    fn hz(self) -> f32 {
        self.tone.hz_at_zero() * (2f32).powi(self.octave as i32)
    }

    pub fn vol(self, volume: f32) -> TwelveToneNote {
        TwelveToneNote {
            pitch: self,
            volume,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TwelveToneNote {
    pitch: TwelveTonePitch,
    volume: f32,
}

impl TwelveToneNote {
    fn hz(&self) -> f32 {
        self.pitch.hz()
    }
}

struct PlayState {
    samples_passed: u32,
    sample_rate: u32,
}

impl Default for PlayState {
    fn default() -> Self {
        Self {
            samples_passed: Default::default(),
            sample_rate: 12500,
        }
    }
}

impl PlayState {
    /// Advances self by the given sample rate and returns the fraction of tau which corresponds to
    /// the current advancement through the period of the wave (if i had more time I would've
    /// written a shorter letter).
    fn advance(&mut self) -> f32 {
        self.samples_passed += 1;
        if self.samples_passed > self.sample_rate {
            self.samples_passed = 0;
        }
        self.time()
    }

    fn time(&self) -> f32 {
        (self.samples_passed as f32) / (self.sample_rate as f32) * std::f32::consts::TAU
    }
}

impl From<u32> for PlayState {
    fn from(value: u32) -> Self {
        Self {
            sample_rate: value,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Chord {
    pub pitches: Vec<TwelveToneNote>,
    pub volume: f32,
}

impl Add<Chord> for Chord {
    type Output = Chord;

    fn add(mut self, mut rhs: Chord) -> Self::Output {
        let volume = self.volume.max(rhs.volume);
        self.pitches.sort_unstable_by_key(|note| note.pitch);
        rhs.pitches.sort_unstable_by_key(|note| note.pitch);

        let pitches = self
            .pitches
            .into_iter()
            .merge_join_by(rhs.pitches.into_iter(), |a, b| a.pitch.cmp(&b.pitch))
            .map(|it| match it {
                EitherOrBoth::Both(a, b) => TwelveToneNote {
                    pitch: a.pitch,
                    volume: a.volume + b.volume,
                },
                EitherOrBoth::Left(pitch) | EitherOrBoth::Right(pitch) => pitch,
            })
            .collect();

        Chord { pitches, volume }
    }
}

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
