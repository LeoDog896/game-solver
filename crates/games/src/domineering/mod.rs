#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::Error;
use array2d::Array2D;
use clap::Args;
use game_solver::game::{Game, GameState, Player, ZeroSumPlayer};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter},
    hash::Hash,
};
use thiserror::Error;

use crate::util::cli::move_failable;

#[derive(Clone, Hash, Eq, PartialEq, Debug, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical
}

impl Orientation {
    fn turn(&self) -> Orientation {
        match *self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Domineering<const WIDTH: usize, const HEIGHT: usize> {
    // TODO: bit array 2d
    /// True represents a square - true if empty, false otherwise
    board: Array2D<bool>,
    move_count: usize,
    /// The orientation the first player will play as.
    primary_orientation: Orientation,
}

impl<const WIDTH: usize, const HEIGHT: usize> Default for Domineering<WIDTH, HEIGHT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Domineering<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Self::new_orientation(Orientation::Vertical)
    }

    pub fn new_orientation(orientation: Orientation) -> Self {
        Self {
            board: Array2D::filled_with(true, WIDTH, HEIGHT),
            move_count: 0,
            primary_orientation: orientation
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum DomineeringMoveError {
    #[error("While no domino is present at {0}, player {1:?} can not move at {0} because a domino is in way of placement.")]
    BlockingAdjacent(DomineeringMove, ZeroSumPlayer),
    #[error("Player {1:?} can not move at {0} because a domino is already at {0}.")]
    BlockingCurrent(DomineeringMove, ZeroSumPlayer),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DomineeringMove(usize, usize);

impl Display for DomineeringMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Domineering<WIDTH, HEIGHT> {
    fn place(&mut self, m: &DomineeringMove, orientation: Orientation) -> Result<(), DomineeringMoveError> {
        match orientation {
            Orientation::Horizontal => {
                if m.0 == WIDTH - 1 {
                    return Err(DomineeringMoveError::BlockingAdjacent(
                        m.clone(),
                        self.player(),
                    ));
                }
                self.board.set(m.0, m.1, false).unwrap();
                self.board.set(m.0 + 1, m.1, false).unwrap();
            },
            Orientation::Vertical => {
                if m.1 == HEIGHT - 1 {
                    return Err(DomineeringMoveError::BlockingAdjacent(
                        m.clone(),
                        self.player(),
                    ));
                }
                self.board.set(m.0, m.1, false).unwrap();
                self.board.set(m.0, m.1 + 1, false).unwrap();
            }
        };
        
        Ok(())
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Game for Domineering<WIDTH, HEIGHT> {
    type Move = DomineeringMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    type Player = ZeroSumPlayer;
    type MoveError = DomineeringMoveError;

    fn max_moves(&self) -> Option<usize> {
        Some(WIDTH * HEIGHT)
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
        if *self.board.get(m.0, m.1).unwrap() {
            self.place(m, if self.player() == ZeroSumPlayer::One {
                self.primary_orientation
            } else {
                self.primary_orientation.turn()
            })?;

            self.move_count += 1;
            Ok(())
        } else {
            Err(DomineeringMoveError::BlockingCurrent(
                m.clone(),
                self.player(),
            ))
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();
        let orientation = if self.player() == ZeroSumPlayer::One {
            self.primary_orientation
        } else {
            self.primary_orientation.turn()
        };

        match orientation {
            Orientation::Horizontal => {
                for i in 0..HEIGHT {
                    for j in 0..WIDTH - 1 {
                        if *self.board.get(j, i).unwrap() && *self.board.get(j + 1, i).unwrap() {
                            moves.push(DomineeringMove(j, i));
                        }
                    }
                }
            },
            Orientation::Vertical => {
                for i in 0..HEIGHT - 1 {
                    for j in 0..WIDTH {
                        if *self.board.get(j, i).unwrap() && *self.board.get(j, i + 1).unwrap() {
                            moves.push(DomineeringMove(j, i));
                        }
                    }
                }
            }
        }

        moves.into_iter()
    }

    fn state(&self) -> GameState<Self::Player> {
        if self.possible_moves().len() == 0 {
            GameState::Win(self.player().next())
        } else {
            GameState::Playable
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Display for Domineering<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
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

/// Analyzes Domineering.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct DomineeringArgs {
    moves: Vec<String>,
}

impl<const WIDTH: usize, const HEIGHT: usize> TryFrom<DomineeringArgs>
    for Domineering<WIDTH, HEIGHT>
{
    type Error = Error;

    fn try_from(args: DomineeringArgs) -> Result<Self, Self::Error> {
        let mut game = Domineering::new();

        // parse every move in args, e.g. 0-0 1-1 in args
        for arg in args.moves {
            let numbers: Vec<usize> = arg
                .split('-')
                .map(|num| num.parse::<usize>().expect("Not a number!"))
                .collect();

            move_failable(&mut game, &DomineeringMove(numbers[0], numbers[1]))?;
        }

        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use game_solver::move_scores;

    use super::*;

    /// Get the winner of a generic configuration of domineering
    fn winner<const WIDTH: usize, const HEIGHT: usize>(orientation: Orientation) -> Option<ZeroSumPlayer> {
        let game = Domineering::<WIDTH, HEIGHT>::new_orientation(orientation);
        let mut move_scores = move_scores(&game, &mut HashMap::new())
            .collect::<Result<Vec<_>, DomineeringMoveError>>()
            .unwrap();

        if move_scores.is_empty() {
            None
        } else {
            move_scores.sort_by_key(|m| m.1);
            move_scores.reverse();
            if move_scores[0].1 > 0 {
                Some(ZeroSumPlayer::One)
            } else {
                Some(ZeroSumPlayer::Two)
            }
        }
    }

    #[test]
    fn test_wins() {
        assert_eq!(winner::<5, 5>(Orientation::Horizontal), Some(ZeroSumPlayer::Two));
        assert_eq!(winner::<4, 4>(Orientation::Horizontal), Some(ZeroSumPlayer::One));
        assert_eq!(winner::<3, 3>(Orientation::Horizontal), Some(ZeroSumPlayer::One));
        assert_eq!(winner::<13, 2>(Orientation::Horizontal), Some(ZeroSumPlayer::Two));
        assert_eq!(winner::<11, 2>(Orientation::Horizontal), Some(ZeroSumPlayer::One));
    }

    #[test]
    fn test_domineering() {
        let game = Domineering::<5, 5>::new_orientation(Orientation::Horizontal);
        let mut move_scores = move_scores(&game, &mut HashMap::new())
            .collect::<Result<Vec<_>, DomineeringMoveError>>()
            .unwrap();

        assert_eq!(move_scores.len(), game.possible_moves().len());

        move_scores.sort();

        let mut current_scores = vec![
            (DomineeringMove(3, 4), -13),
            (DomineeringMove(0, 4), -13),
            (DomineeringMove(3, 3), -13),
            (DomineeringMove(2, 3), -13),
            (DomineeringMove(1, 3), -13),
            (DomineeringMove(0, 3), -13),
            (DomineeringMove(3, 2), -13),
            (DomineeringMove(0, 2), -13),
            (DomineeringMove(3, 1), -13),
            (DomineeringMove(2, 1), -13),
            (DomineeringMove(1, 1), -13),
            (DomineeringMove(0, 1), -13),
            (DomineeringMove(3, 0), -13),
            (DomineeringMove(0, 0), -13),
            (DomineeringMove(2, 4), -15),
            (DomineeringMove(1, 4), -15),
            (DomineeringMove(2, 2), -15),
            (DomineeringMove(1, 2), -15),
            (DomineeringMove(2, 0), -15),
            (DomineeringMove(1, 0), -15),
        ];

        current_scores.sort();

        assert_eq!(move_scores, current_scores);
    }
}
