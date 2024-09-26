mod human;
mod report;
mod robot;

use anyhow::{anyhow, Result};
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

pub fn play<
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
                human_output(game).unwrap();
            }
        }
        GameState::Tie => println!("No moves left! Game tied!"),
        GameState::Win(player) => {
            if TypeId::of::<T::Player>() != TypeId::of::<ImpartialPlayer>() {
                println!("The {player:?} player won!");
            } else {
                println!("Player {player:?} won!");
            }
        }
    }
}

pub fn move_failable<T>(game: &mut T, m: &T::Move) -> Result<()>
where
    T: Game,
    T::MoveError: Display,
    T::Player: Debug,
{
    match game.state() {
        GameState::Playable => (),
        GameState::Tie => return Err(anyhow!("Can't continue - game is tied.")),
        GameState::Win(player) => {
            return Err(anyhow!(
                "Can't continue game if player {player:?} already won."
            ))
        }
    };

    game.make_move(m)
        .map_err(|err| anyhow!("Failed to move: {}", err))
}
