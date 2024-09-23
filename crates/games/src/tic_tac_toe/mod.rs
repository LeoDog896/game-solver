#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::{anyhow, Error};
use clap::Args;
use game_solver::{
    game::{Game, GameState, StateType},
    player::{PartizanPlayer, Player},
};
use itertools::Itertools;
use ndarray::{iter::IndexedIter, ArrayD, Dim, Dimension, IntoDimension, IxDyn, IxDynImpl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    iter::FilterMap,
};

use crate::util::cli::move_failable;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum Square {
    Empty,
    X,
    O,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct TicTacToe {
    dim: usize,
    size: usize,
    /// True represents a square that has not been eaten
    board: ArrayD<Square>,
    move_count: usize,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TicTacToeMove(pub Dim<IxDynImpl>);

#[derive(Error, Debug, Clone)]
pub enum TicTacToeMoveError {
    #[error("the chosen move {0} is already filled")]
    NonEmptySquare(TicTacToeMove),
}

/// Analyzes Tic Tac Toe.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct TicTacToeArgs {
    /// The amount of dimensions in the game.
    dimensions: usize,
    /// The size of the board - i.e. with two dimensions
    /// and a size of three, the board would look like
    ///
    /// ```txt
    /// * * *
    /// * * *
    /// * * *
    /// ```
    size: usize,
    /// The moves to make in the game, by dimension and index in that dimension.
    moves: Vec<String>,
}

impl Default for TicTacToeArgs {
    fn default() -> Self {
        Self {
            dimensions: 2,
            size: 3,
            moves: vec![],
        }
    }
}

impl TryFrom<TicTacToeArgs> for TicTacToe {
    type Error = Error;

    fn try_from(value: TicTacToeArgs) -> Result<Self, Self::Error> {
        let mut game = TicTacToe::new(value.dimensions, value.size);

        // parse every move in args, e.g. 0-0 1-1 in args
        for arg in value.moves {
            let numbers: Result<Vec<usize>, Self::Error> = arg
                .split('-')
                .map(|num| num.parse::<usize>().map_err(|_| anyhow!("Not a number!")))
                .collect();

            move_failable(&mut game, &TicTacToeMove(numbers?.into_dimension()))?;
        }

        Ok(game)
    }
}

fn add_checked(a: Dim<IxDynImpl>, b: Vec<i32>) -> Option<Dim<IxDynImpl>> {
    let mut result = a;
    for (i, j) in result.as_array_view_mut().iter_mut().zip(b.iter()) {
        if *i as i32 + *j < 0 {
            return None;
        }

        *i = ((*i) as i32 + *j).try_into().unwrap();
    }

    Some(result)
}

impl TicTacToe {
    fn new(dim: usize, size: usize) -> Self {
        // we want [SIZE; dim] but dim isn't a const - we have to get the slice from a vec
        let board = ArrayD::from_elem(IxDyn(&vec![size; dim]), Square::Empty);

        Self {
            dim,
            size,
            board,
            move_count: 0,
        }
    }

    fn winning_line(&self, point: &Dim<IxDynImpl>, offset: &[i32]) -> bool {
        let square = self.board.get(point).unwrap();

        if *square == Square::Empty {
            return false;
        }

        let mut n = 1;

        let mut current = point.clone();
        while let Some(new_current) = add_checked(current.clone(), offset.to_owned()) {
            current = new_current;
            if self.board.get(current.clone()) == Some(square) {
                n += 1;
            } else {
                break;
            }
        }
        let mut current = point.clone();

        while let Some(new_current) =
            add_checked(current.clone(), offset.iter().map(|x| -x).collect())
        {
            current = new_current;
            if self.board.get(current.clone()) == Some(square) {
                n += 1;
            } else {
                break;
            }
        }

        n >= self.size
    }
}

impl Game for TicTacToe {
    type Move = TicTacToeMove;
    type Iter<'a> = FilterMap<
        IndexedIter<'a, Square, Dim<IxDynImpl>>,
        fn((Dim<IxDynImpl>, &Square)) -> Option<Self::Move>,
    >;
    type Player = PartizanPlayer;
    type MoveError = TicTacToeMoveError;

    const STATE_TYPE: Option<StateType> = None;

    fn max_moves(&self) -> Option<usize> {
        Some(self.size.pow(self.dim as u32))
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn state(&self) -> GameState<Self::Player> {
        // check if tie
        if Some(self.move_count()) == self.max_moves() {
            return GameState::Tie;
        }

        // check every move
        for (index, square) in self.board.indexed_iter() {
            if square == &Square::Empty {
                continue;
            }

            let point = index.into_dimension();
            for offset in offsets(&point, self.size) {
                if self.winning_line(&point, &offset) {
                    return GameState::Win(self.player().next());
                }
            }
        }

        GameState::Playable
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        if *self.board.get(m.0.clone()).unwrap() == Square::Empty {
            let square = if self.player() == PartizanPlayer::Left {
                Square::X
            } else {
                Square::O
            };

            *self.board.get_mut(m.0.clone()).unwrap() = square;
            self.move_count += 1;
            Ok(())
        } else {
            Err(TicTacToeMoveError::NonEmptySquare(m.clone()))
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        self.board
            .indexed_iter()
            .filter_map(move |(index, square)| {
                if square == &Square::Empty {
                    Some(TicTacToeMove(index))
                } else {
                    None
                }
            })
    }

    fn find_immediately_resolvable_game(&self) -> Result<Option<Self>, Self::MoveError> {
        // check if the amount of moves is less than (size * 2) - 1
        // if it is, then it's impossible to win
        if self.move_count + 1 < self.size * 2 - 1 {
            return Ok(None);
        }

        for m in &mut self.possible_moves() {
            let mut new_self = self.clone();
            new_self.make_move(&m)?;
            if let GameState::Win(_) = new_self.state() {
                return Ok(Some(new_self));
            }
        }

        Ok(None)
    }

    fn player(&self) -> Self::Player {
        if self.move_count % 2 == 0 {
            PartizanPlayer::Left
        } else {
            PartizanPlayer::Right
        }
    }
}

fn offsets(dim: &Dim<IxDynImpl>, size: usize) -> Vec<Vec<i32>> {
    let values = (-1i32..=1).collect::<Vec<_>>(); // every offset
    let permutations = itertools::repeat_n(values.iter(), dim.ndim()).multi_cartesian_product();

    permutations
        .map(|permutation| {
            // dereference the permutation
            permutation.iter().map(|x| **x).collect::<Vec<_>>()
        })
        .filter(|permutation| {
            // filter out the permutations that are all 0
            permutation.iter().any(|x| *x != 0)
        })
        .filter(|permutation| {
            // filter out the permutations that are out of bounds [0, size)
            let result = add_checked(dim.clone(), permutation.to_owned());
            result.map_or(false, |result| {
                result.as_array_view().iter().all(|x| *x < size)
            })
        })
        .collect()
}

impl Display for TicTacToeMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0.as_array_view().as_slice().unwrap())
    }
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (index, square) in self.board.indexed_iter() {
            writeln!(f, "{:?} @ {}", square, TicTacToeMove(index))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_solver::move_scores;
    use std::collections::HashMap;

    fn move_scores_unwrapped(game: &TicTacToe) -> Vec<(TicTacToeMove, isize)> {
        move_scores(game, &mut HashMap::new())
            .collect::<Result<Vec<_>, TicTacToeMoveError>>()
            .unwrap()
    }

    fn best_moves(game: &TicTacToe) -> Option<Dim<IxDynImpl>> {
        move_scores_unwrapped(game)
            .iter()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(m, _)| m.0.clone())
    }

    #[test]
    fn test_middle_move() {
        let mut game = TicTacToe::new(2, 3);
        game.make_move(&TicTacToeMove(vec![0, 0].into_dimension()))
            .unwrap();

        let best_move = best_moves(&game).unwrap();

        assert_eq!(best_move, vec![1, 1].into_dimension());
    }

    #[test]
    fn test_always_tie() {
        let game = TicTacToe::new(2, 3);

        assert!(move_scores_unwrapped(&game)
            .iter()
            .all(|(_, score)| *score == 0));
    }

    #[test]
    fn test_win() {
        let mut game = TicTacToe::new(2, 3);

        game.make_move(&TicTacToeMove(vec![0, 2].into_dimension()))
            .unwrap(); // X
        game.make_move(&TicTacToeMove(vec![0, 1].into_dimension()))
            .unwrap(); // O
        game.make_move(&TicTacToeMove(vec![1, 1].into_dimension()))
            .unwrap(); // X
        game.make_move(&TicTacToeMove(vec![0, 0].into_dimension()))
            .unwrap(); // O
        game.make_move(&TicTacToeMove(vec![2, 0].into_dimension()))
            .unwrap(); // X

        assert!(game.state() == GameState::Win(PartizanPlayer::Left));
    }

    #[test]
    fn test_no_win() {
        let mut game = TicTacToe::new(2, 3);

        game.make_move(&TicTacToeMove(vec![0, 2].into_dimension()))
            .unwrap(); // X
        game.make_move(&TicTacToeMove(vec![0, 1].into_dimension()))
            .unwrap(); // O
        game.make_move(&TicTacToeMove(vec![1, 1].into_dimension()))
            .unwrap(); // X
        game.make_move(&TicTacToeMove(vec![0, 0].into_dimension()))
            .unwrap(); // O

        assert!(game.state() == GameState::Playable);
    }

    #[test]
    fn test_win_3d() {
        let mut game = TicTacToe::new(3, 3);

        game.make_move(&TicTacToeMove(vec![0, 0, 0].into_dimension()))
            .unwrap(); // X
        game.make_move(&TicTacToeMove(vec![0, 0, 1].into_dimension()))
            .unwrap(); // O
        game.make_move(&TicTacToeMove(vec![0, 1, 1].into_dimension()))
            .unwrap(); // X
        game.make_move(&TicTacToeMove(vec![0, 0, 2].into_dimension()))
            .unwrap(); // O
        game.make_move(&TicTacToeMove(vec![0, 2, 2].into_dimension()))
            .unwrap(); // X
        game.make_move(&TicTacToeMove(vec![0, 1, 0].into_dimension()))
            .unwrap(); // O
        game.make_move(&TicTacToeMove(vec![0, 2, 0].into_dimension()))
            .unwrap(); // X

        assert!(game.state() == GameState::Win(PartizanPlayer::Left));
    }

    #[test]
    fn test_always_tie_1d() {
        let game = TicTacToe::new(1, 3);

        assert!(move_scores_unwrapped(&game)
            .iter()
            .all(|(_, score)| *score == 0));
    }
}
