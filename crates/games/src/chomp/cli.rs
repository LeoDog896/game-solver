use std::collections::HashMap;

use clap::Args;
use game_solver::{game::Game, move_scores};
use serde::{Deserialize, Serialize};

use crate::chomp::Chomp;

use super::ChompMove;

/// Analyzes Chomp.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ChompArgs {
    /// The width of the game
    #[arg(short, long, default_value_t = 6)]
    width: usize,
    /// The height of the game
    #[arg(short, long, default_value_t = 4)]
    height: usize,
    /// Chomp moves, ordered as x1-y1 x2-y2 ...
    #[arg(value_parser = clap::value_parser!(ChompMove))]
    moves: Vec<ChompMove>,
}

impl Default for ChompArgs {
    fn default() -> Self {
        Self {
            width: 6,
            height: 4,
            moves: vec![],
        }
    }
}

pub fn main(args: ChompArgs) {
    let mut game = Chomp::new(args.width, args.height);

    // parse every move in args, e.g. 0-0 1-1 in args
    args.moves.iter().for_each(|arg| {
        game.make_move(arg);
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();

    if move_scores.is_empty() {
        println!("Player {:?} won!", game.player().opponent());
    } else {
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("({}), ", game_move);
        }
        println!();
    }
}
