use game_solver::{
    game::Game,
    par_move_scores,
    player::{ImpartialPlayer, TwoPlayer},
};
use tokio_util::sync::CancellationToken;
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

pub async fn robotic_output<
    T: Game<Player = impl TwoPlayer + Debug + Sync + Send + 'static>
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

    let cancellation_token = CancellationToken::new();

    // on CTRL+C, cancel the game solving thread
    let exit = cancellation_token.clone();
    let ctrl_c = tokio::signal::ctrl_c();
    let handle = tokio::spawn(async move {
        ctrl_c.await.expect("Failed to listen for Ctrl+C");
        println!("Cancelling...");
        exit.cancel();
    });

    let move_scores = par_move_scores(&game, None, Some(cancellation_token)).await;

    show_scores(&game, move_scores);

    handle.abort();
}
