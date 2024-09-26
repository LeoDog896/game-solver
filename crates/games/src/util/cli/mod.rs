use anyhow::{anyhow, Result};
use core::hash::Hash;
use game_solver::{
    game::{score_to_outcome, Game, GameScoreOutcome, GameState},
    par_move_scores,
    player::{ImpartialPlayer, TwoPlayer},
};
use std::{any::TypeId, fmt::{Debug, Display}};

pub fn play<
    T: Game<Player = impl TwoPlayer + Debug + 'static> + Eq + Hash + Sync + Send + Display + 'static,
>(
    game: T,
) where
    T::Move: Sync + Send + Display,
    T::MoveError: Sync + Send + Debug,
{
    print!("{}", game);
    println!();

    match game.state() {
        GameState::Playable => {
            if TypeId::of::<T::Player>() != TypeId::of::<ImpartialPlayer>() {
                println!("Player {:?} to move", game.player());
            } else {
                // TODO: can we assert that game.player() is the next player?
                println!("Impartial game; Next player is moving.");
            }

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
                    match score_to_outcome(&game, score) {
                        GameScoreOutcome::Win(moves) => println!("\n\nWin in {} move{} (score {}):", moves, if moves == 1 { "" } else { "s" }, score),
                        GameScoreOutcome::Loss(moves) => println!("\n\nLose in {} move{} (score {}):", moves, if moves == 1 { "" } else { "s" }, score),
                        GameScoreOutcome::Tie => println!("\n\nTie with the following moves:")
                    }
                    current_move_score = Some(score);
                }
                print!("{}, ", &game_move);
            }
            println!();
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
        GameState::Win(player) => return Err(anyhow!(
            "Can't continue game if player {player:?} already won."
        )),
    };

    game.make_move(m)
        .map_err(|err| anyhow!("Failed to move: {}", err))
}
