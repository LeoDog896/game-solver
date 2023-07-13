//! Domineering (aka stop-gate) is a two-player game played on a rectangular grid of squares. The
//! players take turns placing dominoes on the board. The first player to be unable to make a move
//! loses.
//!
//! Player one places dominoes horizontally, and player two places dominoes vertically.

use combinatorial_game::{negamax, Game, Player, TranspositionTable};

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Domineering {
    width: u32,
    height: u32,
    /// True represents an unfilled square
    board: Vec<Vec<bool>>,
    n_moves: u32,
}

impl Domineering {
    fn new(width: u32, height: u32) -> Self {
        let mut board = Vec::new();
        for _ in 0..height {
            let mut row = Vec::new();
            for _ in 0..width {
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

impl Game for Domineering {
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

    fn possible_moves(&self) -> Vec<Self::Move> {
        let mut moves = Vec::new();
        // TODO: moves
        moves
    }

    fn make_move(&mut self, m: Self::Move) -> bool {
        // TODO: make_move
        self.n_moves += 1;
        true
    }

    fn is_over(&self) -> bool {
        self.possible_moves().is_empty()
    }
}

impl Display for Domineering {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.height {
            for j in 0..self.width {
                if self.board[i as usize][j as usize] {
                    write!(f, "O")?;
                } else {
                    write!(f, "X")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() {
    let mut transposition_table = TranspositionTable::<Domineering>::new();
    let game = Domineering::new(8, 5);
    println!("{}", game);

    // let best_move = game
    //     .possible_moves()
    //     .iter()
    //     .map(|m| {
    //         let mut board = game.clone();
    //         board.make_move(*m);
    //         (*m, -negamax(&board, &mut transposition_table, -100, 100))
    //     })
    //     .max_by_key(|(_, score)| *score)
    //     .unwrap();

    // println!("Best move: {:?} with score {}", best_move.0, best_move.1);
}
