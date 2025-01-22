mod human;
mod report;
pub mod robot;

use game_solver::{
    game::{Game, GameState},
    player::{ImpartialPlayer, TwoPlayer},
};
use human::human_output;
use robot::robotic_output;
use std::{
    any::TypeId,
    fmt::{Debug, Display},
    hash::Hash,
};

pub async fn play<
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
    plain: bool,
) where
    T::Move: Sync + Send + Display,
    T::MoveError: Sync + Send + Debug,
{
    match game.state() {
        GameState::Playable => {
            if plain {
                robotic_output(game);
            } else {
                human_output(game).await.unwrap();
            }
        }
        GameState::Tie => println!("No moves left! Thus game is already tied!"),
        GameState::Win(player) => {
            if TypeId::of::<T::Player>() != TypeId::of::<ImpartialPlayer>() {
                println!("The {player:?} player already won this game!");
            } else {
                println!("Player {player:?} already won this game!");
            }
        }
    }
}
