//! Connect 4 is a two-player game played on a 7x6 grid. Players take turns placing pieces on the
//! bottom row, and the pieces fall to the lowest available square in the column.
//! The first player to get 4 in a row (horizontally, vertically, or diagonally) wins.
//!
//! Learn more: https://en.wikipedia.org/wiki/Connect_Four

pub mod cli;

use array2d::Array2D;
use game_solver::game::{Game, ZeroSumPlayer};

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Domineering<const WIDTH: usize, const HEIGHT: usize> {
    /// True represents a square - true if empty, false otherwise
    board: Array2D<bool>,
    move_count: usize,
}

impl<const WIDTH: usize, const HEIGHT: usize> Domineering<WIDTH, HEIGHT> {
    fn new() -> Self {
        Self {
            board: Array2D::filled_with(true, WIDTH, HEIGHT),
            move_count: 0,
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Game for Domineering<WIDTH, HEIGHT> {
    type Move = (usize, usize);
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    type Player = ZeroSumPlayer;

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

    fn make_move(&mut self, m: &Self::Move) -> bool {
        if *self.board.get(m.0, m.1).unwrap() {
            if self.player() == ZeroSumPlayer::One {
                if m.0 == WIDTH - 1 {
                    return false;
                }
                self.board.set(m.0, m.1, false).unwrap();
                self.board.set(m.0 + 1, m.1, false).unwrap();
            } else {
                if m.1 == HEIGHT - 1 {
                    return false;
                }
                self.board.set(m.0, m.1, false).unwrap();
                self.board.set(m.0, m.1 + 1, false).unwrap();
            }

            self.move_count += 1;
            true
        } else {
            false
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();
        if self.player() == ZeroSumPlayer::One {
            for i in 0..HEIGHT {
                for j in 0..WIDTH - 1 {
                    if *self.board.get(j, i).unwrap() && *self.board.get(j + 1, i).unwrap() {
                        moves.push((j, i));
                    }
                }
            }
        } else {
            for i in 0..HEIGHT - 1 {
                for j in 0..WIDTH {
                    if *self.board.get(j, i).unwrap() && *self.board.get(j, i + 1).unwrap() {
                        moves.push((j, i));
                    }
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
        self.move_count == WIDTH * HEIGHT
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

// n, m
type DomineeringGame = Domineering<5, 5>;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use game_solver::move_scores;

    use super::*;

    /// Get the winner of a generic configuration of domineering
    fn winner<const WIDTH: usize, const HEIGHT: usize>() -> Option<ZeroSumPlayer> {
        let game = Domineering::<WIDTH, HEIGHT>::new();
        let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();

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
        assert_eq!(winner::<5, 5>(), Some(ZeroSumPlayer::Two));
        assert_eq!(winner::<4, 4>(), Some(ZeroSumPlayer::One));
        assert_eq!(winner::<3, 3>(), Some(ZeroSumPlayer::One));
        assert_eq!(winner::<13, 2>(), Some(ZeroSumPlayer::Two));
        assert_eq!(winner::<11, 2>(), Some(ZeroSumPlayer::One));
    }

    #[test]
    fn test_domineering() {
        let game = Domineering::<5, 5>::new();
        let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();

        assert_eq!(move_scores.len(), game.possible_moves().len());

        move_scores.sort();

        let mut current_scores = vec![
            ((3, 4), -13),
            ((0, 4), -13),
            ((3, 3), -13),
            ((2, 3), -13),
            ((1, 3), -13),
            ((0, 3), -13),
            ((3, 2), -13),
            ((0, 2), -13),
            ((3, 1), -13),
            ((2, 1), -13),
            ((1, 1), -13),
            ((0, 1), -13),
            ((3, 0), -13),
            ((0, 0), -13),
            ((2, 4), -15),
            ((1, 4), -15),
            ((2, 2), -15),
            ((1, 2), -15),
            ((2, 0), -15),
            ((1, 0), -15),
        ];

        current_scores.sort();

        assert_eq!(move_scores, current_scores);
    }
}
