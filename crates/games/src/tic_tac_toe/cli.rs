use clap::Args;
use game_solver::game::Game;
use ndarray::IntoDimension;
use serde::{Deserialize, Serialize};

use crate::{
    tic_tac_toe::{TicTacToe, TicTacToeMove},
    util::cli::play::play,
};

/// Analyzes Tic Tac Toe.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

impl Default for TicTacToeArgs {
    fn default() -> Self {
        Self {
            dimensions: 2,
            size: 3,
            moves: vec![],
        }
    }
}

pub fn main(args: TicTacToeArgs) {
    let mut game = TicTacToe::new(args.dimensions, args.size);

    // parse every move in args, e.g. 0-0 1-1 in args
    args.moves.iter().for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move(&TicTacToeMove(numbers.into_dimension()));
    });

    print!("{}", game);
    play(game);
}
