use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

/// Represents a player in a two-player combinatorial game.
#[derive(PartialEq, Eq, Debug)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::P1 => Self::P2,
            Self::P2 => Self::P1,
        }
    }
}

/// Represents a combinatorial game.
pub trait Game {
    /// The type of move this game uses.
    type Move: Copy;

    /// Returns the player whose turn it is.
    fn player(&self) -> Player;

    /// Returns the number of moves that have been made.
    fn n_moves(&self) -> u32;

    /// Scores a position. The default implementation uses the size minus the number of moves (for finite games)
    fn score(&self) -> u32 {
        self.size() - self.n_moves()
    }

    /// Returns the size of the board (used to calculate the score)
    fn size(&self) -> u32;

    /// Returns true if the game is over.
    fn is_over(&self) -> bool;

    /// Returns true if the move was valid, and makes the move if it was.
    fn make_move(&mut self, m: Self::Move) -> bool;

    /// Returns a vector of all possible moves.
    ///
    /// If possible, this function should "guess" what the best moves are first.
    /// For example, if this is for tic tac toe, it should give the middle move first.
    /// This allows alpha-beta pruning to move faster.
    fn possible_moves(&self) -> Vec<Self::Move>;

    /// Returns true if the move is a winning move.
    fn is_winning_move(&self, m: Self::Move) -> bool;
}

/// A transposition table for a game.
/// Transposition tables implement caching for minimax algorithms.
///
/// Transposition tables should optimally be O(1) for get, has, and insert.
/// The best structure for this is a `HashMap`.
///
/// all `HashMap`s already implement `TranspositionTable`.
pub trait TranspositionTable<T: Eq + Hash + Game> {
    fn get(&self, board: &T) -> Option<i32>;
    fn insert(&mut self, board: T, score: i32);
    fn has(&self, board: &T) -> bool;
}

impl<K: Eq + Hash + Game, S: BuildHasher + Default> TranspositionTable<K> for HashMap<K, i32, S> {
    fn get(&self, board: &K) -> Option<i32> {
        self.get(board).copied()
    }

    fn insert(&mut self, board: K, score: i32) {
        self.insert(board, score);
    }

    fn has(&self, board: &K) -> bool {
        self.contains_key(board)
    }
}

/// Runs the two-player minimax variant on a game.
/// It uses alpha beta pruning (e.g. you can specify \[-1, 1\] to get only win/loss/draw moves).
///
/// This function requires a transposition table. If you only plan on running this function once,
/// you can use a the in-built `HashMap`.
pub fn negamax<T: Game + Clone + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    for m in game.possible_moves() {
        if game.is_winning_move(m) {
            return game.score() as i32 - 1;
        }
    }

    {
        let max = transposition_table.get(game).unwrap_or(game.score() as i32);
        if beta > max {
            beta = max;
            if alpha >= beta {
                return beta;
            }
        }
    }

    for m in game.possible_moves() {
        let mut board = game.clone();
        board.make_move(m);

        let score = -negamax(&board, transposition_table, -beta, -alpha);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    transposition_table.insert(game.clone(), alpha - game.score() as i32 + 1);

    alpha
}

/// Utility function to get a list of the move scores of a certain game.
///
/// This is useful if you're making a visual interface to display the various scores
/// for each move.
pub fn move_scores<T: Game + Clone + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
    alpha: i32,
    beta: i32,
) -> Vec<(T::Move, i32)> {
    game.possible_moves()
        .iter()
        .map(|m| {
            let mut board = game.clone();
            board.make_move(*m);
            (*m, -negamax(&board, transposition_table, alpha, beta))
        })
        .collect::<Vec<_>>()
}
