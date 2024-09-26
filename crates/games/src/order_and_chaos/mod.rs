#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::{anyhow, Error};
use array2d::Array2D;
use clap::Args;
use game_solver::{
    game::{Game, GameState, StateType},
    player::PartizanPlayer,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};
use thiserror::Error;

use crate::util::cli::move_failable;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum CellType {
    X,
    O,
}

impl Display for CellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::O => "O",
                Self::X => "X",
            }
        )
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct OrderAndChaos<
    const WIDTH: usize,
    const HEIGHT: usize,
    const MIN_WIN_LENGTH: usize,
    const MAX_WIN_LENGTH: usize,
> {
    board: Array2D<Option<CellType>>,
    move_count: usize,
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const MIN_WIN_LENGTH: usize,
        const MAX_WIN_LENGTH: usize,
    > Default for OrderAndChaos<WIDTH, HEIGHT, MIN_WIN_LENGTH, MAX_WIN_LENGTH>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const MIN_WIN_LENGTH: usize,
        const MAX_WIN_LENGTH: usize,
    > OrderAndChaos<WIDTH, HEIGHT, MIN_WIN_LENGTH, MAX_WIN_LENGTH>
{
    /// Create a new game of Nim with the given heaps,
    /// where heaps is a list of the number of objects in each heap.
    pub fn new() -> Self {
        assert!(MIN_WIN_LENGTH <= MAX_WIN_LENGTH, "MIN > MAX win length?");
        // [a, b][(a < b) as usize] is essentially the max function: https://stackoverflow.com/a/53646925/7589775
        assert!(
            MAX_WIN_LENGTH <= [WIDTH, HEIGHT][(WIDTH < HEIGHT) as usize],
            "Win length should not be "
        );

        Self {
            board: Array2D::filled_with(None, HEIGHT, WIDTH),
            move_count: 0,
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum OrderAndChaosMoveError {
    #[error("Can not make move {played:?} as it is out of bounds of (w:{width},h:{height})")]
    OutOfBounds {
        played: (usize, usize),
        width: usize,
        height: usize,
    },
    #[error("There is already a filled in value present at {0:?}.")]
    AlreadyPresent((usize, usize)),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderAndChaosMove(((usize, usize), CellType));

impl Display for OrderAndChaosMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ ({}, {})", self.0 .1, self.0 .0 .0, self.0 .0 .1)
    }
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const MIN_WIN_LENGTH: usize,
        const MAX_WIN_LENGTH: usize,
    > Game for OrderAndChaos<WIDTH, HEIGHT, MIN_WIN_LENGTH, MAX_WIN_LENGTH>
{
    /// where Move is a tuple of:
    /// ((row, column), player)
    type Move = OrderAndChaosMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    /// Define Order and Chaos as a zero-sum game,
    /// where Left is Order,
    /// and Right is Chaos
    type Player = PartizanPlayer;
    type MoveError = OrderAndChaosMoveError;

    const STATE_TYPE: Option<StateType> = None;

    fn max_moves(&self) -> Option<usize> {
        Some(WIDTH * HEIGHT)
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        let ((row, column), player) = m.0;
        // check for indexing OOB
        if row >= HEIGHT || column >= WIDTH {
            return Err(OrderAndChaosMoveError::OutOfBounds {
                played: m.0 .0,
                width: self.board.num_columns(),
                height: self.board.num_rows(),
            });
        }

        // check if the cell is empty
        if self.board[(row, column)].is_some() {
            return Err(OrderAndChaosMoveError::AlreadyPresent(m.0 .0));
        }

        // make the move
        self.board[(row, column)] = Some(player);
        self.move_count += 1;

        Ok(())
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();

        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                if self.board[(row, column)].is_none() {
                    moves.push(OrderAndChaosMove(((row, column), CellType::X)));
                    moves.push(OrderAndChaosMove(((row, column), CellType::O)));
                }
            }
        }

        moves.into_iter()
    }

    fn state(&self) -> GameState<Self::Player> {
        // we need at least MIN_WIN_LENGTH plays to get a win
        if self.move_count < MIN_WIN_LENGTH {
            return GameState::Playable;
        }

        // check every horizontal row
        for i in 0..HEIGHT {
            // for the first (some) elements in that row
            // TODO: this will not work if min_width_length > width! can we consider this?
            'row_check: for j in 0..=(WIDTH - MIN_WIN_LENGTH) {
                // find a piece? lets see if it continues going
                if let Some(cell_type) = self.board[(j, i)] {
                    for k in (j + 1)..((MIN_WIN_LENGTH + j).min(WIDTH)) {
                        if let Some(found_cell_type) = self.board[(k, i)] {
                            if cell_type == found_cell_type {
                                continue;
                            } else {
                                break 'row_check;
                            }
                        } else {
                            break 'row_check;
                        }
                    }

                    return GameState::Win(PartizanPlayer::Left);
                }
            }
        }

        // check every column
        for i in 0..WIDTH {
            'column_check: for j in 0..=(HEIGHT - MIN_WIN_LENGTH) {
                // find a piece? see if it continues going
                if let Some(cell_type) = self.board[(i, j)] {
                    for k in (j + 1)..((MIN_WIN_LENGTH + j).min(HEIGHT)) {
                        if let Some(found_cell_type) = self.board[(i, k)] {
                            if cell_type == found_cell_type {
                                continue;
                            } else {
                                break 'column_check;
                            }
                        } else {
                            break 'column_check;
                        }
                    }

                    return GameState::Win(PartizanPlayer::Left);
                }
            }
        }

        // check every diag - we can essentially
        // check every value in the top right corner
        for i in 0..=(WIDTH - MIN_WIN_LENGTH) {
            'diag_check: for j in 0..=(HEIGHT - MIN_WIN_LENGTH) {
                if let Some(cell_type) = self.board[(i, j)] {
                    // found a cell! lets continue going down!
                    for k in 1..MIN_WIN_LENGTH.min(WIDTH).min(HEIGHT) {
                        if let Some(found_cell_type) = self.board[(i + k, j + k)] {
                            if found_cell_type == cell_type {
                                continue;
                            } else {
                                break 'diag_check;
                            }
                        } else {
                            break 'diag_check;
                        }
                    }

                    return GameState::Win(PartizanPlayer::Left);
                }
            }
        }

        if self.move_count == WIDTH * HEIGHT {
            return GameState::Win(PartizanPlayer::Right);
        }

        GameState::Playable
    }

    fn player(&self) -> PartizanPlayer {
        if self.move_count % 2 == 0 {
            PartizanPlayer::Left
        } else {
            PartizanPlayer::Right
        }
    }
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const MIN_WIN_LENGTH: usize,
        const MAX_WIN_LENGTH: usize,
    > Display for OrderAndChaos<WIDTH, HEIGHT, MIN_WIN_LENGTH, MAX_WIN_LENGTH>
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                match self.board[(row, column)] {
                    Some(CellType::X) => write!(f, "X")?,
                    Some(CellType::O) => write!(f, "O")?,
                    None => write!(f, "-")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Analyzes Order and Chaos.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct OrderAndChaosArgs {
    moves: Vec<String>,
}

impl<
        const WIDTH: usize,
        const HEIGHT: usize,
        const MIN_WIN_LENGTH: usize,
        const MAX_WIN_LENGTH: usize,
    > TryFrom<OrderAndChaosArgs> for OrderAndChaos<WIDTH, HEIGHT, MIN_WIN_LENGTH, MAX_WIN_LENGTH>
{
    type Error = Error;

    fn try_from(value: OrderAndChaosArgs) -> Result<Self, Self::Error> {
        let mut game = OrderAndChaos::new();

        // parse every move in args, e.g. 0-0-x 1-1-o in args
        for arg in value.moves {
            let args: Vec<&str> = arg.split('-').collect();

            let numbers = args[0..2]
                .iter()
                .map(|num| num.parse::<usize>().expect("Not a number!"))
                .collect::<Vec<_>>();

            let player = match args[2].to_ascii_lowercase().as_str() {
                "x" => Ok(CellType::X),
                "o" => Ok(CellType::O),
                _ => Err(anyhow!("Invalid player!")),
            }?;

            assert_eq!(args.len(), 3);

            let move_to_make = OrderAndChaosMove(((numbers[0], numbers[1]), player));
            move_failable(&mut game, &move_to_make)?;
        }

        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_string(string: &str) -> OrderAndChaos<6, 6, 5, 6> {
        let board_internal = string
            .chars()
            .map(|ch| match ch {
                'X' => Some(Some(CellType::X)),
                'O' => Some(Some(CellType::O)),
                '.' => Some(None),
                '\n' => None,
                _ => panic!("There shouldn't be other characters in the string!"),
            })
            .filter_map(|x| x)
            .collect::<Vec<_>>();

        let element_count = board_internal.iter().filter(|x| x.is_some()).count();

        let board = Array2D::from_row_major(&board_internal, 6, 6).unwrap();

        OrderAndChaos {
            board,
            move_count: element_count,
        }
    }

    #[test]
    fn win_empty() {
        let empty_board = from_string(
            "......\
        ......\
        ......\
        ......\
        ......\
        ......",
        );

        assert_eq!(empty_board.state(), GameState::Playable);
    }

    #[test]
    fn lose_horizontal_tiny() {
        let horizontal_board = from_string(
            "......\
        .XOXXX\
        .X....\
        .OOOO.\
        ......\
        ......",
        );

        assert_eq!(
            horizontal_board.state(),
            GameState::Playable
        );
    }

    #[test]
    fn win_horizontal() {
        let horizontal_board = from_string(
            "......\
        .XOXXX\
        .X....\
        .OOOOO\
        ......\
        ......",
        );

        assert_eq!(
            horizontal_board.state(),
            GameState::Win(PartizanPlayer::Left)
        );
    }

    #[test]
    fn win_vertical() {
        let vertical_board = from_string(
            "......\
        .XOXXX\
        .X.X..\
        .OOXOO\
        ...X..\
        ...X..",
        );

        assert_eq!(vertical_board.state(), GameState::Win(PartizanPlayer::Left));
    }

    #[test]
    fn lose_vertical_tiny() {
        let vertical_board = from_string(
            "......\
        .XOXXX\
        .X.X..\
        .OOXOO\
        ...X..\
        ......",
        );

        assert_eq!(vertical_board.state(), GameState::Playable);
    }

    #[test]
    fn win_diagonal() {
        let diagonal_board = from_string(
            "......\
        .XOOXX\
        .XX...\
        .OOXOO\
        ...XX.\
        ...X.X",
        );

        assert_eq!(diagonal_board.state(), GameState::Win(PartizanPlayer::Left));
    }

    #[test]
    fn lose_diagonal_tiny() {
        let diagonal_board = from_string(
            "......\
        .OOOXX\
        .XX...\
        .OOXOO\
        ...XX.\
        ...X.X",
        );

        assert_eq!(diagonal_board.state(), GameState::Playable);
    }
}
