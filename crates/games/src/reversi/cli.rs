use std::fmt;

use crate::{
    reversi::{Reversi, ReversiMove},
    util::move_natural::NaturalMove,
};
use clap::Args;
use game_solver::{
    game::{Game, ZeroSumPlayer},
    par_move_scores,
};
use serde::{Deserialize, Serialize};

use super::{HEIGHT, WIDTH};

/// Analyzes Reversi.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ReversiArgs {
    /// Reversi moves, ordered as x1-y1 x2-y2 ...
    #[arg(value_parser = clap::value_parser!(ReversiMove))]
    moves: Vec<ReversiMove>,
}

impl Default for ReversiArgs {
    fn default() -> Self {
        Self {
            moves: vec![]
        }
    }
}

fn player_to_char(player: Option<ZeroSumPlayer>) -> char {
    match player {
        Some(ZeroSumPlayer::One) => 'X',
        Some(ZeroSumPlayer::Two) => 'O',
        None => '-',
    }
}

impl fmt::Display for Reversi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Current player: {}", player_to_char(Some(self.player())))?;

        let moves = self.possible_moves().collect::<Vec<_>>();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let character = if moves.contains(&NaturalMove([x, y])) {
                    '*'
                } else {
                    player_to_char(*self.board.get(x, y).unwrap())
                };

                write!(f, "{}", character)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

pub fn main(args: ReversiArgs) {
    let mut game = Reversi::new();

    // parse every move in args, e.g. 0-0 1-1 in args
    args.moves.iter().for_each(|game_move| {
        game.make_move(game_move);
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = par_move_scores(&game);

    if move_scores.is_empty() {
        game.winning_player().map_or_else(
            || {
                println!("Game tied!");
            },
            |player| {
                println!("Player {:?} won!", player.opponent());
            },
        )
    } else {
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("{}, ", game_move);
        }
        println!();
    }
}
