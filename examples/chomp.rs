//! Chomp is a two-player game played on a rectangular grid of squares.
//! The bottom right square is poisoned, and the players take turns eating squares.
//! Every square they eat, every square to the right and above it is also eaten (inclusively)
//!
//! This is a flipped version of the traiditional [Chomp](https://en.wikipedia.org/wiki/Chomp) game.

use combinatorial_game::{negamax, Game, Player};

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
    let moves: Vec<(u32, u32)> = args()
        .skip(1)
        .map(|arg| {
            let numbers: Vec<u32> = arg
                .split("-")
                .map(|num| num.parse::<u32>().unwrap())
                .collect();

            (numbers[0], numbers[1])
        })
        .collect();

    for game_move in moves {
        game.make_move(game_move);
    }

    println!("{}", game);

    let possible_moves = game.possible_moves();

    let move_scores = possible_moves.iter().map(|m| {
        let mut board = game.clone();
        board.make_move(*m);
        (
            *m,
            negamax(
                &board,
                &mut transposition_table,
                -(game.size() as i32),
                game.size() as i32,
            ),
        )
    }).collect::<Vec<_>>();

    let best_move = move_scores.iter().min_by_key(|(_, score)| *score).copied();

    if let Some((_, score)) = best_move {
        let matching_moves = move_scores.iter().filter(|m| m.1 == score);
        println!("Best moves @ score {}: ", score);
        for game_move in matching_moves {
            println!("\t{:?}", game_move.0);
        }
    } else {
        println!("Player {:?} won!", game.player().opposite());
    }
}
