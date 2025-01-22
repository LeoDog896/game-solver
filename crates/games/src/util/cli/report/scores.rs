use std::fmt::{Debug, Display};

use game_solver::{
    game::{score_to_outcome, Game, GameScoreOutcome},
    CollectedMoves, GameSolveError,
};

use crate::util::move_score::normalize_move_scores;

pub fn show_scores<T: Game + Debug>(game: &T, move_scores: CollectedMoves<T>)
where
    T::Move: Display,
{
    let move_scores = normalize_move_scores::<T>(move_scores).unwrap_or_else(|err| {
        match err {
            GameSolveError::CancellationTokenError => {
                eprintln!("Search was cancelled.");
            },
            GameSolveError::MoveError(err) => {
                eprintln!("Error making move: {:?}", err);
            },
        }
        vec![]
    });

    let mut current_move_score = None;
    for (game_move, score) in move_scores {
        if current_move_score != Some(score) {
            match score_to_outcome(game, score) {
                GameScoreOutcome::Win(moves) => println!(
                    "\n\nWin in {} move{} (score {}):",
                    moves,
                    if moves == 1 { "" } else { "s" },
                    score
                ),
                GameScoreOutcome::Loss(moves) => println!(
                    "\n\nLose in {} move{} (score {}):",
                    moves,
                    if moves == 1 { "" } else { "s" },
                    score
                ),
                GameScoreOutcome::Tie => println!("\n\nTie with the following moves:"),
            }
            current_move_score = Some(score);
        }
        print!("{}, ", &game_move);
    }
    println!();
}
