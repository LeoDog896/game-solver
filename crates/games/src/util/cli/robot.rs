use game_solver::{
    game::{score_to_outcome, Game, GameScoreOutcome},
    par_move_scores,
    player::{ImpartialPlayer, TwoPlayer},
};
use std::{
    any::TypeId,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::util::{cli::report::scores::show_scores, move_score::normalize_move_scores};

pub fn robotic_output<
    T: Game<Player = impl TwoPlayer + Debug + Sync + 'static>
        + Eq
        + Hash
        + Sync
        + Send
        + Display
        + Debug
        + 'static,
>(
    game: T,
) where
    T::Move: Sync + Send + Display,
    T::MoveError: Sync + Send + Debug,
{
    print!("{}", game);
    println!();

    if TypeId::of::<T::Player>() != TypeId::of::<ImpartialPlayer>() {
        println!("Player {:?} to move", game.player());
    } else {
        // TODO: can we assert that game.player() is the next player?
        println!("Impartial game; Next player is moving.");
    }

    let move_scores = par_move_scores(&game, None, &None);

    show_scores(&game, move_scores);
}
