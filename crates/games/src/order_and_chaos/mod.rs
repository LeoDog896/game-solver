#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;
use anyhow::{anyhow, Error};
use array2d::Array2D;
use clap::Args;
use game_solver::game::{Game, GameState, ZeroSumPlayer};
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
pub struct OrderAndChaos {
    board: Array2D<Option<CellType>>,
    move_count: usize,
}

const WIDTH: usize = 6;
const HEIGHT: usize = 6;
const WIN_LENGTH: usize = 5;

impl OrderAndChaos {
    /// Create a new game of Nim with the given heaps,
    /// where heaps is a list of the number of objects in each heap.
    pub fn new() -> Self {
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

impl Game for OrderAndChaos {
    /// where Move is a tuple of:
    /// ((row, column), player)
    type Move = OrderAndChaosMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    /// Define Nimbers as a zero-sum game
    type Player = ZeroSumPlayer;
    type MoveError = OrderAndChaosMoveError;

    fn max_moves(&self) -> Option<usize> {
        Some(WIDTH * HEIGHT)
    }

    fn player(&self) -> ZeroSumPlayer {
        if self.move_count % 2 == 0 {
            ZeroSumPlayer::One
        } else {
            ZeroSumPlayer::Two
        }
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        let ((row, column), player) = m.0;
        // check for indexing OOB
        if row >= HEIGHT || column >= WIDTH {
            return Err(OrderAndChaosMoveError::OutOfBounds {
                played: m.0 .0.clone(),
                width: self.board.num_columns(),
                height: self.board.num_rows(),
            });
        }

        // check if the cell is empty
        if self.board[(row, column)] != None {
            return Err(OrderAndChaosMoveError::AlreadyPresent(m.0 .0.clone()));
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
                if self.board[(row, column)] == None {
                    moves.push(OrderAndChaosMove(((row, column), CellType::X)));
                    moves.push(OrderAndChaosMove(((row, column), CellType::O)));
                }
            }
        }

        moves.into_iter()
    }

    // a move is winning if the next player
    // has no possible moves to make (normal play for Nim)
    fn next_state(&self, m: &Self::Move) -> Result<GameState<Self::Player>, Self::MoveError> {
        let mut board = self.clone();
        board.make_move(m)?;
        let found = 'found: {
            let ((row, column), square) = m.0;

            // check for horizontal win
            let mut count = 0;
            let mut mistakes = 0;
            'horiz: for i in 0..WIDTH {
                if board.board[(row, i)] == Some(square) {
                    count += 1;
                    if count == WIN_LENGTH {
                        break 'found true;
                    }
                } else {
                    count = 0;
                    mistakes += 1;
                    if mistakes > WIDTH - WIN_LENGTH {
                        break 'horiz;
                    }
                }
            }

            // check for vertical win
            let mut count = 0;
            let mut mistakes = 0;
            'vert: for i in 0..HEIGHT {
                if board.board[(i, column)] == Some(square) {
                    count += 1;
                    if count == WIN_LENGTH {
                        break 'found true;
                    }
                } else {
                    count = 0;
                    mistakes += 1;
                    if mistakes > HEIGHT - WIN_LENGTH {
                        break 'vert;
                    }
                }
            }

            // check for diagonal win - top left to bottom right
            let mut count = 0;
            let mut mistakes = 0;
            let origins = [(0, 0), (1, 0), (0, 1)];

            'diag: for (row, column) in &origins {
                let mut row = *row;
                let mut column = *column;
                while row < HEIGHT && column < WIDTH {
                    if board.board[(row, column)] == Some(square) {
                        count += 1;
                        if count == WIN_LENGTH {
                            break 'found true;
                        }
                    } else {
                        count = 0;
                        mistakes += 1;
                        if mistakes > HEIGHT - WIN_LENGTH {
                            break 'diag;
                        }
                    }
                    row += 1;
                    column += 1;
                }
            }

            // check for diagonal win - top right to bottom left
            let mut count = 0;
            let mut mistakes = 0;
            let origins = [(0, WIDTH - 1), (1, WIDTH - 1), (0, WIDTH - 2)];

            'diag: for (row, column) in &origins {
                let mut row = *row;
                let mut column = *column;
                while row < HEIGHT {
                    if board.board[(row, column)] == Some(square) {
                        count += 1;
                        if count == WIN_LENGTH {
                            break 'found true;
                        }
                    } else {
                        count = 0;
                        mistakes += 1;
                        if mistakes > HEIGHT - WIN_LENGTH {
                            break 'diag;
                        }
                    }
                    row += 1;
                    if column == 0 {
                        break;
                    }
                    column -= 1;
                }
            }

            false
        };

        Ok(if found {
            GameState::Win(ZeroSumPlayer::One)
        } else if board.possible_moves().next().is_none() {
            GameState::Win(ZeroSumPlayer::Two)
        } else {
            GameState::Playable
        })
    }

    fn state(&self) -> GameState<Self::Player> {
        unimplemented!()
    }
}

impl Display for OrderAndChaos {
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
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct OrderAndChaosArgs {
    moves: Vec<String>,
}

impl Default for OrderAndChaosArgs {
    fn default() -> Self {
        Self { moves: vec![] }
    }
}

impl TryFrom<OrderAndChaosArgs> for OrderAndChaos {
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

            let player = match args[2] {
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
