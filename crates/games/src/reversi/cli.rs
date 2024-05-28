use std::str::FromStr;

use clap::Args;
use itertools::Itertools;
use crate::reversi::{Reversi, ReversiMove};
use game_solver::{game::Game, par_move_scores};

#[derive(Args)]
pub struct ReversiArgs {
    /// Reversi moves, ordered as x1-y1 x2-y2 ...
    #[arg(value_parser = clap::value_parser!(ReversiMove))]
    moves: Vec<ReversiMove>
}

impl FromStr for ReversiMove {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers = s.split("-").collect::<Vec<_>>();

        if numbers.len() != 2 {
            return Err("Must be two numbers separated by a hyphen (x-y), i.e. 2-6".to_string());
        }

        let numbers = numbers.iter()
            .map(|num| num.parse::<usize>())
            .collect::<Vec<_>>();

        if let Some((position, _)) = numbers.iter().find_position(|x| x.is_err()) {
            let position = if position == 0 {
                "first"
            } else {
                "second"
            };
            
            return Err(format!("The {} number is not a number.", position));
        }
        
        Ok(ReversiMove((
            numbers[0].clone().unwrap(),
            numbers[1].clone().unwrap()
        )))
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
            print!("{:?}, ", game_move);
        }
        println!();
    }
}
