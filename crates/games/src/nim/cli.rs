use std::{env::args, fmt::{Display, Formatter}};

use game_solver::{game::Game, par_move_scores};

use crate::{nim::Nim, util::move_natural::NaturalMove};

impl Display for Nim {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (i, &heap) in self.heaps.iter().enumerate() {
            writeln!(f, "Heap {}: {}", i, heap)?;
        }
        Ok(())
    }
}

pub fn main() {
    // parse the original configuration of the game from args
    // e.g. 3,5,7 for 3 heaps with 3, 5, and 7 objects respectively
    let config = args()
        .nth(1)
        .expect("Please provide a configuration of the game, e.g. 3,5,7")
        .split(',')
        .map(|num| num.parse::<usize>().expect("Not a number!"))
        .collect::<Vec<_>>();

    // create a new game of Nim with the given configuration
    let mut game = Nim::new(config);

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(2).for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move(&NaturalMove([numbers[0], numbers[1]]));
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
            print!("({}), ", game_move);
        }
        println!();
    }
}
