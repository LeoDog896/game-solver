// TODO: make generic as some n-tuple move (macro generation / dynamic type?)

use std::{fmt::Display, str::FromStr};

use itertools::Itertools;

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct ReversiMove(pub (usize, usize));

impl FromStr for ReversiMove {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers = s.split("-").collect::<Vec<_>>();

        if numbers.len() != 2 {
            return Err("Must be two numbers separated by a hyphen (x-y), i.e. 2-6".to_string());
        }

        let numbers = numbers.iter()
            .map(|num| num.parse::<usize>())
            .collect::<Vec<_>>();

        if let Some((position, _)) = numbers.iter().find_position(|x| x.is_err()) {
            let position = if position == 0 {
                "first"
            } else {
                "second"
            };
            
            return Err(format!("The {} number is not a number.", position));
        }
        
        Ok(ReversiMove((
            numbers[0].clone().unwrap(),
            numbers[1].clone().unwrap()
        )))
    }
}

impl Display for ReversiMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0.0, self.0.1)
    }
}
