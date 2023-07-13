use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    marker::PhantomData,
};

enum Player {
    P1,
    P2,
}

trait Game<T> {
    fn player(&self) -> Player;
    fn n_moves(&self) -> u32;
    fn size(&self) -> u32;
    fn is_over(&self) -> bool;
    fn make_move(&mut self, m: T) -> bool;
    fn possible_moves(&self) -> Vec<T>;
    fn is_winning_move(&self, m: T) -> bool;
}

struct TranspositionTable<T: Eq + Hash + Game<U>, U> {
    table: HashMap<T, i32>,
    _t: PhantomData<U>,
}

impl<T: Game<U> + Clone + Eq + Hash, U: Debug> TranspositionTable<T, U> {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
            _t: PhantomData,
        }
    }
}

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

fn negamax<T: Game<U> + Clone + Eq + Hash, U: Debug>(
    game: &T,
    transposition_table: &mut TranspositionTable<T, U>,
    mut alpha: i32,
    mut beta: i32
) -> i32 {
    for m in game.possible_moves() {
        if game.is_winning_move(m) {
            return game.size() as i32 - game.n_moves() as i32;
        }
    }

    let max = game.size() - game.n_moves();
    if beta > max as i32 {
        beta = max as i32;
        if alpha >= beta {
            return beta;
        }
    }

    for m in game.possible_moves() {
        let mut board = game.clone();
        board.make_move(m);
        let score = if transposition_table.table.contains_key(&board) {
            transposition_table.table[&board]
        } else {
            let score = -negamax(&board, transposition_table, -beta, -alpha);

            transposition_table.table.insert(board.clone(), score);

            score
        };
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn main() {
    let mut transposition_table: TranspositionTable<Chomp, (u32, u32)> = TranspositionTable::new();
    let mut game = Chomp::new(8, 5);
    println!("{}", game);

    let best_move: ((u32, u32), i32) = game
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
