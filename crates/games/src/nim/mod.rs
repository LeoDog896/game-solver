#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::Error;
use clap::Args;
use game_solver::{
    game::{Game, GameState, Normal, NormalImpartial},
    player::ImpartialPlayer,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};
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

impl Normal for Nim {}
impl NormalImpartial for Nim {}
impl Game for Nim {
    /// where Move is a tuple of the heap index and the number of objects to remove
    type Move = NimMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;

    /// Define Nim as a zero-sum impartial game
    type Player = ImpartialPlayer;
    type MoveError = NimMoveError;

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
        <Self as Normal>::state(&self)
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

impl Debug for Nim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
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
    use std::collections::HashMap;

    use game_solver::{move_scores, CollectedMoves};
    use itertools::Itertools;

    use crate::util::move_score::best_move_score_testing;

    use super::*;

    fn play(nim: Nim) -> CollectedMoves<Nim> {
        move_scores(&nim, &mut HashMap::new(), None, &None).collect_vec()
    }

    #[test]
    fn max_moves_is_heap_sum() {
        assert_eq!(Nim::new(vec![3, 5, 7]).max_moves(), Some(3 + 5 + 7));
        assert_eq!(Nim::new(vec![0, 2, 2]).max_moves(), Some(2 + 2));
    }

    #[test]
    fn single_heap() {
        // p1 always wins for single-heap stacks
        // the score is equivalent to the stack, as we can make 1 move to win,
        // and score is always (max moves - moves made (+ 1 to account for ties))
        assert_eq!(best_move_score_testing(play(Nim::new(vec![7]))).1, 7);
        assert_eq!(best_move_score_testing(play(Nim::new(vec![4]))).1, 4);
        assert_eq!(best_move_score_testing(play(Nim::new(vec![20]))).1, 20);
    }

    #[test]
    fn empty_heap() {
        // unless the heaps have nothing, in which we cant play
        assert!(play(Nim::new(vec![0])).is_empty());
        assert!(play(Nim::new(vec![0, 0])).is_empty());
    }

    #[test]
    fn symmetrical_nim_wins() {
        // a loss in 4 moves: take 1, other player takes from other, take 1, other player takes from other
        assert_eq!(best_move_score_testing(play(Nim::new(vec![2, 2]))).1, -1);

        // generalize this for more cases:
        assert_eq!(best_move_score_testing(play(Nim::new(vec![6, 6]))).1, -1);
        assert_eq!(
            best_move_score_testing(play(Nim::new(vec![5, 5, 3, 3]))).1,
            -1
        );
        assert_eq!(best_move_score_testing(play(Nim::new(vec![7, 7]))).1, -1);
    }
}
