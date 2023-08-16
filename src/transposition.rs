//! Transposition tables for memoization.

#[cfg(feature = "rayon")]
use {
    dashmap::{DashMap, Map},
    std::sync::Arc,
};

use crate::game::Game;

use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

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
