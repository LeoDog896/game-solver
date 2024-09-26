use std::sync::atomic::{AtomicU64, AtomicUsize};

use crate::player::Player;

#[derive(Debug)]
pub struct TerminalEnds {
    pub winning: AtomicU64,
    pub tie: AtomicU64,
    pub losing: AtomicU64,
}

impl Default for TerminalEnds {
    fn default() -> Self {
        Self {
            winning: AtomicU64::new(0),
            tie: AtomicU64::new(0),
            losing: AtomicU64::new(0),
        }
    }
}

#[derive(Debug)]
pub struct Stats<P: Player> {
    pub states_explored: AtomicU64,
    pub max_depth: AtomicUsize,
    pub cache_hits: AtomicU64,
    pub pruning_cutoffs: AtomicU64,
    pub terminal_ends: TerminalEnds,
    pub original_player: P,
    pub original_move_count: usize,
}
