use std::{collections::HashMap, fmt::Display, ops::Mul, str::FromStr};

use itertools::merge_join_by;
use macroquad::prelude::*;
use prime_factorization::Factorization;

use crate::l3x::Direction;

#[derive(PartialEq, Eq, Debug)]
pub struct Registers(pub Vec<(u64, u32)>);

impl TryFrom<u64> for Registers {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            Err(())
        } else if value == 1 {
            Ok(Self([(1, 1)].into_iter().collect()))
        } else {
            let factorization = Factorization::run(value); // TODO look into factorization alternatives
            let mut state = HashMap::new();
            for &e in factorization.factors.iter() {
                state.entry(e).and_modify(|v| *v += 1).or_insert(1);
            }
            Ok(Self(state.into_iter().collect()))
        }
    }
}

impl FromStr for Registers {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u64>()
            .map_err(|_| ())
            .and_then(Registers::try_from)
        // todo!()
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let u64_repr = self.0.iter().fold(0, |st, &(base, pow)| st + base.pow(pow));
        write!(f, "{u64_repr}")
    }
}

impl Mul for &Registers {
    type Output = Registers;

    fn mul(self, rhs: Self) -> Self::Output {
        let result = merge_join_by(
            self.0.iter(),
            rhs.0.iter(),
            |(left_base, _), (right_base, _)| left_base.cmp(right_base),
        )
        .map(|cmp| match cmp {
            itertools::EitherOrBoth::Both((base_left, pow_left), (base_right, pow_right)) => {
                debug_assert_eq!(base_left, base_right);
                (*base_left, pow_left + pow_right)
            }
            itertools::EitherOrBoth::Left((a, b)) | itertools::EitherOrBoth::Right((a, b)) => {
                (*a, *b)
            }
        })
        .collect();

        Registers(result)
    }
}

#[duplicate::duplicate_item(
    left right left_ref right_ref;
    [Registers] [Registers] [&self] [&rhs];
    [Registers] [&Registers] [&self] [rhs];
    [&Registers] [Registers] [self] [&rhs];
)]
impl Mul<right> for left {
    type Output = Registers;

    fn mul(self, rhs: right) -> Self::Output {
        left_ref * right_ref
    }
}

#[cfg(test)]
mod test_registers {
    use super::*;
    #[test]
    fn zero_and_one() {
        assert_eq!(Registers::try_from(0), Err(()));
        assert_eq!(Registers::try_from(1), Ok(Registers(vec![(1, 1)])));
    }

    #[test]
    fn primes() {
        // very large
        assert_eq!(
            Registers::try_from(18_446_744_073_709_551_557),
            Ok(Registers(vec![(18_446_744_073_709_551_557, 1)]))
        )
    }

    #[test]
    fn multiplication() {
        let r1 = Registers(vec![(2, 1), (5, 1)]);
        let r2 = Registers(vec![(3, 1), (7, 1)]);
        assert_eq!(
            r1 * r2,
            Registers(vec![(2, 1), (3, 1), (5, 1), (7, 1)]),
            "appending disjoint registers together"
        );

        let r1 = Registers(vec![(2, 3), (3, 2), (7, 9)]);
        let r2 = Registers(vec![(2, 6), (3, 4), (5, 1)]);
        assert_eq!(
            r1 * r2,
            Registers(vec![(2, 9), (3, 6), (5, 1), (7, 9)]),
            "adding between registers 2 and 3"
        )
    }
}

pub struct Traveler {
    value: Registers, // TODO new number type representing registers directly
    position: UVec2,
    direction: Direction,
}
