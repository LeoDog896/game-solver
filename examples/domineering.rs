//! Connect 4 is a two-player game played on a 7x6 grid. Players take turns placing pieces on the
//! bottom row, and the pieces fall to the lowest available square in the column.
//! The first player to get 4 in a row (horizontally, vertically, or diagonally) wins.
//!
//! Learn more: https://en.wikipedia.org/wiki/Connect_Four

use array2d::Array2D;
use game_solver::{move_scores, Game, Player};

use std::{
    env::args,
    fmt::{Display, Formatter},
    hash::Hash, collections::HashMap,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Domineering<const WIDTH: usize, const HEIGHT: usize> {
    /// True represents a square - true if empty, false otherwise
    board: Array2D<bool>,
    move_count: u32,
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

    fn max_score(&self) -> u32 {
        (WIDTH * HEIGHT) as u32
    }

    fn min_score(&self) -> i32 {
        -(WIDTH as i32 * HEIGHT as i32)
    }

    fn player(&self) -> Player {
        if self.move_count % 2 == 0 {
            Player::P1
        } else {
            Player::P2
        }
    }

    fn score(&self) -> u32 {
        self.max_score() - self.move_count
    }

    fn make_move(&mut self, m: Self::Move) -> bool {
        if *self.board.get(m.0, m.1).unwrap() {
            if self.player() == Player::P1 {
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
        if self.player() == Player::P1 {
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

    fn is_winning_move(&self, m: Self::Move) -> bool {
        let mut board = self.clone();
        board.make_move(m);
        board.possible_moves().next().is_none()
    }

    fn is_draw(&self) -> bool {
        self.move_count == WIDTH as u32 * HEIGHT as u32
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

fn main() {
    let mut game = DomineeringGame::new();

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(1).for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move((numbers[0], numbers[1]));
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();

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

    /// Get the winner of a generic configuration of domineering
    fn winner<const WIDTH: usize, const HEIGHT: usize>() -> Option<Player> {
        let game = Domineering::<WIDTH, HEIGHT>::new();
        let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();

        if move_scores.is_empty() {
            None
        } else {
            move_scores.sort_by_key(|m| m.1);
            move_scores.reverse();
            if move_scores[0].1 > 0 {
                Some(Player::P1)
            } else {
                Some(Player::P2)
            }
        }
    }

    #[test]
    fn test_wins() {
        assert_eq!(winner::<5, 5>(), Some(Player::P2));
        assert_eq!(winner::<4, 4>(), Some(Player::P1));
        assert_eq!(winner::<3, 3>(), Some(Player::P1));
        assert_eq!(winner::<13, 2>(), Some(Player::P2));
        assert_eq!(winner::<11, 2>(), Some(Player::P1));
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
