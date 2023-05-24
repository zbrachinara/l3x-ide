use std::fmt::Display;
use macroquad::prelude::*;

use crate::{l3x::Direction, registers::Registers, sound::{TwelveTonePitch, TwelveToneNote}};

const PITCHES: [TwelveTonePitch;0] = [];

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
        todo!()
    }
}

impl Display for Traveler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.value, self.direction)
    }
}
