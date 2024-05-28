//! Tic Tac Toe is a traditional two-player game played on a 3x3 grid.
//! For the sake of complexity, this allows simulating any n-dimensional 3-in-a-row game
//! with the same bounds as the traditional game.

pub mod cli;

use game_solver::game::{Game, ZeroSumPlayer};
use itertools::Itertools;
use ndarray::{iter::IndexedIter, ArrayD, Dim, Dimension, IntoDimension, IxDyn, IxDynImpl};

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    iter::FilterMap,
};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
enum Square {
    Empty,
    X,
    O,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct TicTacToe {
    dim: usize,
    size: usize,
    /// True represents a square that has not been eaten
    board: ArrayD<Square>,
    move_count: usize,
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

    fn won(&self) -> bool {
        // check every move
        for (index, square) in self.board.indexed_iter() {
            if square == &Square::Empty {
                continue;
            }

            let point = index.into_dimension();
            for offset in offsets(&point, self.size) {
                if self.winning_line(&point, &offset) {
                    return true;
                }
            }
        }

        false
    }
}

impl Game for TicTacToe {
    type Move = Dim<IxDynImpl>;
    type Iter<'a> = FilterMap<
        IndexedIter<'a, Square, Self::Move>,
        fn((Self::Move, &Square)) -> Option<Self::Move>,
    >;
    type Player = ZeroSumPlayer;

    fn max_moves(&self) -> Option<usize> {
        Some(self.size.pow(self.dim as u32))
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn player(&self) -> Self::Player {
        if self.move_count % 2 == 0 {
            ZeroSumPlayer::One
        } else {
            ZeroSumPlayer::Two
        }
    }

    fn make_move(&mut self, m: &Self::Move) -> bool {
        if *self.board.get(m.clone()).unwrap() == Square::Empty {
            let square = if self.player() == ZeroSumPlayer::One {
                Square::X
            } else {
                Square::O
            };

            *self.board.get_mut(m).unwrap() = square;
            self.move_count += 1;
            true
        } else {
            false
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        self.board
            .indexed_iter()
            .filter_map(move |(index, square)| {
                if square == &Square::Empty {
                    Some(index)
                } else {
                    None
                }
            })
    }

    fn is_winning_move(&self, m: &Self::Move) -> Option<Self::Player> {
        // check if the amount of moves is less than (size * 2) - 1
        // if it is, then it's impossible to win
        if self.move_count + 1 < self.size * 2 - 1 {
            return None;
        }

        let mut board = self.clone();
        board.make_move(m);

        // check if the board has any matches of SIZE in a row
        // horizontal, diagonal, and vertical
        // wins whenever it meets the following conditions:
        // where (a1, a2, ... an) is the move
        // each an has to follow a set rule:
        // - it stays the same
        // - it increases
        // - it decreases
        // e.g. (0, 0, 2), (0, 1, 1), (0, 2, 0) wins
        for offset in offsets(m, self.size) {
            if board.winning_line(m, &offset) {
                return Some(self.player());
            }
        }

        None
    }

    fn is_draw(&self) -> bool {
        self.move_count == self.size.pow(self.dim as u32)
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

fn format_dim(dim: &Dim<IxDynImpl>) -> String {
    format!("{:?}", dim.as_array_view().as_slice().unwrap())
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (index, square) in self.board.indexed_iter() {
            writeln!(f, "{:?} @ {}", square, format_dim(&index))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_solver::move_scores;
    use std::collections::HashMap;

    fn best_moves(game: &TicTacToe) -> Option<Dim<IxDynImpl>> {
        move_scores(game, &mut HashMap::new())
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(m, _)| m)
    }

    #[test]
    fn test_middle_move() {
        let mut game = TicTacToe::new(2, 3);
        game.make_move(&vec![0, 0].into_dimension());

        let best_move = best_moves(&game).unwrap();

        assert_eq!(best_move, vec![1, 1].into_dimension());
    }

    #[test]
    fn test_always_tie() {
        let game = TicTacToe::new(2, 3);

        assert!(move_scores(&game, &mut HashMap::new()).all(|(_, score)| score == 0));
    }

    #[test]
    fn test_win() {
        let mut game = TicTacToe::new(2, 3);

        game.make_move(&vec![0, 2].into_dimension()); // X
        game.make_move(&vec![0, 1].into_dimension()); // O
        game.make_move(&vec![1, 1].into_dimension()); // X
        game.make_move(&vec![0, 0].into_dimension()); // O
        game.make_move(&vec![2, 0].into_dimension()); // X

        assert!(game.won());
    }

    #[test]
    fn test_win_3d() {
        let mut game = TicTacToe::new(3, 3);

        game.make_move(&vec![0, 0, 0].into_dimension()); // X
        game.make_move(&vec![0, 0, 1].into_dimension()); // O
        game.make_move(&vec![0, 1, 1].into_dimension()); // X
        game.make_move(&vec![0, 0, 2].into_dimension()); // O
        game.make_move(&vec![0, 2, 2].into_dimension()); // X
        game.make_move(&vec![0, 1, 0].into_dimension()); // O
        game.make_move(&vec![0, 2, 0].into_dimension()); // X

        assert!(game.won());
    }

    #[test]
    fn test_always_tie_1d() {
        let game = TicTacToe::new(1, 3);

        assert!(move_scores(&game, &mut HashMap::new()).all(|(_, score)| score == 0));
    }
}
