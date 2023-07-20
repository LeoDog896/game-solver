//! Domineering is a two-player game played on a grid of squares.
//! The goal is to be the last player to make a legal move.
//! 
//! Player 1 places a domino (two adjacent squares) horizontally, and player 2 places a domino vertically.
//! 
//! Learn more: https://en.wikipedia.org/wiki/Domineering

use combinatorial_game::{move_scores, Game, Player};

use std::{
    collections::HashMap,
    env::args,
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Domineering<const SIZE: usize, const WIDTH: usize, const HEIGHT: usize> {
    /// True represents a square - true if empty, false otherwise
    board: [bool; SIZE],
    n_moves: u32,
}

impl<const SIZE: usize, const WIDTH: usize, const HEIGHT: usize> Domineering<SIZE, WIDTH, HEIGHT> {
    fn new() -> Self {
        Self {
            board: [true; SIZE],
            n_moves: 0,
        }
    }
}

impl<const SIZE: usize, const WIDTH: usize, const HEIGHT: usize> Game for Domineering<SIZE, WIDTH, HEIGHT> {
    type Move = (usize, usize);
    type Iter = std::vec::IntoIter<Self::Move>;

    fn max_score(&self) -> u32 {
        (WIDTH * HEIGHT) as u32
    }

    fn min_score(&self) -> i32 {
        -(WIDTH as i32 * HEIGHT as i32) as i32
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
        if !self.board[m.0 as usize + m.1 as usize * WIDTH as usize] {
            false
        } else {
            if self.player() == Player::P1 {
                if m.0 == WIDTH - 1 {
                    return false;
                }
                self.board[m.0 as usize + m.1 as usize * WIDTH as usize] = false;
                self.board[(m.0 + 1) as usize + m.1 as usize * WIDTH as usize] = false;
            } else {
                if m.1 == HEIGHT - 1 {
                    return false;
                }
                self.board[m.0 as usize + m.1 as usize * WIDTH as usize] = false;
                self.board[m.0 as usize + (m.1 + 1) as usize * WIDTH as usize] = false;
            }

            self.n_moves += 1;
            true
        }
    }

    fn possible_moves(&self) -> Self::Iter {
        let mut moves = Vec::new();
        if self.player() == Player::P1 {
            for i in 0..HEIGHT {
                for j in 0..WIDTH - 1 {
                    if self.board[j as usize + i as usize * WIDTH as usize] && self.board[(j + 1) as usize + i as usize * WIDTH as usize] {
                        moves.push((j, i));
                    }
                }
            }
        } else {
            for i in 0..HEIGHT - 1 {
                for j in 0..WIDTH {
                    if self.board[j as usize + i as usize * WIDTH as usize] && self.board[j as usize + (i + 1) as usize * WIDTH as usize] {
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
        board.possible_moves().collect::<Vec<_>>().is_empty()
    }
}

impl<const SIZE: usize, const WIDTH: usize, const HEIGHT: usize> Display for Domineering<SIZE, WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                if self.board[j as usize + i as usize * WIDTH as usize] {
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

// nm, n, m
type DomineeringGame = Domineering<36, 6, 6>;

fn main() {
    let mut game = DomineeringGame::new();
    let mut transposition_table = HashMap::<DomineeringGame, i32>::new();

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

    let mut move_scores = move_scores(
        &game,
        &mut transposition_table,
        game.min_score(),
        game.max_score() as i32,
    ).collect::<Vec<_>>();

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
    fn test_domineering() {
        let mut game = Domineering::<25, 5, 5>::new();
        let mut move_scores = move_scores(
            &game,
            &mut HashMap::new(),
            game.min_score(),
            game.max_score() as i32,
        ).collect::<Vec<_>>();

        assert_eq!(move_scores.len(), game.possible_moves().len());
        
        move_scores.sort();
        let mut current_scores = vec![
            ((4, 3), -13),
            ((3, 3), -13),
            ((2, 3), -13),
            ((1, 3), -13),
            ((0, 3), -13),
            ((3, 2), -13),
            ((1, 2), -13),
            ((3, 1), -13),
            ((1, 1), -13),
            ((4, 0), -13),
            ((3, 0), -13),
            ((2, 0), -13),
            ((1, 0), -13),
            ((0, 0), -13),
            ((4, 2), -15),
            ((2, 2), -15),
            ((0, 2), -15),
            ((4, 1), -15),
            ((2, 1), -15),
            ((0, 1), -15),
        ];

        current_scores.sort();

        assert_eq!(
            move_scores,
            current_scores
        );
    }
}
