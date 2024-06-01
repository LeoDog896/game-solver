use clap::Args;
use game_solver::{game::Game, par_move_scores};
use ndarray::IntoDimension;

use crate::tic_tac_toe::{format_dim, TicTacToe};

/// Analyzes Tic Tac Toe.
/// 
#[doc = include_str!("./README.md")]
#[derive(Args)]
pub struct TicTacToeArgs {
    /// The amount of dimensions in the game.
    dimensions: usize,
    /// The size of the board - i.e. with two dimensions
    /// and a size of three, the board would look like
    ///
    /// ```txt
    /// * * *
    /// * * *
    /// * * *
    /// ```
    size: usize,
    /// The moves to make in the game, by dimension and index in that dimension.
    moves: Vec<String>,
}

pub fn main(args: TicTacToeArgs) {
    let mut game = TicTacToe::new(args.dimensions, args.size);

    // parse every move in args, e.g. 0-0 1-1 in args
    args.moves.iter().for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move(&numbers.into_dimension());
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = par_move_scores(&game);

    if game.won() {
        println!("Player {:?} won!", game.player().opponent());
    } else if move_scores.is_empty() {
        println!("No moves left! Game tied!");
    } else {
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("{}, ", format_dim(&game_move));
        }
        println!();
    }
}
