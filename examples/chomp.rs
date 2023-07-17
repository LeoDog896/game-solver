//! Chomp is a two-player game played on a rectangular grid of squares.
//! The bottom right square is poisoned, and the players take turns eating squares.
//! Every square they eat, every square to the right and above it is also eaten (inclusively)
//!
//! This is a flipped version of the traiditional [Chomp](https://en.wikipedia.org/wiki/Chomp) game.

use combinatorial_game::{move_scores, Game, Player};

use std::{
    collections::HashMap,
    env::args,
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Chomp {
    width: u32,
    height: u32,
    /// True represents a square that has not been eaten
    board: Vec<Vec<bool>>,
    n_moves: u32,
}

impl Chomp {
    fn new(width: u32, height: u32) -> Self {
        let mut board = Vec::new();
        for i in 0..height {
            let mut row = Vec::new();
            for j in 0..width {
                if i == height - 1 && j == 0 {
                    row.push(false);
                    continue;
                }

                row.push(true);
            }
            board.push(row);
        }

        Self {
            width,
            height,
            board,
            n_moves: 0,
        }
    }
}

impl Game for Chomp {
    type Move = (u32, u32);

    fn player(&self) -> Player {
        if self.n_moves % 2 == 0 {
            Player::P1
        } else {
            Player::P2
        }
    }

    fn n_moves(&self) -> u32 {
        self.n_moves
    }

    fn size(&self) -> u32 {
        self.width * self.height
    }

    fn make_move(&mut self, m: Self::Move) -> bool {
        if self.board[m.1 as usize][m.0 as usize] {
            for i in m.0..self.width {
                for j in 0..=m.1 {
                    self.board[j as usize][i as usize] = false;
                }
            }
            self.n_moves += 1;
            true
        } else {
            false
        }
    }

    fn possible_moves(&self) -> Vec<Self::Move> {
        let mut moves = Vec::new();
        for i in 0..self.width {
            for j in 0..self.height {
                if self.board[j as usize][i as usize] {
                    moves.push((i, j));
                }
            }
        }
        moves
    }

    fn is_over(&self) -> bool {
        // the game is over when there are no longer any moves
        self.possible_moves().is_empty()
    }

    fn is_winning_move(&self, m: Self::Move) -> bool {
        let mut board = self.clone();
        board.make_move(m);
        board.is_over()
    }
}

impl Display for Chomp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for i in 0..self.height {
            for j in 0..self.width {
                if self.board[i as usize][j as usize] {
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
    let mut transposition_table = HashMap::<Chomp, i32>::new();
    let mut game = Chomp::new(8, 5);

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
        -(game.size() as i32),
        game.size() as i32,
    );

    if !move_scores.is_empty() {
        move_scores.sort_by_key(|m| m.1);
        if game.player() == Player::P1 {
            move_scores.reverse();
        }

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
    fn test_add() {
        let mut game = Chomp::new(8, 5);
        assert_eq!(game.make_move((0, 0)), true);
        assert_eq!(game.possible_moves().len(), 31);
        let move_scores = move_scores(
            &game,
            &mut HashMap::new(),
            -(game.size() as i32),
            game.size() as i32,
        );
        assert_eq!(move_scores.len(), game.possible_moves().len());
        assert_eq!(
            move_scores,
            vec![
                ((0, 1), -2),
                ((0, 2), -10),
                ((0, 3), -37),
                ((1, 1), -38),
                ((1, 2), -38),
                ((1, 3), -2),
                ((1, 4), -37),
                ((2, 1), -38),
                ((2, 2), -38),
                ((2, 3), -2),
                ((2, 4), -35),
                ((3, 1), -38),
                ((3, 2), -38),
                ((3, 3), -32),
                ((3, 4), -1),
                ((4, 1), -38),
                ((4, 2), -38),
                ((4, 3), -8),
                ((4, 4), -38),
                ((5, 1), -38),
                ((5, 2), -38),
                ((5, 3), -30),
                ((5, 4), -38),
                ((6, 1), -38),
                ((6, 2), -38),
                ((6, 3), -30),
                ((6, 4), -38),
                ((7, 1), -38),
                ((7, 2), -38),
                ((7, 3), -30),
                ((7, 4), -38)
            ]
        );
    }
}