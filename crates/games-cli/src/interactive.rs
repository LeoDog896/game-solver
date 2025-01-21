use std::{fmt::{Debug, Display}, str::FromStr};

use game_solver::{game::Game, player::TwoPlayer};
use games::util::cli::{move_failable, robot::announce_player};

use owo_colors::OwoColorize;

use dialoguer::{theme::ColorfulTheme, Input};

pub fn play_interactive<
    T: Game<Player = impl TwoPlayer + Debug + 'static>
        + Display
>(
    mut game: T
) where <T as Game>::Move: FromStr + Debug, <<T as Game>::Move as FromStr>::Err: Debug {
    loop {
        print!("{}", game);
        println!();

        announce_player(&game);

        let game_move: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Move")
            .interact_text()
            .unwrap();

        match FromStr::from_str(&game_move) {
            Ok(game_move) => {
                if let Err(err) = move_failable(&mut game, &game_move) {
                    clearscreen::clear().expect("failed to clear screen");
                    println!("{}", format!("Failed to make move {game_move:?}: {err:?}").red());
                    continue;
                }
            },
            Err(err) => {
                clearscreen::clear().expect("failed to clear screen");
                println!("{}", format!("Invalid move {game_move}: {err:?}").red());
                continue;
            }
        }

        clearscreen::clear().expect("failed to clear screen");
    }
}
