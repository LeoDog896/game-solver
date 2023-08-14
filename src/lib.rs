//! `game_solver` is a library for solving games.
//!
//! If you want to read how to properly use this library,
//! [the book](https://leodog896.github.io/game-solver/book) is
//! a great place to start.

#[cfg(feature = "rayon")]
use dashmap::{DashMap, Map};
#[cfg(feature = "rayon")]
use rayon::prelude::*;
#[cfg(feature = "rayon")]
use std::sync::Arc;

use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

/// Represents a player in a two-player combinatorial game.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Player {
    One,
    Two,
}

impl Player {
    /// Get the player opposite to this one.
    #[must_use]
    pub const fn opponent(&self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
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
    /// The implementation of this should be:
    ///
    /// ```rust
    /// fn player(&self) -> Player {
    ///     if game.move_count % 2 == 0 {
    ///        Player::One
    ///     } else {
    ///         Player::Two
    ///     }
    /// }
    /// ```
    ///
    /// However, no implementation is provided
    /// because this does not keep track of the move count.
    fn player(&self) -> Player;

    /// Scores a position. The default implementation uses the size minus the number of moves (for finite games)
    fn score(&self) -> usize;

    /// Get the max score of a game.
    fn max_score(&self) -> usize;

    /// Get the min score of a game. This should be negative.
    fn min_score(&self) -> isize;

    /// Returns true if the move was valid, and makes the move if it was.
    fn make_move(&mut self, m: &Self::Move) -> bool;

    /// Returns a vector of all possible moves.
    ///
    /// If possible, this function should "guess" what the best moves are first.
    /// For example, if this is for tic tac toe, it should give the middle move first.
    /// This allows alpha-beta pruning to move faster.
    fn possible_moves(&self) -> Self::Iter<'_>;

    /// Returns true if the move is a winning move.
    fn is_winning_move(&self, m: &Self::Move) -> bool;

    /// Returns true if the game is a draw.
    /// This function must exist for the current game,
    /// e.g. with tic tac toe, it must check if the board is full.
    fn is_draw(&self) -> bool;
}

/// A score in a transposition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranspositionTableScore {
    /// The lower bound of the score.
    /// This generally doesn't bring too much benefit,
    /// but still helps optimize a bit.
    LowerBound(isize),
    /// The upper bound of the score,
    /// which helps get rid of many useless branches.
    UpperBound(isize),
}

/// A memoization strategy for a perfect-information sequential game.
///
/// Transposition tables should optimally be a form of hash table.
pub trait TranspositionTable<T: Eq + Hash + Game> {
    /// Get the score of a board, if it exists.
    fn get(&self, board: &T) -> Option<TranspositionTableScore>;

    /// Insert a board into the transposition table.
    fn insert(&mut self, board: T, score: TranspositionTableScore);

    /// Returns true if the board is in the transposition table.
    fn has(&self, board: &T) -> bool;
}

impl<K: Eq + Hash + Game, S: BuildHasher + Default> TranspositionTable<K>
    for HashMap<K, TranspositionTableScore, S>
{
    fn get(&self, board: &K) -> Option<TranspositionTableScore> {
        self.get(board).copied()
    }

    fn insert(&mut self, board: K, score: TranspositionTableScore) {
        self.insert(board, score);
    }

    fn has(&self, board: &K) -> bool {
        self.contains_key(board)
    }
}

