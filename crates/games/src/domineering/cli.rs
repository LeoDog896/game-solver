#[doc = include_str!("./README.md")]
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use clap::Args;
use game_solver::{game::Game, move_scores};
use serde::{Deserialize, Serialize};

use crate::domineering::DomineeringGame;

use super::Domineering;

impl<const WIDTH: usize, const HEIGHT: usize> Display for Domineering<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                if *self.board.get(j, i).unwrap() {
                    write!(f, "X")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Analyzes Domineering.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DomineeringArgs {
    moves: Vec<String>,
}

impl Default for DomineeringArgs {
    fn default() -> Self {
        Self { moves: vec![] }
    }
}

pub fn main(args: DomineeringArgs) {
    let mut game = DomineeringGame::new();

    // parse every move in args, e.g. 0-0 1-1 in args
    args.moves.iter().for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move(&(numbers[0], numbers[1]));
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();

    if !move_scores.is_empty() {
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("({}, {}), ", game_move.0, game_move.1);
        }
        println!();
    } else {
        println!("Player {:?} won!", game.player().opponent());
    }
}
