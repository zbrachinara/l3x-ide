use std::{collections::HashMap, fmt::Display};

use macroquad::prelude::*;
use prime_factorization::Factorization;

#[derive(PartialEq, Eq, Debug)]
pub struct Registers(pub HashMap<u64, u32>);

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
            Ok(Self(state))
        }
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let u64_repr = self
            .0
            .iter()
            .fold(0, |st, (&base, &pow)| st + base.pow(pow));

        write!(f, "{u64_repr}")
    }
}

#[cfg(test)]
mod test_registers {
    use super::*;
    #[test]
    fn test_zero_and_one() {
        assert_eq!(Registers::try_from(0), Err(()));
        assert_eq!(
            Registers::try_from(1),
            Ok(Registers([(1, 1)].into_iter().collect()))
        );
    }

    #[test]
    fn test_primes() {
        // very large
        assert_eq!(
            Registers::try_from(18_446_744_073_709_551_557),
            Ok(Registers(
                [(18_446_744_073_709_551_557, 1)].into_iter().collect()
            ))
        )
    }
}

pub struct Traveler {
    value: Registers, // TODO new number type representing registers directly
    position: UVec2,
}
