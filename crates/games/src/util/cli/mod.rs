use anyhow::{anyhow, Result};
use core::hash::Hash;
use game_solver::{
    game::{Game, GameState, ZeroSumPlayer},
    par_move_scores,
};
use std::fmt::{Debug, Display};

pub fn play<T>(game: T)
where
    T: Game<Player = ZeroSumPlayer> + Eq + Hash + Sync + Send + Display + 'static,
    T::Move: Sync + Send + Display,
    T::MoveError: Sync + Send + Debug,
{
    print!("{}", game);
    println!();

    match game.state() {
        GameState::Playable => {
            println!("Player {:?} to move", game.player());

            let move_scores = par_move_scores(&game);
            let mut move_scores = move_scores
                .into_iter()
                .collect::<Result<Vec<_>, T::MoveError>>()
                .unwrap();

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
        GameState::Tie => println!("No moves left! Game tied!"),
        GameState::Win(player) => println!("Player {player:?} won!"),
    }
}

pub fn move_failable<T>(game: &mut T, m: &T::Move) -> Result<()>
where
    T: Game,
    T::MoveError: Display,
    T::Player: Debug,
{
    game.make_move(m)
        .map_err(|err| anyhow!("Failed to move: {}", err))?;

    match game.state() {
        GameState::Playable => Ok(()),
        GameState::Tie => Err(anyhow!("Can't continue - game is tied.")),
        GameState::Win(player) => Err(anyhow!(
            "Can't continue game if player {player:?} already won."
        )),
    }
}
