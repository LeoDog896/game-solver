#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;

use array2d::Array2D;
use game_solver::{game::{Game, GameState}, player::PartizanPlayer};
use thiserror::Error;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum InnerCellType {
    Wave,
    Cross,
    Circle,
    Square,
    Star
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

            gutter: None
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum ZenerMoveError {
    #[error("can not move at {0:?} since there's no piece!")]
    NoPiece(ZenerPosition)
}

/// (row, col)
pub type ZenerPosition = (usize, usize);

#[derive(Clone, Copy)]
pub struct ZenerMove {
    from: ZenerPosition,
    to: ZenerPosition
}

impl Game for Zener {
    type Move = ZenerMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    /// Left is bottom, Right is top
    type Player = PartizanPlayer;
    type MoveError = ZenerMoveError;

    fn max_moves(&self) -> Option<usize> {
        // TODO
        None
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        // get the piece to move
        let from_piece = self.board[m.from]
            .last()
            .ok_or(ZenerMoveError::NoPiece(m.from))?.clone();
        
        // add it on the 'to' stack.
        self.board[m.to].push(from_piece);
        
        Ok(())
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        unimplemented!()
    }

    fn player(&self) -> Self::Player {
        self.player
    }

    fn state(&self) -> GameState<Self::Player> {
        match self.gutter {
            None => GameState::Playable,
            Some(CellType(_, player)) => GameState::Win(player)
        }
    }
}
