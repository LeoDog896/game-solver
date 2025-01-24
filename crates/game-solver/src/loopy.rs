use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use smallvec::SmallVec;

/// We handle loopy games with a custom struct, `LoopyTracker`, which is a
/// HashSet of some state T. This is used to keep track of the states that
/// have been visited, and if a state has been visited, we can handle it appropriately.
/// 
/// `LoopyTracker` should be updated at `Game::make_move` and checked in `Game::state`.
/// 
/// We say `T` is the primary type, and `S` is some representation of `T` without the `LoopyTracker`.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoopyTracker<S: Eq + Hash, T: Eq + Hash> {
    visited: SmallVec<[T; 3]>,
    _phantom: PhantomData<S>,
}

pub trait Loopy<S: Hash + Eq> where Self: Eq + Hash + Sized {
    fn tracker_mut(&mut self) -> &mut LoopyTracker<S, Self>;
    fn tracker(&self) -> &LoopyTracker<S, Self>;

    fn without_tracker(&self) -> S;
}

impl<S: Eq + Hash, T: Eq + Hash + Loopy<S>> LoopyTracker<S, T> {
    /// Create a new `LoopyTracker`.
    pub fn new() -> Self {
        Self {
            visited: SmallVec::new(),
            _phantom: PhantomData,
        }
    }

    /// Check if a state has been visited.
    pub fn has_visited(&self, state: &T) -> bool {
        self.visited.contains(&state)
    }

    /// Mark a state as visited.
    pub fn mark_visited(&mut self, state: T) {
        self.visited.push(state);
    }

    /// The number of states visited.
    pub fn halfmoves(&self) -> usize {
        self.visited.len()
    }

    /// This should be called when an irreversible move is made,
    /// in place of `Self::mark_visited`.
    pub fn clear(&mut self) {
        self.visited.clear();
    }
}

impl<S: Eq + Hash, T: Eq + Hash + Loopy<S>> Default for LoopyTracker<S, T> {
    fn default() -> Self {
        Self::new()
    }
}

// impl<S: Eq + Hash, T: Eq + Hash + Loopy<S>> Hash for LoopyTracker<S, T> {
//     fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
//         for item in self.visited.iter() {
//             item.hash(hasher);
//         }
//     }
// }
