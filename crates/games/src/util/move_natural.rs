// TODO: make generic as some n-tuple move (macro generation / dynamic type?)

use std::{fmt::Display, iter, str::FromStr};

use itertools::Itertools;

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct NaturalMove<const LENGTH: usize>(pub [usize; LENGTH]);

impl<const LENGTH: usize> FromStr for NaturalMove<LENGTH> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        assert!(LENGTH > 0, "Length must be greater than 0");

        let numbers = s.split("-").collect::<Vec<_>>();

        if numbers.len() != LENGTH {
            return Err(
                format!(
                    "Must be {} numbers separated by a hyphen ({})",
                    LENGTH,
                    iter::repeat("x").take(LENGTH).join("-")
                )
            );
        }

        let numbers = numbers.iter()
            .map(|num| num.parse::<usize>())
            .collect::<Vec<_>>();

        if let Some((position, _)) = numbers.iter().find_position(|x| x.is_err()) {
            let ordinal = ordinal::Ordinal(position + 1).to_string();
            
            return Err(format!("The {} number is not a number.", ordinal));
        }
        
        numbers.iter().map(|x| x.clone().unwrap()).collect::<Vec<_>>().try_into()
            .map_err(|_| "Could not convert Vec to fixed array; this is a bug.".to_string())
            .map(|x| NaturalMove(x))
    }
}

impl<const LENGTH: usize> Display for NaturalMove<LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join("-"))
    }
}