#[cfg(feature = "rayon")]
impl<K: Eq + Hash + Game + Sync, S: BuildHasher + Default + Clone + Sync + Send>
    TranspositionTable<K> for Arc<DashMap<K, TranspositionTableScore, S>>
{
    fn get(&self, board: &K) -> Option<TranspositionTableScore> {
        self._get(board).map(|x| *x)
    }

    fn insert(&mut self, board: K, score: TranspositionTableScore) {
        self._insert(board, score);
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
    mut alpha: isize,
    mut beta: isize,
) -> isize {
    if game.is_draw() {
        return 0;
    }

    // check if this is a winning configuration
    for m in &mut game.possible_moves() {
        if game.is_winning_move(&m) {
            let mut board = game.clone();
            board.make_move(&m);
            return board.score() as isize;
        }
    }

    // fetch values from the transposition table
    {
        let score = transposition_table
            .get(game)
            .unwrap_or_else(|| TranspositionTableScore::UpperBound(game.max_score() as isize));

        match score {
            TranspositionTableScore::UpperBound(max) => {
                if beta > max {
                    beta = max;
                    if alpha >= beta {
                        return beta;
                    }
                }
            }
            TranspositionTableScore::LowerBound(min) => {
                if alpha < min {
                    alpha = min;
                    if alpha >= beta {
                        return alpha;
                    }
                }
            }
        };
    }

    // for principal variation search
    let mut first_child = true;

    for m in &mut game.possible_moves() {
        let mut board = game.clone();
        board.make_move(&m);

        let score = if first_child {
            -negamax(&board, transposition_table, -beta, -alpha)
        } else {
            let score = -negamax(&board, transposition_table, -alpha - 1, -alpha);
            if score > alpha {
                -negamax(&board, transposition_table, -beta, -alpha)
            } else {
                score
            }
        };

        // alpha-beta pruning - we can return early
        if score >= beta {
            transposition_table.insert(game.clone(), TranspositionTableScore::LowerBound(score));
            return beta;
        }

        if score > alpha {
            alpha = score;
        }

        first_child = false;
    }

    transposition_table.insert(game.clone(), TranspositionTableScore::UpperBound(alpha));

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
) -> isize {
    let mut alpha = game.min_score();
    let mut beta = game.max_score() as isize + 1;

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
) -> impl Iterator<Item = (T::Move, isize)> + 'a {
    game.possible_moves().map(move |m| {
        let mut board = game.clone();
        board.make_move(&m);
        // We flip the sign of the score because we want the score from the
        // perspective of the player playing the move, not the player whose turn it is.
        (m, -solve(&board, transposition_table))
    })
}

/// Parallelized version of `move_scores`. (faster by a large margin)
/// This requires the `rayon` feature to be enabled.
/// It uses rayon's parallel iterators to evaluate the scores of each move in parallel.
///
/// This also allows you to pass in your own hasher, for transposition table optimization.
///
/// # Returns
///
/// A vector of tuples of the form `(move, score)`.
#[cfg(feature = "rayon")]
pub fn par_move_scores_with_hasher<T>(
    game: &T,
    hasher: impl BuildHasher + Default + Clone + Sync + Send,
) -> Vec<(T::Move, isize)>
where
    T: Game + Clone + Eq + Hash + Sync + Send,
    T::Move: Sync + Send,
{
    // we need to collect it first as we cant parallelize an already non-parallel iterator
    let all_moves = game.possible_moves().collect::<Vec<_>>();
    let hashmap = Arc::new(DashMap::with_hasher(hasher));

    all_moves
        .par_iter()
        .map(move |m| {
            let mut board = game.clone();
            board.make_move(m);
            // We flip the sign of the score because we want the score from the
            // perspective of the player playing the move, not the player whose turn it is.
            let mut map = hashmap.clone();
            ((*m).clone(), -solve(&board, &mut map))
        })
        .collect::<Vec<_>>()
}

/// Parallelized version of `move_scores`. (faster by a large margin)
/// This requires the `rayon` feature to be enabled.
/// It uses rayon's parallel iterators to evaluate the scores of each move in parallel.
///
/// # Returns
///
/// A vector of tuples of the form `(move, score)`.
#[cfg(feature = "rayon")]
pub fn par_move_scores<T>(game: &T) -> Vec<(T::Move, isize)>
where
    T: Game + Clone + Eq + Hash + Sync + Send,
    T::Move: Sync + Send,
{
    use std::collections::hash_map::RandomState;

    par_move_scores_with_hasher(game, RandomState::new())
}
