#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::Error;
use clap::Args;
use game_solver::{
    game::{Game, GameState, StateType},
    player::ImpartialPlayer,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, hash::Hash};
use thiserror::Error;

use crate::util::{cli::move_failable, move_natural::NaturalMove};

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Nim {
    heaps: Vec<usize>,
    move_count: usize,
    max_moves: usize,
}

type NimMove = NaturalMove<2>;

impl Nim {
    /// Create a new game of Nim with the given heaps,
    /// where heaps is a list of the number of objects in each heap.
    pub fn new(heaps: Vec<usize>) -> Self {
        Self {
            heaps: heaps.clone(),
            move_count: 0,
            // sum of all the heaps is the upper bound for the amount of moves
            max_moves: heaps.iter().sum::<usize>(),
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum NimMoveError {
    #[error("chosen heap {heap} is out of bounds of the amount of heaps {heap_count}.")]
    HeapOutOfBounds { heap: usize, heap_count: usize },
    #[error("can't remove {removal_count} when there is only {actual_count} in {heap}.")]
    TooManyObjectsRemoval {
        heap: usize,
        removal_count: usize,
        actual_count: usize,
    },
}

impl Game for Nim {
    /// where Move is a tuple of the heap index and the number of objects to remove
    type Move = NimMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    
    /// Define Nim as a zero-sum impartial game
    type Player = ImpartialPlayer;
    type MoveError = NimMoveError;

    const STATE_TYPE: Option<StateType> = Some(StateType::Normal);

    fn max_moves(&self) -> Option<usize> {
        Some(self.max_moves)
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        let [heap, amount] = m.0;
        // check for indexing OOB
        if heap >= self.heaps.len() {
            return Err(NimMoveError::HeapOutOfBounds {
                heap,
                heap_count: self.heaps.len(),
            });
        }

        // check for removing too many objects
        if amount > self.heaps[heap] {
            return Err(NimMoveError::TooManyObjectsRemoval {
                heap,
                removal_count: amount,
                actual_count: self.heaps[heap],
            });
        }

        self.heaps[heap] -= amount;
        self.move_count += 1;
        Ok(())
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();

        // loop through every heap and add every possible move
        for (i, &heap) in self.heaps.iter().enumerate() {
            for j in 1..=heap {
                moves.push(NaturalMove([i, j]));
            }
        }

        moves.into_iter()
    }

    fn state(&self) -> GameState<Self::Player> {
        Self::STATE_TYPE.unwrap().state(self)
    }

    fn player(&self) -> Self::Player {
        ImpartialPlayer::Next
    }
}

impl Display for Nim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, heap) in self.heaps.iter().enumerate() {
            writeln!(f, "Heap {i}: {heap}")?;
        }

        Ok(())
    }
}

/// Analyzes Nim.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct NimArgs {
    /// The configuration of the game. For example, 3,5,7
    /// creates a Nim game that has three heaps, where each
    /// heap has 3, 5, and 7 objects respectively
    configuration: String,
    /// Nim moves, ordered as x1-y1 x2-y2 ...
    #[arg(value_parser = clap::value_parser!(NimMove))]
    moves: Vec<NimMove>,
}

impl Default for NimArgs {
    fn default() -> Self {
        Self {
            configuration: "3,5,7".to_string(),
            moves: vec![],
        }
    }
}

impl TryFrom<NimArgs> for Nim {
    type Error = Error;

    fn try_from(args: NimArgs) -> Result<Self, Self::Error> {
        // parse the original configuration of the game from args
        // e.g. 3,5,7 for 3 heaps with 3, 5, and 7 objects respectively
        let config = args
            .configuration
            .split(',')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect::<Vec<_>>();

        // create a new game of Nim with the given configuration
        let mut game = Nim::new(config);

        // parse every move in args, e.g. 0-0 1-1 in args
        for nim_move in args.moves {
            move_failable(&mut game, &nim_move)?;
        }

        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_moves_is_heap_sum() {
        assert_eq!(Nim::new(vec![3, 5, 7]).max_moves(), Some(3 + 5 + 7));
    }
}
