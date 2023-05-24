use macroquad::prelude::*;
use std::fmt::Display;

use crate::{
    l3x::Direction,
    registers::Registers,
    sound::{TwelveToneNote, TwelveTonePitch},
};

const PITCHES: &[TwelveTonePitch] = {
    use crate::sound::TwelveTone::*;
    &[
        TwelveTonePitch {
            tone: CNat,
            octave: 4,
        },
        TwelveTonePitch {
            tone: GNat,
            octave: 4,
        },
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
        self.value
            .0
            .iter()
            .zip(PITCHES)
            .map(|(&(_, magnitude), &pitch)| pitch.vol(magnitude as f32))
            .collect()
    }
}

impl Display for Traveler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.value, self.direction)
    }
}
