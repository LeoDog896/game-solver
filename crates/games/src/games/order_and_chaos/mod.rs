//! Order and chaos is a tic tac toe variant
//! that plays on a 6x6 square board.
//!
//! The game is played by two players, order and chaos.
//! Order plays first, and places Xs and Os on the board.
//! Chaos also plays Xs and Os, but Chaos's goal is to
//! make Order tie the game.
//!
//! 5 in a row wins the game for Order - otherwise, Chaos wins.
//! This serves as an exemplary example
//! for the simplicity in implementation,
//! showing how trivial it is to implement a new game.
//!
//! Learn more: <https://en.wikipedia.org/wiki/Order_and_Chaos>

pub mod cli;

use array2d::Array2D;
use game_solver::game::{Game, ZeroSumPlayer};
use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum CellType {
    X,
    O,
    Empty,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct OrderAndChaos {
    board: Array2D<CellType>,
    move_count: usize,
}

const WIDTH: usize = 6;
const HEIGHT: usize = 6;
const WIN_LENGTH: usize = 5;

impl OrderAndChaos {
    /// Create a new game of Nim with the given heaps,
    /// where heaps is a list of the number of objects in each heap.
    fn new() -> Self {
        Self {
            board: Array2D::filled_with(CellType::Empty, HEIGHT, WIDTH),
            move_count: 0,
        }
    }
}

impl Game for OrderAndChaos {
    /// where Move is a tuple of:
    /// ((row, column), player)
    type Move = ((usize, usize), CellType);
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    /// Define Nimbers as a zero-sum game
    type Player = ZeroSumPlayer;

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

    fn make_move(&mut self, m: &Self::Move) -> bool {
        let ((row, column), player) = *m;
        // check for indexing OOB
        if row >= HEIGHT || column >= WIDTH {
            return false;
        }

        // check if the cell is empty
        if self.board[(row, column)] != CellType::Empty {
            return false;
        }

        // make the move
        self.board[(row, column)] = player;
        self.move_count += 1;

        true
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();

        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                if self.board[(row, column)] == CellType::Empty {
                    moves.push(((row, column), CellType::X));
                    moves.push(((row, column), CellType::O));
                }
            }
        }

        moves.into_iter()
    }

    // a move is winning if the next player
    // has no possible moves to make (normal play for Nim)
    fn is_winning_move(&self, m: &Self::Move) -> Option<Self::Player> {
        let mut board = self.clone();
        board.make_move(m);
        let found = 'found: {
            let ((row, column), square) = *m;

            // check for horizontal win
            let mut count = 0;
            let mut mistakes = 0;
            'horiz: for i in 0..WIDTH {
                if board.board[(row, i)] == square {
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
                if board.board[(i, column)] == square {
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
                    if board.board[(row, column)] == square {
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
                    if board.board[(row, column)] == square {
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

        if self.player() == ZeroSumPlayer::One {
            // order
            if found {
                Some(ZeroSumPlayer::One)
            } else {
                None
            }
        } else if found {
            Some(ZeroSumPlayer::One)
        } else if board.possible_moves().next().is_none() {
            Some(ZeroSumPlayer::Two)
        } else {
            None
        }
    }

    // Nim can never be a draw -
    // if there are no possible moves, the game is already won
    fn is_draw(&self) -> bool {
        false
    }
}

impl Display for OrderAndChaos {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                match self.board[(row, column)] {
                    CellType::X => write!(f, "X")?,
                    CellType::O => write!(f, "O")?,
                    CellType::Empty => write!(f, "-")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
