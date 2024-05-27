use crate::games::reversi::Reversi;
use game_solver::{game::Game, par_move_scores};
use std::env::args;

pub fn main() {
    let mut game = Reversi::new();

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(1).for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move(&(numbers[0], numbers[1]));
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
