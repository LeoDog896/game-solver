use core::hash::Hash;
use game_solver::{
    game::{Game, ZeroSumPlayer},
    par_move_scores,
};
use std::fmt::Display;

use crate::util::state::{GameState, State};

pub fn play<T>(game: T)
where
    T: Game<Player = ZeroSumPlayer> + Clone + Eq + Hash + Sync + Send + GameState + 'static,
    T::Move: Sync + Send + Display,
{
    println!("Player {:?} to move", game.player());

    let mut move_scores = par_move_scores(&game);

    if game.state() == State::Continuing {
        println!("Player {:?} won!", game.player().opponent());
    } else if move_scores.is_empty() {
        println!("No moves left! Game tied!");
    } else {
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("{}, ", &game_move);
        }
        println!();
    }
}
