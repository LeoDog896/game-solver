use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

pub enum Player {
    P1,
    P2,
}

pub trait Game<T> {
    /// Returns the player whose turn it is.
    fn player(&self) -> Player;
    /// Returns the number of moves that have been made.
    fn n_moves(&self) -> u32;
    /// Returns the size of the board (used to calculate the score)
    fn size(&self) -> u32;
    /// Returns true if the game is over.
    fn is_over(&self) -> bool;
    /// Returns true if the move was valid, and makes the move if it was.
    fn make_move(&mut self, m: T) -> bool;
    /// Returns a vector of all possible moves.
    fn possible_moves(&self) -> Vec<T>;
    /// Returns true if the move is a winning move.
    fn is_winning_move(&self, m: T) -> bool;
}

/// A transposition table for a game.
/// Transposition tables implement caching for minimax algorithms.
pub struct TranspositionTable<T: Eq + Hash + Game<U>, U> {
    table: HashMap<T, i32>,
    _t: PhantomData<U>,
}

impl<T: Game<U> + Clone + Eq + Hash, U: Debug> TranspositionTable<T, U> {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            _t: PhantomData,
        }
    }
}

/// Runs the two-player minimax variant on a game.
/// It uses alpha beta pruning (e.g. you can specify [-1, 1])
pub fn negamax<T: Game<U> + Clone + Eq + Hash, U: Debug>(
    game: &T,
    transposition_table: &mut TranspositionTable<T, U>,
    mut alpha: i32,
    mut beta: i32,
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
