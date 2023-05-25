use macroquad::prelude::*;
use std::fmt::Display;

use crate::{
    l3x::Direction,
    registers::Registers,
    sound::{TwelveToneNote, TwelveTonePitch},
};

const PITCHES: &[(u64, TwelveTonePitch)] = {
    use crate::sound::pitch_from_str as p;
    &[
        (2, p("C", 4)),
        (3, p("G", 4)),
        (5, p("B", 4)),
        (7, p("D", 5)),
        (11, p("F", 4)),
        (13, p("A", 4)),
        (17, p("E", 5)),
        (19, p("G", -5))
    ]
};

#[derive(Clone, Debug)]
pub struct Traveler {
    pub value: Registers,
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
        'outer: for &(test_prime, magnitude) in &self.value.0 {
            while let Some(&(prime, pitch)) = PITCHES.get(ix) {
                ix += 1;
                if prime == test_prime {
                    v.push(pitch.vol(magnitude as f32));
                    continue 'outer;
                }
            }
        }
        v
    }
}

impl Display for Traveler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.value, self.direction)
    }
}
