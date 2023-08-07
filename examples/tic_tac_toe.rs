//! Tic Tac Toe is a traditional two-player game played on a 3x3 grid.
//! For the sake of complexity, this allows simulating any n-dimensional 3-in-a-row game
//! with the same bounds as the traditional game.

use ndarray::{ArrayD, IxDyn, Dim, IxDynImpl, iter::IndexedIter, IntoDimension, Dimension};
use game_solver::{move_scores, Game, Player};

use std::{
    env::args,
    fmt::{Display, Formatter},
    hash::Hash, iter::FilterMap,
};

/// The straight size of the board. E.g. if there were 2 dimensions, it would be a SIZE x SIZE board.
const SIZE: usize = 3;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
enum Square {
    Empty,
    X,
    O,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct TicTacToe {
    dim: usize,
    /// True represents a square that has not been eaten
    board: ArrayD<Square>,
    n_moves: u32,
}

impl TicTacToe {
    fn new(dim: usize) -> Self {
        // we want [SIZE; dim] but dim isn't a const - we have to get the slice from a vec
        let board = ArrayD::from_elem(IxDyn(&vec![SIZE; dim]), Square::Empty);

        Self {
            dim,
            board,
            n_moves: 0,
        }
    }
}

impl Game for TicTacToe {
    type Move = Dim<IxDynImpl>;
    type Iter<'a> = FilterMap<IndexedIter<'a, Square, Self::Move>, fn((Self::Move, &Square)) -> Option<Self::Move>>;

    fn max_score(&self) -> u32 {
        (SIZE.pow(self.dim as u32) * self.dim).try_into().unwrap()
    }

    fn min_score(&self) -> i32 {
        -(SIZE.pow(self.dim as u32) as i32 * self.dim as i32)
    }

    fn player(&self) -> Player {
        if self.n_moves % 2 == 0 {
            Player::P1
        } else {
            Player::P2
        }
    }

    fn score(&self) -> u32 {
        self.max_score() - self.n_moves
    }

    fn make_move(&mut self, m: Self::Move) -> bool {
        if *self.board.get(m.clone()).unwrap() == Square::Empty {
            let square = if self.player() == Player::P1 {
                Square::X
            } else {
                Square::O
            };

            *self.board.get_mut(m).unwrap() = square;
            true
        } else {
            false
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        self.board.indexed_iter().filter_map(move |(index, square)| {
            if square == &Square::Empty {
                Some(index)
            } else {
                None
            }
        })
    }

    fn is_winning_move(&self, m: Self::Move) -> bool {
        let mut board = self.clone();
        board.make_move(m.clone());

        // check if the board has any matches of SIZE in a row
        // horizontal, diagonal, and vertical
        // wins whenever it meets the following conditions:
        // where (a1, a2, ... an) is the move
        // each an has to follow a set rule:
        // - it stays the same
        // - it increases
        // - it decreases
        // e.g. (0, 0, 2), (0, 1, 1), (0, 2, 0) wins

        // we can get the neighbors of the current move
    }
}

fn format_dim(dim: &Dim<IxDynImpl>) -> String {
    format!("{:?}", dim.as_array_view().as_slice().unwrap())
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (index, square) in self.board.indexed_iter() {
            write!(f, "{:?} @ {}\n", square, format_dim(&index))?;
        }
        Ok(())
    }
}

fn main() {
    let mut game = TicTacToe::new(2);

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(1).for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split("-")
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move(numbers.into_dimension());
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = move_scores(&game).collect::<Vec<_>>();

    if !move_scores.is_empty() {
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("{}, ", format_dim(&game_move));
        }
        println!();
    } else {
        println!("Player {:?} won!", game.player().opposite());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tictactoe() {
        let game = TicTacToe::new(4);
        let mut move_scores = move_scores(&game).collect::<Vec<_>>();
        move_scores.sort();

        let mut new_scores = vec![
            ((2, 2), 13),
            ((5, 0), -12),
            ((4, 0), -12),
            ((3, 0), -12),
            ((2, 0), -12),
            ((0, 0), -12),
            ((5, 1), -12),
            ((4, 1), -12),
            ((3, 1), -12),
            ((2, 1), -12),
            ((0, 1), -12),
            ((5, 2), -12),
            ((4, 2), -12),
            ((3, 2), -12),
            ((5, 3), -12),
            ((1, 0), -16),
            ((1, 1), -16),
            ((1, 2), -16),
            ((4, 3), -16),
            ((3, 3), -16),
            ((2, 3), -16),
            ((0, 2), -22),
            ((1, 3), -22),
        ];

        new_scores.sort();

        assert_eq!(move_scores, new_scores);
    }
}
