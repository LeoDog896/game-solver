use std::{collections::HashMap, hash::Hash};

#[derive(PartialEq, Eq, Debug)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    pub fn opposite(&self) -> Player {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1
        }
    }
}

pub trait Game {
    /// The type of move this game uses.
    type Move;

    /// Returns the player whose turn it is.
    fn player(&self) -> Player;

    /// Returns the number of moves that have been made.
    fn n_moves(&self) -> u32;

    /// Returns the size of the board (used to calculate the score)
    fn size(&self) -> u32;

    /// Returns true if the game is over.
    fn is_over(&self) -> bool;

    /// Returns true if the move was valid, and makes the move if it was.
    fn make_move(&mut self, m: Self::Move) -> bool;

    /// Returns a vector of all possible moves.
    fn possible_moves(&self) -> Vec<Self::Move>;

    /// Returns true if the move is a winning move.
    fn is_winning_move(&self, m: Self::Move) -> bool;
}

/// A transposition table for a game.
/// Transposition tables implement caching for minimax algorithms.
pub trait TranspositionTable<T: Eq + Hash + Game> {
    fn new() -> Self where Self: Sized;
    fn get(&self, board: &T) -> Option<i32>;
    fn insert(&mut self, board: T, score: i32);
    fn has(&self, board: &T) -> bool;
}

impl<K: Eq + Hash + Game> TranspositionTable<K> for HashMap<K, i32> {
    fn new() -> Self where Self: Sized {
        HashMap::new()
    }

    fn get(&self, board: &K) -> Option<i32> {
        self.get(&board).copied()
    }

    fn insert(&mut self, board: K, score: i32) {
        self.insert(board, score);
    }

    fn has(&self, board: &K) -> bool {
        self.contains_key(&board)
    }
}

/// Runs the two-player minimax variant on a game.
/// It uses alpha beta pruning (e.g. you can specify [-1, 1] to get only win/loss/draw moves).
pub fn negamax<T: Game + Clone + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
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
        let score = if transposition_table.has(&board) {
            transposition_table.get(&board).unwrap()
        } else {
            let score = -negamax(&board, transposition_table, -beta, -alpha);
            transposition_table.insert(board, score);

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
