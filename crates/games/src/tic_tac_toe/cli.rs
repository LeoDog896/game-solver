use std::env::args;

use game_solver::{game::Game, par_move_scores};
use ndarray::IntoDimension;

use crate::tic_tac_toe::{format_dim, TicTacToe};

pub fn main() {
    // get the amount of dimensions from the first argument
    let dim = args()
        .nth(1)
        .expect("Please provide a dimension!")
        .parse::<usize>()
        .expect("Not a number!");

    // get the size of the board from the second argument
    let size = args()
        .nth(2)
        .expect("Please provide a game size")
        .parse::<usize>()
        .expect("Not a number!");

    let mut game = TicTacToe::new(dim, size);

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(3).for_each(|arg| {
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
