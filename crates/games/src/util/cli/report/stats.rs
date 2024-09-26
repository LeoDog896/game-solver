use std::sync::atomic::Ordering;

use game_solver::{game::Game, stats::Stats};

pub fn show_stats<T: Game>(
    stats: &Stats<T::Player>,
    // plain: bool
) {
    println!("Stats: ");
    println!();
    println!("States explored: {}", stats.states_explored.load(Ordering::SeqCst));
    println!("Max depth:       {}", stats.max_depth.load(Ordering::SeqCst));
    println!("Cache hits:      {}", stats.cache_hits.load(Ordering::SeqCst));
    println!("Pruning cutoffs: {}", stats.pruning_cutoffs.load(Ordering::SeqCst));
    println!("End nodes:");
    println!("\tWinning: {}", stats.terminal_ends.winning.load(Ordering::SeqCst));
    println!("\tLosing:  {}", stats.terminal_ends.losing.load(Ordering::SeqCst));
    println!("\tTies:    {}", stats.terminal_ends.tie.load(Ordering::SeqCst));
    println!();
}
