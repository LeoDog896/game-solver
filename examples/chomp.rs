//! Chomp is a two-player game played on a rectangular grid of squares.
//! The bottom right square is poisoned, and the players take turns eating squares.
//! Every square they eat, every square to the right and above it is also eaten (inclusively)
//!
//! This is a flipped version of the traiditional [Chomp](https://en.wikipedia.org/wiki/Chomp) game.

use array2d::Array2D;
use combinatorial_game::{move_scores, Game, Player};

use std::{
    env::args,
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Chomp {
    width: usize,
    height: usize,
    /// True represents a square that has not been eaten
    board: Array2D<bool>,
    n_moves: u32,
}

impl Chomp {
    fn new(width: usize, height: usize) -> Self {
        let mut board = Array2D::filled_with(true, width, height);
        board.set(0, height - 1, false).unwrap();

        Self {
            width,
            height,
            board,
            n_moves: 0,
        }
    }
}

impl Game for Chomp {
    type Move = (usize, usize);
    type Iter = std::vec::IntoIter<Self::Move>;

    fn max_score(&self) -> u32 {
        (self.width * self.height).try_into().unwrap()
    }

    fn min_score(&self) -> i32 {
        -(self.width as i32 * self.height as i32)
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
        if *self.board.get(m.0, m.1).unwrap() {
            for i in m.0..self.width {
                for j in 0..=m.1 {
                    self.board.set(i, j, false).unwrap();
                }
            }
            self.n_moves += 1;
            true
        } else {
            false
        }
    }

    fn possible_moves(&self) -> Self::Iter {
        let mut moves = Vec::new();
        for i in (0..self.height).rev() {
            for j in 0..self.width {
                if *self.board.get(j, i).unwrap() {
                    moves.push((j, i));
                }
            }
        }
        moves.into_iter()
    }

    fn is_winning_move(&self, m: Self::Move) -> bool {
        let mut board = self.clone();
        board.make_move(m);
        board.possible_moves().collect::<Vec<_>>().is_empty()
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

fn main() {
    let mut game = Chomp::new(6, 4);

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(1).for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split("-")
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move((numbers[0], numbers[1]));
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
            print!("({}, {}), ", game_move.0, game_move.1);
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
    fn test_chomp() {
        let game = Chomp::new(6, 4);
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
