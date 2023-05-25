use macroquad::prelude::*;
use std::fmt::Display;

use crate::{
    l3x::Direction,
    registers::Registers,
    sound::{TwelveToneNote, TwelveTonePitch},
};

const PITCHES: &[(u64, TwelveTonePitch)] = {
    use crate::sound::TwelveTone::*;
    &[
        (
            2,
            TwelveTonePitch {
                tone: CNat,
                octave: 4,
            },
        ),
        (
            3,
            TwelveTonePitch {
                tone: GNat,
                octave: 4,
            },
        ),
    ]
};

#[derive(Clone, Debug)]
pub struct Traveler {
    pub value: Registers, // TODO new number type representing registers directly
    pub location: IVec2,
    pub direction: Direction,
}

impl Traveler {
    pub fn direct(mut self, direction: Direction) -> Self {
        self.location += IVec2::from(direction);
        self.direction = direction;
        self
    }

    pub fn value(mut self, value: Registers) -> Self {
        self.value = value;
        self
    }

    pub fn mul(mut self, value: &Registers) -> Self {
        self.value *= value;
        self
    }

    pub fn step(mut self) -> Self {
        self.location += IVec2::from(self.direction);
        self
    }

    pub fn pitches(&self) -> Vec<TwelveToneNote> {
        let mut v = Vec::new();
        let mut ix = 0;
        'outer: for &(prime, pitch) in PITCHES {
            while let Some(&(test_prime, magnitude)) = self.value.0.get(ix) {
                ix += 1;
                if prime == test_prime {
                    v.push(pitch.vol(magnitude as f32));
                    continue 'outer;
                }
            }
        }
        log::debug!("{v:?}");
        v
    }
}

impl Display for Traveler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.value, self.direction)
    }
}
