//! `game-solver` is a library for solving games.
//!
//! If you want to read how to properly use this library,
//! [the book](https://leodog896.github.io/game-solver/book) is
//! a great place to start.

#[cfg(feature = "rayon")]
use rayon::prelude::*;

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
    /// Get the player opposite to this one.
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
    type Move: Clone;

    /// The iterator type for possible moves.
    type Iter<'a>: Iterator<Item = Self::Move> + 'a
    where
        Self: 'a;

    /// Returns the player whose turn it is.
    fn player(&self) -> Player;

    /// Scores a position. The default implementation uses the size minus the number of moves (for finite games)
    fn score(&self) -> u32;

    /// Get the max score of a game.
    fn max_score(&self) -> u32;

    /// Get the min score of a game. This should be negative.
    fn min_score(&self) -> i32;

    /// Returns true if the move was valid, and makes the move if it was.
    fn make_move(&mut self, m: Self::Move) -> bool;

    /// Returns a vector of all possible moves.
    ///
    /// If possible, this function should "guess" what the best moves are first.
    /// For example, if this is for tic tac toe, it should give the middle move first.
    /// This allows alpha-beta pruning to move faster.
    fn possible_moves(&self) -> Self::Iter<'_>;

    /// Returns true if the move is a winning move.
    fn is_winning_move(&self, m: Self::Move) -> bool;

    /// Returns true if the game is a draw.
    fn is_draw(&self) -> bool;
}

/// A memoization strategy for a perfect-information sequential game.
///
/// Transposition tables should optimally be a form of hash table.
///
/// # Optimization
///
/// [rustc-hash](https://crates.io/crates/rustc-hash) is the best
/// hashmap implementation for this crate, given its speed.
///
/// To optimize it, its better to have your Moves (keys) be numbers.
pub trait TranspositionTable<T: Eq + Hash + Game> {
    /// Get the score of a board, if it exists.
    fn get(&self, board: &T) -> Option<i32>;

    /// Insert a board into the transposition table.
    fn insert(&mut self, board: T, score: i32);

    /// Returns true if the board is in the transposition table.
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
fn negamax<T: Game + Clone + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    if game.is_draw() {
        return 0;
    }

    for m in &mut game.possible_moves() {
        if game.is_winning_move(m.clone()) {
            let mut board = game.clone();
            board.make_move(m);
            return board.score() as i32;
        }
    }

    {
        let max = transposition_table
            .get(game)
            .unwrap_or(game.max_score() as i32);
        if beta > max {
            beta = max;
            if alpha >= beta {
                return beta;
            }
        }
    }

    for m in &mut game.possible_moves() {
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

    transposition_table.insert(game.clone(), alpha);

    alpha
}

/// Solves a game, returning the evaluated score.
///
/// The score of a position is defined by the best possible end result for the player whose turn it is.
/// In 2 player games, if a score > 0, then the player whose turn it is has a winning strategy.
/// If a score < 0, then the player whose turn it is has a losing strategy.
/// Else, the game is a draw (score = 0).
pub fn solve<T: Game + Clone + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
) -> i32 {
    let min = game.min_score();
    let max = game.max_score() as i32 + 1;

    let mut alpha = min;
    let mut beta = max;

    while alpha < beta {
        let med = alpha + (beta - alpha) / 2;
        let r = negamax(game, transposition_table, med, med + 1);

        if r <= med {
            beta = r;
        } else {
            alpha = r;
        }
    }

    alpha
}

/// Utility function to get a list of the move scores of a certain game.
/// Since its evaluating the same game, you can use the same transposition table.
///
/// If you want to evaluate the score of a board as a whole, use the `solve` function.
///
/// # Returns
///
/// An iterator of tuples of the form `(move, score)`.
pub fn move_scores<'a, T: Game + Clone + Eq + Hash>(
    game: &'a T,
    transposition_table: &'a mut dyn TranspositionTable<T>,
) -> impl Iterator<Item = (T::Move, i32)> + 'a {
    game.possible_moves().map(move |m| {
        let mut board = game.clone();
        board.make_move(m.clone());
        // We flip the sign of the score because we want the score from the
        // perspective of the player playing the move, not the player whose turn it is.
        (m, -solve(&board, transposition_table))
    })
}

#[cfg(feature = "rayon")]
pub fn move_scores_par<T>(game: &T) -> Vec<(T::Move, i32)>
where
    T: Game + Clone + Eq + Hash + Sync,
    T::Move: Sync,
{
    let all_moves = game.possible_moves().collect::<Vec<_>>();

    all_moves
        .par_iter()
        .map(move |m| {
            let mut board = game.clone();
            board.make_move(m.clone());
            // We flip the sign of the score because we want the score from the
            // perspective of the player playing the move, not the player whose turn it is.
            (m, -solve(&board, &mut HashMap::new()))
        })
        .collect::<Vec<_>>()
        .iter()
        .map(|(m, s)| ((*m).clone(), *s))
        .collect()
}
