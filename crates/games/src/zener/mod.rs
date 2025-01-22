#![doc = include_str!("./README.md")]

#[cfg(feature = "egui")]
pub mod gui;

use std::{fmt::Display, str::FromStr};

use arrayvec::ArrayVec;
use clap::Args;
use game_solver::{
    game::{Game, GameState, Normal},
    loopy::{Loopy, LoopyTracker},
    player::PartizanPlayer,
};
use grid_stack::Grid;
use owo_colors::{OwoColorize, Stream::Stdout, Style};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum InnerCellType {
    Wave,
    Cross,
    Circle,
    Square,
    Star,
}

impl Display for InnerCellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Wave => "~",
                Self::Cross => "+",
                Self::Circle => "∘",
                Self::Square => "□",
                Self::Star => "⋆",
            }
        )
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct CellType(InnerCellType, PartizanPlayer);

impl Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!("{}", self.0).if_supports_color(Stdout, |text| text.style(match self.1 {
                PartizanPlayer::Left => Style::new().on_bright_white(),
                PartizanPlayer::Right => Style::new().on_black(),
            }))
        )
    }
}

const WIDTH: usize = 5;
const HEIGHT: usize = 7;
const TOTAL_SIZE: usize = WIDTH * HEIGHT;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ZenerNoLoopTrack {
    /// vec is used as a fifo
    board: Grid<ArrayVec<CellType, 10>, WIDTH, HEIGHT, TOTAL_SIZE>,
    compulsory: Option<InnerCellType>,
    move_count: usize,
    gutter: Option<CellType>,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Zener {
    /// vec is used as a fifo
    board: Grid<ArrayVec<CellType, 10>, WIDTH, HEIGHT, TOTAL_SIZE>,
    compulsory: Option<InnerCellType>,
    move_count: usize,
    gutter: Option<CellType>,
    loopy: LoopyTracker<ZenerNoLoopTrack, Self>,
}

impl Loopy<ZenerNoLoopTrack> for Zener {
    fn tracker(&self) -> &LoopyTracker<ZenerNoLoopTrack, Self> {
        &self.loopy
    }

    fn tracker_mut(&mut self) -> &mut LoopyTracker<ZenerNoLoopTrack, Self> {
        &mut self.loopy
    }

    fn without_tracker(&self) -> ZenerNoLoopTrack {
        ZenerNoLoopTrack {
            board: self.board.clone(),
            compulsory: self.compulsory,
            move_count: self.move_count,
            gutter: self.gutter,
        }
    }
}

impl Default for Zener {
    fn default() -> Self {
        let mut board = Grid::filled_with(ArrayVec::<CellType, 10>::new());
        // TODO: make this generic
        board[(0, 0)].push(CellType(InnerCellType::Star, PartizanPlayer::Right));
        board[(1, 0)].push(CellType(InnerCellType::Square, PartizanPlayer::Right));
        board[(2, 0)].push(CellType(InnerCellType::Wave, PartizanPlayer::Right));
        board[(3, 0)].push(CellType(InnerCellType::Cross, PartizanPlayer::Right));
        board[(4, 0)].push(CellType(InnerCellType::Circle, PartizanPlayer::Right));

        board[(4, HEIGHT - 1)].push(CellType(InnerCellType::Star, PartizanPlayer::Left));
        board[(3, HEIGHT - 1)].push(CellType(InnerCellType::Square, PartizanPlayer::Left));
        board[(2, HEIGHT - 1)].push(CellType(InnerCellType::Wave, PartizanPlayer::Left));
        board[(1, HEIGHT - 1)].push(CellType(InnerCellType::Cross, PartizanPlayer::Left));
        board[(0, HEIGHT - 1)].push(CellType(InnerCellType::Circle, PartizanPlayer::Left));

        let x = Self {
            board,
            compulsory: None,
            move_count: 0,

            gutter: None,

            loopy: LoopyTracker::new(),
        };

        println!("{}", x);

        x
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ZenerPosition {
    Position(usize, usize),
    Gutter,
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    fn as_step(&self) -> (isize, isize) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    fn directions() -> Vec<Direction> {
        vec![Self::Up, Self::Down, Self::Left, Self::Right]
    }

    fn apply_to_position(&self, position: (usize, usize)) -> Result<ZenerPosition, anyhow::Error> {
        let offset = self.as_step();

        let new_x = (position.0 as isize) + offset.0;
        let new_y = (position.1 as isize) + offset.1;

        if new_y == -1 || new_y == (HEIGHT as isize) {
            return Ok(ZenerPosition::Gutter);
        }

        if (HEIGHT as isize) < new_y {
            return Err(anyhow::anyhow!("out of height bounds ({HEIGHT} < {new_y})"));
        }

        let Ok(new_x) = new_x.try_into() else {
            return Err(anyhow::anyhow!("out of width bounds ({new_x} < 0)"));
        };

        if WIDTH <= new_x {
            return Err(anyhow::anyhow!("out of width bounds ({WIDTH} <= {new_x})"));
        }

        return Ok(ZenerPosition::Position(new_x.try_into().unwrap(), new_x));
    }
}

impl FromStr for Direction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            "right" => Ok(Direction::Right),
            _ => Err(anyhow::anyhow!(
                "Direction {s} does not exist. Valid options are up/down/left/right"
            )),
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum ZenerMoveError {
    #[error("can not move from {0:?} since there's no piece!")]
    NoPiece((usize, usize)),
    #[error("can not move a piece 'from' a non-existent position {0:?}")]
    FromOutOfBounds((usize, usize)),
    #[error("can not move a piece 'from' a non-existent position {0:?} in direction {1:?}")]
    ToOutOfBounds((usize, usize), Direction),
    #[error("the gutter is filled - the game is already won!")]
    GutterFilled,
    #[error("can't move {want:?}: need to move {need:?}")]
    Compulsory { want: CellType, need: InnerCellType },
    #[error("can't move a player into their own gutter!")]
    WrongMoveGutter,
    #[error("tried to play as {0:?}, but is {1:?}")]
    WrongPlayer(PartizanPlayer, PartizanPlayer),
}

#[derive(Clone, Copy, Debug)]
pub struct ZenerMove {
    from: (usize, usize),
    to: Direction,
}

impl Display for ZenerMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.from, self.to)?;

        Ok(())
    }
}

