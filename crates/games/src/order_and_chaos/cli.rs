use clap::Args;
use game_solver::{game::Game, par_move_scores};

use crate::order_and_chaos::{CellType, OrderAndChaos};

/// Analyzes Order and Chaos.
///
#[doc = include_str!("./README.md")]
#[derive(Args)]
pub struct OrderAndChaosArgs {
    moves: Vec<String>,
}

pub fn main(args: OrderAndChaosArgs) {
    // create a new game of Nim with the given configuration
    let mut game = OrderAndChaos::new();

    // parse every move in args, e.g. 0-0-x 1-1-o in args
    args.moves.iter().for_each(|arg| {
        let args: Vec<&str> = arg.split('-').collect();

        let numbers = args[0..2]
            .iter()
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect::<Vec<_>>();

        let player = match args[2] {
            "x" => CellType::X,
            "o" => CellType::O,
            _ => panic!("Invalid player!"),
        };

        assert_eq!(args.len(), 3);

        let move_to_make = ((numbers[0], numbers[1]), player);
        if let Some(player) = game.is_winning_move(&move_to_make) {
            panic!("Player {:?} won!", player);
        } else {
            game.make_move(&move_to_make);
        }
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = par_move_scores(&game);

    // check for the win condition
    if move_scores.is_empty() {
        println!("Player {:?} won!", game.player().opponent());
    } else {
        // sort for the best moves first
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }

            let chr = match game_move.1 {
                CellType::X => "x",
                CellType::O => "o",
                CellType::Empty => panic!("Invalid player!"),
            };

            print!("(({}, {}), {}), ", game_move.0 .0, game_move.0 .1, chr);
        }
        println!();
    }
}
