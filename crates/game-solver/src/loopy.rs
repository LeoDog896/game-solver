use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashSet;

/// We handle loopy games with a custom struct, `LoopyTracker`, which is a
/// HashSet of some state T. This is used to keep track of the states that
/// have been visited, and if a state has been visited, we can handle it appropriately.
/// 
/// `LoopyTracker` should be updated at `Game::make_move` and checked in `Game::state`.

#[derive(Debug, Clone)]
pub struct LoopyTracker<T: Eq + Hash> {
    visited: HashSet<T>,
}

impl<T: Eq + Hash> LoopyTracker<T> {
    /// Create a new `LoopyTracker`.
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
        }
    }

    /// Check if a state has been visited.
    pub fn has_visited(&self, state: &T) -> bool {
        self.visited.contains(state)
    }

    /// Mark a state as visited.
    pub fn mark_visited(&mut self, state: T) {
        self.visited.insert(state);
    }

    /// The number of states visited.
    pub fn age(&self) -> usize {
        self.visited.len()
    }
}

impl<T: Eq + Hash> Default for LoopyTracker<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> PartialEq for LoopyTracker<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T: Eq + Hash> Eq for LoopyTracker<T> {}

impl<T: Eq + Hash> Hash for LoopyTracker<T> {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        for item in self.visited.iter() {
            item.hash(hasher);
        }
    }
}
