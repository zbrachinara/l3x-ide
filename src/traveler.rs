use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Mul, MulAssign},
    str::FromStr,
};

use itertools::{merge_join_by, Itertools};
use macroquad::prelude::*;
use num_bigint::BigUint;
use prime_factorization::Factorization;

use crate::l3x::Direction;

/// Represents the value of a traveler or a cell. If the value of the register is 1, then the
/// vector in this struct is empty. Otherwise, it is a list of (p, pow), where p is a prime,
/// representing `p_0 ^ pow_0 * p_1 ^ pow_1 * ... * p_n ^ pow_n`
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Registers(
    #[cfg(test)] pub Vec<(u64, u32)>,
    #[cfg(not(test))] Vec<(u64, u32)>,
);

impl Registers {
    pub const ONE: Self = Registers(vec![]);
}

impl TryFrom<BigUint> for Registers {
    type Error = ();

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        if value == BigUint::from(0u32) {
            Err(())
        } else if value == BigUint::from(1u32) {
            Ok(Self(vec![]))
        } else {
            let factorization = num_prime::nt_funcs::factorize(value); // TODO look into factorization alternatives
            factorization
                .into_iter() // btree into_iter guarantees order by key
                .map(|(factor, pow)| {
                    factor
                        .to_u64_digits()
                        .into_iter()
                        .exactly_one()
                        .map(|factor_u64| (factor_u64, pow as u32))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Self)
                .map_err(|_| ())
        }
    }
}

impl TryFrom<u64> for Registers {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            Err(())
        } else if value == 1 {
            Ok(Self(vec![]))
        } else {
            let factorization = Factorization::run(value); // TODO look into factorization alternatives
            let mut state = HashMap::new();
            for &e in factorization.factors.iter() {
                state.entry(e).and_modify(|v| *v += 1).or_insert(1);
            }
            let mut res: Vec<_> = state.into_iter().collect();
            res.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(Self(res))
        }
    }
}

impl FromStr for Registers {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(int) = s.parse::<u64>() {
            Registers::try_from(int)
        } else if let Ok(bigint) = s.parse::<BigUint>() {
            Registers::try_from(bigint)
        } else {
            Err(())
        }
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let u64_repr = self.0.iter().fold(BigUint::from(1u64), |st, &(base, pow)| {
            st * BigUint::from(base).pow(pow)
        });
        write!(f, "{u64_repr}")
    }
}

impl Registers {
    pub fn try_div(&self, divisor: &Self) -> Option<Self> {
        let result = merge_join_by(
            self.0.iter(),
            divisor.0.iter(),
            |(left_base, _), (right_base, _)| left_base.cmp(right_base),
        )
        .map(|cmp| match cmp {
            itertools::EitherOrBoth::Both(&(base, pow_left), &(_, pow_right)) => {
                pow_left.checked_sub(pow_right).map(|pow| (base, pow))
            }
            itertools::EitherOrBoth::Left(&val) => Some(val),
            itertools::EitherOrBoth::Right(_) => None,
        })
        .filter(|u| u.map(|(_, pow)| pow != 0).unwrap_or(true))
        .collect::<Option<Vec<_>>>()?;

        Some(Self(result))
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

impl MulAssign<&Registers> for Registers {
    fn mul_assign(&mut self, rhs: &Registers) {
        *self = &*self * rhs;
    }
}

impl MulAssign for Registers {
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs
    }
}

#[cfg(test)]
mod test_registers {
    use super::*;
    #[test]
    fn zero_and_one() {
        assert_eq!(Registers::try_from(0), Err(()));
        assert_eq!(Registers::try_from(1), Ok(Registers(vec![])));
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
    fn create() {
        assert_eq!(
            Registers::from_str("30"),
            Ok(Registers(vec![(2, 1), (3, 1), (5, 1)]))
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

    #[test]
    fn division() {
        let r1 = Registers(vec![(2, 1), (3, 1), (5, 1)]);
        let r2 = Registers(vec![(3, 1)]);
        assert_eq!(
            r1.try_div(&r2),
            Some(Registers(vec![(2, 1), (5, 1)])),
            "Factor should be allowed to divide a number"
        );
        assert_eq!(
            r2.try_div(&r1),
            None,
            "a number cannot divide a lesser number"
        );

        let r1 = Registers(vec![(2, 9), (3, 4), (5, 7), (7, 4)]);
        let r2 = Registers(vec![(2, 1), (3, 1), (7, 1)]);
        assert_eq!(
            r1.try_div(&r2),
            Some(Registers(vec![(2, 8), (3, 3), (5, 7), (7, 3)])),
            "Normal case"
        );
    }
}

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
}

impl Display for Traveler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.value, self.direction)
    }
}
