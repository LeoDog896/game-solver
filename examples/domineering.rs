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
struct Domineering {
    width: u32,
    height: u32,
    /// True represents a square - true if empty, false otherwise
    board: Vec<bool>,
    n_moves: u32,
}

impl Domineering {
    fn new(width: u32, height: u32) -> Self {
        let mut board = Vec::with_capacity((width * height) as usize);
        for _ in 0..height {
            for _ in 0..width {
                board.push(true);
            }
        }

        Self {
            width,
            height,
            board,
            n_moves: 0,
        }
    }
}

impl Game for Domineering {
    type Move = (u32, u32);
    type Iter = std::vec::IntoIter<Self::Move>;

    fn max_score(&self) -> u32 {
        self.width * self.height
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
        if !self.board[m.0 as usize + m.1 as usize * self.width as usize] {
            false
        } else {
            if self.player() == Player::P1 {
                if m.0 == self.width - 1 {
                    return false;
                }
                self.board[m.0 as usize + m.1 as usize * self.width as usize] = false;
                self.board[(m.0 + 1) as usize + m.1 as usize * self.width as usize] = false;
            } else {
                if m.1 == self.height - 1 {
                    return false;
                }
                self.board[m.0 as usize + m.1 as usize * self.width as usize] = false;
                self.board[m.0 as usize + (m.1 + 1) as usize * self.width as usize] = false;
            }

            self.n_moves += 1;
            true
        }
    }

    fn possible_moves(&self) -> Self::Iter {
        let mut moves = Vec::new();
        if self.player() == Player::P1 {
            for i in 0..self.height {
                for j in 0..self.width - 1 {
                    if self.board[j as usize + i as usize * self.width as usize] && self.board[(j + 1) as usize + i as usize * self.width as usize] {
                        moves.push((j, i));
                    }
                }
            }
        } else {
            for i in 0..self.height - 1 {
                for j in 0..self.width {
                    if self.board[j as usize + i as usize * self.width as usize] && self.board[j as usize + (i + 1) as usize * self.width as usize] {
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

impl Display for Domineering {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for i in 0..self.height {
            for j in 0..self.width {
                if self.board[j as usize + i as usize * self.width as usize] {
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
    let mut transposition_table = HashMap::<Domineering, i32>::new();
    let mut game = Domineering::new(5, 5);

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(1).for_each(|arg| {
        let numbers: Vec<u32> = arg
            .split("-")
            .map(|num| num.parse::<u32>().expect("Not a number!"))
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
        let game = Domineering::new(5, 5);
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
