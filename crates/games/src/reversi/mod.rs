#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;

use anyhow::Error;
use array2d::Array2D;
use clap::Args;
use game_solver::game::{Game, GameState, Player, ZeroSumPlayer};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

use crate::util::{cli::move_failable, move_natural::NaturalMove};

pub const WIDTH: usize = 6;
pub const HEIGHT: usize = 6;

pub type ReversiMove = NaturalMove<2>;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Reversi {
    /// None if empty, Some(Player) if occupied
    board: Array2D<Option<ZeroSumPlayer>>,
    move_count: usize,
}

impl Reversi {
    fn new() -> Self {
        let mut board = Array2D::filled_with(None, WIDTH, HEIGHT);

        // set middle squares to occupied:
        board
            .set(WIDTH / 2 - 1, HEIGHT / 2 - 1, Some(ZeroSumPlayer::One))
            .unwrap();
        board
            .set(WIDTH / 2, HEIGHT / 2, Some(ZeroSumPlayer::One))
            .unwrap();
        board
            .set(WIDTH / 2 - 1, HEIGHT / 2, Some(ZeroSumPlayer::Two))
            .unwrap();
        board
            .set(WIDTH / 2, HEIGHT / 2 - 1, Some(ZeroSumPlayer::Two))
            .unwrap();

        Self {
            board,
            move_count: 0,
        }
    }

    fn on_board(&self, x: usize, y: usize) -> bool {
        x < WIDTH && y < HEIGHT
    }

    fn is_valid_move(&self, m: &<Self as Game>::Move) -> Option<Vec<<Self as Game>::Move>> {
        let cell = *self.board.get(m.0[0], m.0[1]).unwrap();

        if cell.is_some() {
            return None;
        }

        let opposing_tile = self.player().next();

        let mut tiles_to_flip = Vec::new();

        let directions: &[(isize, isize)] = &[
            (0, 1),
            (1, 1),
            (1, 0),
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ];

        for (x_dir, y_dir) in directions {
            let mut x = m.0[0];
            let mut y = m.0[1];

            x = x.wrapping_add_signed(*x_dir);
            y = y.wrapping_add_signed(*y_dir);

            if self.board.get(x, y) != Some(&Some(opposing_tile)) {
                continue;
            }

            x = x.wrapping_add_signed(*x_dir);
            y = y.wrapping_add_signed(*y_dir);

            while self.board.get(x, y) == Some(&Some(opposing_tile)) {
                x = x.wrapping_add_signed(*x_dir);
                y = y.wrapping_add_signed(*y_dir);

                if !self.on_board(x, y) {
                    break;
                }
            }

            if !self.on_board(x, y) {
                continue;
            }

            if self.board.get(x, y) == Some(&Some(self.player())) {
                loop {
                    x = x.checked_add_signed(-*x_dir).unwrap();
                    y = y.checked_add_signed(-*y_dir).unwrap();

                    if x == m.0[0] && y == m.0[1] {
                        break;
                    }

                    tiles_to_flip.push(NaturalMove([x, y]));
                }
            }
        }

        if tiles_to_flip.is_empty() {
            None
        } else {
            Some(tiles_to_flip)
        }
    }
}

impl Game for Reversi {
    type Move = ReversiMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    type Player = ZeroSumPlayer;
    type MoveError = array2d::Error;

    fn max_moves(&self) -> Option<usize> {
        Some(WIDTH * HEIGHT)
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn player(&self) -> ZeroSumPlayer {
        if self.move_count % 2 == 0 {
            ZeroSumPlayer::One
        } else {
            ZeroSumPlayer::Two
        }
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        let move_set = self.is_valid_move(m).unwrap();

        self.board.set(m.0[0], m.0[1], Some(self.player())).unwrap();

        for idx in move_set {
            self.board.set(idx.0[0], idx.0[1], Some(self.player()))?;
        }

        self.move_count += 1;

        Ok(())
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if self.is_valid_move(&NaturalMove([x, y])).is_some() {
                    moves.push(NaturalMove([x, y]));
                }
            }
        }
        moves.into_iter()
    }

    fn state(&self) -> GameState<Self::Player> {
        if self.possible_moves().len() > 0 {
            return GameState::Playable;
        }

        let mut player_one_count = 0;
        let mut player_two_count = 0;

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                match *self.board.get(x, y).unwrap() {
                    Some(ZeroSumPlayer::One) => player_one_count += 1,
                    Some(ZeroSumPlayer::Two) => player_two_count += 1,
                    None => (),
                }
            }
        }

        match player_one_count.cmp(&player_two_count) {
            std::cmp::Ordering::Greater => GameState::Win(ZeroSumPlayer::One),
            std::cmp::Ordering::Less => GameState::Win(ZeroSumPlayer::Two),
            std::cmp::Ordering::Equal => GameState::Tie,
        }
    }
}

fn player_to_char(player: Option<ZeroSumPlayer>) -> char {
    match player {
        Some(ZeroSumPlayer::One) => 'X',
        Some(ZeroSumPlayer::Two) => 'O',
        None => '-',
    }
}

impl fmt::Display for Reversi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Current player: {}", player_to_char(Some(self.player())))?;

        let moves = self.possible_moves().collect::<Vec<_>>();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let character = if moves.contains(&NaturalMove([x, y])) {
                    '*'
                } else {
                    player_to_char(*self.board.get(x, y).unwrap())
                };

                write!(f, "{}", character)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

/// Analyzes Reversi.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct ReversiArgs {
    /// Reversi moves, ordered as x1-y1 x2-y2 ...
    #[arg(value_parser = clap::value_parser!(ReversiMove))]
    moves: Vec<ReversiMove>,
}

impl TryFrom<ReversiArgs> for Reversi {
    type Error = Error;

    fn try_from(value: ReversiArgs) -> Result<Self, Self::Error> {
        let mut game = Reversi::new();

        // parse every move in args, e.g. 0-0 1-1 in args
        for game_move in value.moves {
            move_failable(&mut game, &game_move)?;
        }

        Ok(game)
    }
}