impl Normal for Zener {}
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
        let Ok(position) = m.to.apply_to_position(m.from) else {
            return Err(ZenerMoveError::ToOutOfBounds(m.from, m.to));
        };

        // we borrow self mutably after this
        let prev = self.clone();

        let player = self.player();

        // get the piece to move
        let from_piece_arr = self
            .board
            .get_mut(m.from.0, m.from.1)
            .ok_or(ZenerMoveError::FromOutOfBounds(m.from))?;

        let from_piece = from_piece_arr
            .last()
            .ok_or(ZenerMoveError::NoPiece(m.from))?;

        // check that this is the right player
        if from_piece.1 != player {
            return Err(ZenerMoveError::WrongPlayer(from_piece.1, self.player()));
        }

        if let ZenerPosition::Gutter = position {
            if m.from.0 == 0 && player == PartizanPlayer::Right
                || m.from.0 != 0 && player == PartizanPlayer::Left
            {
                return Err(ZenerMoveError::WrongMoveGutter);
            }
        }

        let from_piece = from_piece_arr
            .pop()
            .ok_or(ZenerMoveError::NoPiece(m.from))?;

        if let Some(compulsory) = self.compulsory {
            if from_piece.0 != compulsory {
                return Err(ZenerMoveError::Compulsory {
                    want: from_piece,
                    need: compulsory,
                });
            }
        }

        // add it on the 'to' stack.
        match position {
            ZenerPosition::Position(x, y) => self
                .board
                .get_mut(x, y)
                .expect("Guard check failed - this shouldn't happen.") // guaranteed with the guard check
                .push(from_piece),
            ZenerPosition::Gutter => self.gutter = Some(from_piece),
        }

        self.move_count += 1;
        self.loopy.mark_visited(prev);

        Ok(())
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        if self.gutter.is_some() {
            return vec![].into_iter();
        }

        let mut moves = Vec::new();

        for (x, y) in self.board.indices_row_major() {
            let Some(cell) = self
                .board
                .get(x, y)
                .map(|cell| cell.len().checked_sub(1).map(|i| cell.get(i)).flatten())
                .flatten()
            else {
                continue;
            };

            if cell.1 != self.player() {
                continue;
            }

            for direction in Direction::directions() {
                if let Ok(new_position) = direction.apply_to_position((x, y)) {
                    if let ZenerPosition::Gutter = new_position {
                        if y == 0 && self.player() == PartizanPlayer::Right
                            || y != 0 && self.player() == PartizanPlayer::Left
                        {
                            continue;
                        }
                    }

                    moves.push(ZenerMove {
                        from: (x, y),
                        to: direction,
                    });
                }
            }
        }

        return moves.into_iter();
    }

    fn player(&self) -> Self::Player {
        if self.move_count % 2 == 0 {
            PartizanPlayer::Left
        } else {
            PartizanPlayer::Right
        }
    }

    fn state(&self) -> GameState<Self::Player> {
        if self.loopy.has_visited(self) {
            return GameState::Tie;
        }

        <Self as Normal>::state(&self)
    }
}

/// Analyzes Zener.
///
#[doc = include_str!("./README.md")]
#[derive(Args, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ZenerArgs {}

impl Default for ZenerArgs {
    fn default() -> Self {
        Self {}
    }
}

impl FromStr for ZenerMove {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(":").collect::<Vec<_>>().into_iter();

        if split.len() != 3 {
            return Err(anyhow::anyhow!("move must be separated as `x:y:direction`"));
        }

        let Some(x) = split.next() else {
            return Err(anyhow::anyhow!("No `x` present. (Format: `x:y:direction`)"));
        };
        let x: usize = x
            .parse()
            .map_err(|_| anyhow::anyhow!("x {x} is not a number"))?;

        let Some(y) = split.next() else {
            return Err(anyhow::anyhow!("No y present. (Format: `x:y:direction`)"));
        };
        let y = y
            .parse()
            .map_err(|_| anyhow::anyhow!("y {y} is not a number"))?;

        let Some(direction) = split.next() else {
            return Err(anyhow::anyhow!(
                "No direction present. (Format: `x:y:direction`)"
            ));
        };

        Ok(Self {
            from: (x, y),
            to: Direction::from_str(direction)?,
        })
    }
}

impl TryFrom<ZenerArgs> for Zener {
    type Error = anyhow::Error;

    fn try_from(value: ZenerArgs) -> Result<Self, Self::Error> {
        Ok(Zener::default())
    }
}

impl Display for Zener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row_iter in self.board.rows_iter() {
            for element in row_iter {
                if let Some(top) = element.last() {
                    let int_display = if element.len() != 1 {
                        format!("{}", element.len())
                    } else {
                        " ".to_string()
                    };

                    match top.1 {
                        PartizanPlayer::Left => write!(f, "[ {top}{}]", int_display)?,
                        PartizanPlayer::Right => write!(f, "( {top}{})", int_display)?,
                    }
                } else {
                    write!(f, "{{   }}")?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
