//! Transposition tables for memoization.

#[cfg(feature = "rayon")]
use {
    moka::sync::Cache,
    sysinfo::SystemExt,
    std::sync::Arc
};

use crate::game::Game;

use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

/// A score in a transposition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
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
    fn get(&self, board: &T) -> Option<Score>;

    /// Insert a board into the transposition table.
    fn insert(&mut self, board: T, score: Score);

    /// Returns true if the board is in the transposition table.
    fn has(&self, board: &T) -> bool;
}

impl<K: Eq + Hash + Game, S: BuildHasher + Default> TranspositionTable<K>
    for HashMap<K, Score, S>
{
    fn get(&self, board: &K) -> Option<Score> {
        self.get(board).copied()
    }

    fn insert(&mut self, board: K, score: Score) {
        self.insert(board, score);
    }

    fn has(&self, board: &K) -> bool {
        self.contains_key(board)
    }
}

/// Complex transposition table that uses an underlying concurrent LFU cache,
/// powered by [moka](https://github.com/moka-rs/moka).
#[cfg(feature = "rayon")]
pub struct TranspositionCache<K: Eq + Hash + Game + Send + Sync + 'static, S: BuildHasher + Default>(Cache<K, Score, S>);

#[cfg(feature = "rayon")]
impl<K: Eq + Hash + Game + Send + Sync, S: BuildHasher + Default + Send + Sync + Clone + 'static> TranspositionCache<K, S> {
    /// Create a new transposition cache with the given capacity and hasher.
    pub fn with_capacity(capacity: u64) -> Self {
        Self(Cache::builder()
            .weigher(|_key, _value: &Score | -> u32 {
                // get the memory size of the score
                std::mem::size_of::<Score>() as u32
            })
            .max_capacity(capacity)
            .build_with_hasher(S::default()))
    }

    /// Create a new transposition cache with an estimated 3/4ths of the remaining memory.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(
            sysinfo::System::new_all().free_memory() * 3 / 4,
        )
    }
}

#[cfg(feature = "rayon")]
impl<K: Eq + Hash + Game + Send + Sync, S: BuildHasher + Default + Send + Sync + Clone + 'static> Default for TranspositionCache<K, S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rayon")]
impl<K: Eq + Hash + Game + Send + Sync + 'static, S: BuildHasher + Default + Send + Sync + Clone + 'static> TranspositionTable<K>
    for Arc<TranspositionCache<K, S>>
{
    fn get(&self, board: &K) -> Option<Score> {
        self.0.get(board)
    }

    fn insert(&mut self, board: K, score: Score) {
        self.0.insert(board, score);
    }

    fn has(&self, board: &K) -> bool {
        self.0.contains_key(board)
    }
}