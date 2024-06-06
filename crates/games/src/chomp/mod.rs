#![doc = include_str!("./README.md")]

pub mod cli;
#[cfg(feature = "egui")]
pub mod gui;

use array2d::Array2D;
use game_solver::game::{Game, ZeroSumPlayer};

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

use crate::util::move_natural::NaturalMove;

#[derive(Clone, Hash, Eq, PartialEq)]
struct Chomp {
    width: usize,
    height: usize,
    /// True represents a square that has not been eaten
    board: Array2D<bool>,
    move_count: usize,
}

impl Chomp {
    fn new(width: usize, height: usize) -> Self {
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

pub type ChompMove = NaturalMove<2>;

impl Game for Chomp {
    type Move = ChompMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    type Player = ZeroSumPlayer;

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

    fn make_move(&mut self, m: &Self::Move) -> bool {
        if *self.board.get(m.0[0], m.0[1]).unwrap() {
            for i in m.0[0]..self.width {
                for j in 0..=m.0[1] {
                    self.board.set(i, j, false).unwrap();
                }
            }
            self.move_count += 1;
            true
        } else {
            false
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

    fn is_winning_move(&self, m: &Self::Move) -> Option<Self::Player> {
        let mut board = self.clone();
        board.make_move(m);
        if board.possible_moves().next().is_none() {
            Some(self.player())
        } else {
            None
        }
    }

    fn is_draw(&self) -> bool {
        self.move_count == self.width * self.height
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use game_solver::move_scores;

    use super::*;

    #[test]
    fn test_chomp() {
        let game = Chomp::new(6, 4);
        let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();
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
