#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;

use array2d::Array2D;
use game_solver::{
    game::{Game, GameState},
    player::PartizanPlayer,
};
use thiserror::Error;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum InnerCellType {
    Wave,
    Cross,
    Circle,
    Square,
    Star,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct CellType(InnerCellType, PartizanPlayer);

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Zener {
    /// [0] is the piece on the bottom, [len - 1] is the piece on top.
    board: Array2D<Vec<CellType>>,
    player: PartizanPlayer,
    compulsory: Option<InnerCellType>,
    move_count: usize,
    gutter: Option<CellType>,
}

const NUM_ROWS: usize = 7;
const NUM_COLS: usize = 5;

impl Default for Zener {
    fn default() -> Self {
        let mut board = Array2D::filled_with(vec![], NUM_ROWS, NUM_COLS);
        // TODO: make this generic
        board[(0, 0)] = vec![CellType(InnerCellType::Star, PartizanPlayer::Right)];
        board[(0, 1)] = vec![CellType(InnerCellType::Square, PartizanPlayer::Right)];
        board[(0, 2)] = vec![CellType(InnerCellType::Wave, PartizanPlayer::Right)];
        board[(0, 3)] = vec![CellType(InnerCellType::Cross, PartizanPlayer::Right)];
        board[(0, 4)] = vec![CellType(InnerCellType::Circle, PartizanPlayer::Right)];

        board[(NUM_ROWS - 1, 4)] = vec![CellType(InnerCellType::Star, PartizanPlayer::Right)];
        board[(NUM_ROWS - 1, 3)] = vec![CellType(InnerCellType::Square, PartizanPlayer::Right)];
        board[(NUM_ROWS - 1, 2)] = vec![CellType(InnerCellType::Wave, PartizanPlayer::Right)];
        board[(NUM_ROWS - 1, 1)] = vec![CellType(InnerCellType::Cross, PartizanPlayer::Right)];
        board[(NUM_ROWS - 1, 0)] = vec![CellType(InnerCellType::Circle, PartizanPlayer::Right)];

        Self {
            board,
            player: PartizanPlayer::Left,
            compulsory: None,
            move_count: 0,

            gutter: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ZenerPosition {
    Position(usize, usize),
    Gutter,
}

#[derive(Error, Clone, Debug)]
pub enum ZenerMoveError {
    #[error("can not move from {0:?} since there's no piece!")]
    NoPiece((usize, usize)),
    #[error("can not move a piece 'from' a non-existent position {0:?}")]
    FromOutOfBounds((usize, usize)),
    #[error("can not move a piece 'to' a non-existent position {0:?}")]
    ToOutOfBounds((usize, usize)),
    #[error("the gutter is filled - the game is already won!")]
    GutterFilled,
    #[error("can't move {want:?}: need to move {need:?}")]
    Compulsory { want: CellType, need: InnerCellType },
}

#[derive(Clone, Copy)]
pub struct ZenerMove {
    from: (usize, usize),
    to: ZenerPosition,
}

impl Game for Zener {
    type Move = ZenerMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    /// Left is bottom, Right is top
    type Player = PartizanPlayer;
    type MoveError = ZenerMoveError;

    fn max_moves(&self) -> Option<usize> {
        None
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        if self.gutter.is_some() {
            return Err(ZenerMoveError::GutterFilled);
        }

        // check that to is in bounds
        if let ZenerPosition::Position(row, col) = m.to {
            if self.board.get(row, col).is_none() {
                return Err(ZenerMoveError::ToOutOfBounds((row, col)));
            }
        }

        // get the piece to move
        let from_piece = self
            .board
            .get_mut(m.from.0, m.from.1)
            .ok_or(ZenerMoveError::FromOutOfBounds(m.from))?
            .pop()
            .ok_or(ZenerMoveError::NoPiece(m.from))?
            .clone();

        if let Some(compulsory) = self.compulsory {
            if from_piece.0 != compulsory {
                return Err(ZenerMoveError::Compulsory {
                    want: from_piece,
                    need: compulsory,
                });
            }
        }

        // add it on the 'to' stack.
        match m.to {
            ZenerPosition::Position(row, col) => self
                .board
                .get_mut(row, col)
                .expect("Guard check failed - this shouldn't happen.") // guaranteed with the guard check
                .push(from_piece),
            ZenerPosition::Gutter => unimplemented!(),
        }

        Ok(())
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        if self.gutter.is_some() {
            return vec![].into_iter();
        }

        let mut moves = Vec::new();

        for (row, col) in self.board.indices_row_major() {
            if self.board.get(row, col).map(|cell| cell.last()).flatten().is_none() {
                continue;
            };

            let offsets: Vec<(isize, isize)> = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];
            for offset in offsets {
                let new_row = (row as isize) + offset.0;
                let new_col = (col as isize) + offset.1;

                if new_row == -1 || new_row == (NUM_ROWS as isize) {
                    moves.push(ZenerMove {
                        from: (row, col),
                        to: ZenerPosition::Gutter,
                    });
                    continue;
                }

                let Ok(new_col) = new_col.try_into() else {
                    continue;
                };

                if self
                    .board
                    .get(new_row.try_into().unwrap(), new_col)
                    .is_some()
                {
                    moves.push(ZenerMove {
                        from: (row, col),
                        to: ZenerPosition::Position(new_row.try_into().unwrap(), new_col),
                    });
                }
            }
        }

        return moves.into_iter();
    }

    fn player(&self) -> Self::Player {
        self.player
    }

    fn state(&self) -> GameState<Self::Player> {
        match self.gutter {
            None => GameState::Playable,
            Some(CellType(_, player)) => GameState::Win(player),
        }
    }
}
