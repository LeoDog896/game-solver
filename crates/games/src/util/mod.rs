use std::fmt::{Debug, Display};

use game_solver::game::{Game, GameState};
use anyhow::{anyhow, Result};

#[cfg(feature = "egui")]
pub mod gui;
pub mod move_natural;
pub mod move_score;

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
