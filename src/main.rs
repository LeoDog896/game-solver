mod solve;

use solve::{negamax, Game, Player, TranspositionTable};

use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Chomp {
    n: u32,
    m: u32,
    board: Vec<Vec<bool>>,
    n_moves: u32,
}

impl Chomp {
    fn new(n: u32, m: u32) -> Self {
        let mut board = Vec::new();
        for i in 0..n {
            let mut row = Vec::new();
            for j in 0..m {
                if i == 0 && j == m - 1 {
                    row.push(false);
                    continue;
                }

                row.push(true);
            }
            board.push(row);
        }

        Self {
            n,
            m,
            board,
            n_moves: 0,
        }
    }
}

impl Game<(u32, u32)> for Chomp {
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
        self.n * self.m
    }

    fn make_move(&mut self, m: (u32, u32)) -> bool {
        if self.board[m.0 as usize][m.1 as usize] {
            for i in m.0..self.n {
                for j in 0..=m.1 {
                    self.board[i as usize][j as usize] = false;
                }
            }
            self.n_moves += 1;
            true
        } else {
            false
        }
    }

    fn possible_moves(&self) -> Vec<(u32, u32)> {
        let mut moves = Vec::new();
        for i in 0..self.n {
            for j in 0..self.m {
                if self.board[i as usize][j as usize] {
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

    fn is_winning_move(&self, m: (u32, u32)) -> bool {
        let mut board = self.clone();
        board.make_move(m);
        board.is_over()
    }
}

impl Display for Chomp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for j in 0..self.m {
            for i in 0..self.n {
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
    let mut transposition_table: TranspositionTable<Chomp, (u32, u32)> = TranspositionTable::new();
    let game = Chomp::new(8, 5);
    println!("{}", game);

    let best_move = game
        .possible_moves()
        .iter()
        .map(|m| {
            let mut board = game.clone();
            board.make_move(*m);
            (*m, -negamax(&board, &mut transposition_table, -100, 100))
        })
        .max_by_key(|(_, score)| *score)
        .unwrap();

    println!("Best move: {:?} with score {}", best_move.0, best_move.1);
}
