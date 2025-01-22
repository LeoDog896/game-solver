use game_solver::{
    game::Game,
    par_move_scores,
    player::{ImpartialPlayer, TwoPlayer},
};
use std::{
    any::TypeId,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::report::scores::show_scores;

pub fn announce_player<T: Game<Player = impl TwoPlayer + Debug + 'static>>(game: &T) {
    if TypeId::of::<T::Player>() != TypeId::of::<ImpartialPlayer>() {
        println!("Player {:?} to move", game.player());
    } else {
        // TODO: can we assert that game.player() is the next player?
        println!("Impartial game; Next player is moving.");
    }
}

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

    announce_player(&game);

    let move_scores = par_move_scores(&game, None);

    show_scores(&game, move_scores);
}
