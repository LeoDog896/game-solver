#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::{anyhow, Error};
use clap::Args;
use game_solver::{
    game::{Game, GameState},
    player::{PartizanPlayer, Player},
};
use itertools::Itertools;
use ndarray::{iter::IndexedIter, ArrayD, Dim, Dimension, IntoDimension, IxDyn, IxDynImpl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::{
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    iter::FilterMap, str::FromStr,
};

use crate::util::move_failable;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum Square {
    X,
    O,
}

impl Square {
    fn to_player(self) -> PartizanPlayer {
        match self {
            Self::X => PartizanPlayer::Left,
            Self::O => PartizanPlayer::Right,
        }
    }

    fn from_player(player: PartizanPlayer) -> Square {
        match player {
            PartizanPlayer::Left => Self::X,
            PartizanPlayer::Right => Self::O,
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct TicTacToe {
    dim: usize,
    size: usize,
    /// True represents a square that has not been eaten
    board: ArrayD<Option<Square>>,
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

impl FromStr for TicTacToeMove {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers: Result<Vec<usize>, Self::Err> = s
            .split('-')
            .map(|num| num.parse::<usize>().map_err(|_| anyhow!("Not a number!")))
            .collect();
        
        Ok(Self(numbers?.into_dimension()))
    }
}

impl TryFrom<TicTacToeArgs> for TicTacToe {
    type Error = Error;

    fn try_from(value: TicTacToeArgs) -> Result<Self, Self::Error> {
        let mut game = TicTacToe::new(value.dimensions, value.size);

        // parse every move in args, e.g. 0-0 1-1 in args
        for arg in value.moves {
            move_failable(&mut game, &TicTacToeMove::from_str(&arg)?)?;
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
        let board = ArrayD::from_elem(IxDyn(&vec![size; dim]), None);

        Self {
            dim,
            size,
            board,
            move_count: 0,
        }
    }

    /// Returns the square on this winning line.
    fn winning_line(&self, point: &Dim<IxDynImpl>, offset: &[i32]) -> Option<Square> {
        let square = self.board.get(point).unwrap();

        if square.is_none() {
            return None;
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

        if n >= self.size {
            *square
        } else {
            None
        }
    }
}

impl Game for TicTacToe {
    type Move = TicTacToeMove;
    type Iter<'a> = FilterMap<
        IndexedIter<'a, Option<Square>, Dim<IxDynImpl>>,
        fn((Dim<IxDynImpl>, &Option<Square>)) -> Option<Self::Move>,
    >;
    type Player = PartizanPlayer;
    type MoveError = TicTacToeMoveError;

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
            if square.is_none() {
                continue;
            }

            let point = index.into_dimension();
            for offset in offsets(&point, self.size) {
                if let Some(square) = self.winning_line(&point, &offset) {
                    return GameState::Win(square.to_player());
                }
            }
        }

        GameState::Playable
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        if self.board.get(m.0.clone()).unwrap().is_none() {
            let square = Square::from_player(self.player());

            *self.board.get_mut(m.0.clone()).unwrap() = Some(square);
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
                if square.is_none() {
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

        let mut best_non_winning_game: Option<Self> = None;

        for m in &mut self.possible_moves() {
            let mut new_self = self.clone();
            new_self.make_move(&m)?;
            match new_self.state() {
                GameState::Playable => continue,
                GameState::Tie => best_non_winning_game = Some(new_self),
                GameState::Win(winning_player) => {
                    if winning_player == self.player().turn() {
                        return Ok(Some(new_self));
                    } else if best_non_winning_game.is_none() {
                        best_non_winning_game = Some(new_self)
                    }
                }
            };
        }

        Ok(best_non_winning_game)
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

impl Debug for TicTacToe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_solver::{move_scores, GameSolveError};
    use std::collections::HashMap;

    fn move_scores_unwrapped(game: &TicTacToe) -> Vec<(TicTacToeMove, isize)> {
        move_scores(game, &mut HashMap::new(), None)
            .collect::<Result<Vec<_>, GameSolveError<TicTacToe>>>()
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
