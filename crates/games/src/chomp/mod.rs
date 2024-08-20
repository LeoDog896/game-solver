#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::Error;
use array2d::Array2D;
use clap::Args;
use game_solver::game::{Game, GameState, Player, ZeroSumPlayer};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

use crate::util::{cli::move_failable, move_natural::NaturalMove};

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Chomp {
    width: usize,
    height: usize,
    // TODO: bit array 2d
    /// True represents a square that has not been eaten
    board: Array2D<bool>,
    move_count: usize,
}

impl Chomp {
    pub fn new(width: usize, height: usize) -> Self {
        let mut board = Array2D::filled_with(true, width, height);
        board.set(0, height - 1, false).unwrap();

        Self {
            width,
            height,
            board,
            move_count: 0,
        }
    }
}

/// Analyzes Chomp.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ChompArgs {
    /// The width of the game
    #[arg(short, long, default_value_t = 6)]
    width: usize,
    /// The height of the game
    #[arg(short, long, default_value_t = 4)]
    height: usize,
    /// Chomp moves, ordered as x1-y1 x2-y2 ...
    #[arg(value_parser = clap::value_parser!(ChompMove))]
    moves: Vec<ChompMove>,
}

impl Default for ChompArgs {
    fn default() -> Self {
        Self {
            width: 6,
            height: 4,
            moves: vec![],
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum ChompMoveError {
    #[error("position {0:?} is already filled.")]
    ValueAlreadyFilled(ChompMove),
}

pub type ChompMove = NaturalMove<2>;

impl Game for Chomp {
    type Move = ChompMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    type Player = ZeroSumPlayer;
    type MoveError = ChompMoveError;

    fn max_moves(&self) -> Option<usize> {
        Some(self.width * self.height)
    }

    fn player(&self) -> Self::Player {
        if self.move_count % 2 == 0 {
            ZeroSumPlayer::One
        } else {
            ZeroSumPlayer::Two
        }
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        if *self.board.get(m.0[0], m.0[1]).unwrap() {
            for i in m.0[0]..self.width {
                for j in 0..=m.0[1] {
                    self.board.set(i, j, false).unwrap();
                }
            }
            self.move_count += 1;
            Ok(())
        } else {
            Err(ChompMoveError::ValueAlreadyFilled(*m))
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();
        for i in (0..self.height).rev() {
            for j in 0..self.width {
                if *self.board.get(j, i).unwrap() {
                    moves.push(NaturalMove([j, i]));
                }
            }
        }
        moves.into_iter()
    }

    fn state(&self) -> GameState<Self::Player> {
        if self.move_count == self.width * self.height {
            GameState::Win(self.player().next())
        } else {
            GameState::Playable
        }
    }
}

impl Display for Chomp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for i in 0..self.height {
            for j in 0..self.width {
                if *self.board.get(j, i).unwrap() {
                    write!(f, "X")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl TryFrom<ChompArgs> for Chomp {
    type Error = Error;

    fn try_from(args: ChompArgs) -> Result<Self, Self::Error> {
        let mut game = Chomp::new(args.width, args.height);

        // parse every move in args, e.g. 0-0 1-1 in args
        for arg in args.moves {
            move_failable(&mut game, &arg)?;
        }

        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use game_solver::move_scores;

    use super::*;

    #[test]
    fn test_chomp() {
        let game = Chomp::new(6, 4);
        let mut move_scores = move_scores(&game, &mut HashMap::new())
            .collect::<Result<Vec<_>, ChompMoveError>>()
            .unwrap();
        move_scores.sort();

        let mut new_scores = vec![
            (NaturalMove([2, 2]), 13),
            (NaturalMove([5, 0]), -12),
            (NaturalMove([4, 0]), -12),
            (NaturalMove([3, 0]), -12),
            (NaturalMove([2, 0]), -12),
            (NaturalMove([0, 0]), -12),
            (NaturalMove([5, 1]), -12),
            (NaturalMove([4, 1]), -12),
            (NaturalMove([3, 1]), -12),
            (NaturalMove([2, 1]), -12),
            (NaturalMove([0, 1]), -12),
            (NaturalMove([5, 2]), -12),
            (NaturalMove([4, 2]), -12),
            (NaturalMove([3, 2]), -12),
            (NaturalMove([5, 3]), -12),
            (NaturalMove([1, 0]), -16),
            (NaturalMove([1, 1]), -16),
            (NaturalMove([1, 2]), -16),
            (NaturalMove([4, 3]), -16),
            (NaturalMove([3, 3]), -16),
            (NaturalMove([2, 3]), -16),
            (NaturalMove([0, 2]), -22),
            (NaturalMove([1, 3]), -22),
        ];

        new_scores.sort();

        assert_eq!(move_scores, new_scores);
    }
}
