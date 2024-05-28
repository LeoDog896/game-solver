//! Reversi is a two-player game played on a rectangular grid of squares.
//! The grid is usually 8x8, but any size can be used.
//!
//! More information: <https://en.wikipedia.org/wiki/Reversi>

pub mod mv;
pub mod cli;

use array2d::Array2D;
use game_solver::game::{Game, ZeroSumPlayer};
use std::hash::Hash;

use self::mv::ReversiMove;

pub const WIDTH: usize = 6;
pub const HEIGHT: usize = 6;

#[derive(Clone, Hash, Eq, PartialEq)]
struct Reversi {
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
        let cell = *self.board.get(m.0.0, m.0.1).unwrap();

        if cell.is_some() {
            return None;
        }

        let opposing_tile = self.player().opponent();

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
            let mut x = m.0.0;
            let mut y = m.0.1;

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

                    if x == m.0.0 && y == m.0.1 {
                        break;
                    }

                    tiles_to_flip.push(ReversiMove((x, y)));
                }
            }
        }

        if tiles_to_flip.is_empty() {
            None
        } else {
            Some(tiles_to_flip)
        }
    }

    fn winning_player(&self) -> Option<ZeroSumPlayer> {
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
            std::cmp::Ordering::Greater => Some(ZeroSumPlayer::One),
            std::cmp::Ordering::Less => Some(ZeroSumPlayer::Two),
            std::cmp::Ordering::Equal => None,
        }
    }
}

impl Game for Reversi {
    type Move = ReversiMove;
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    type Player = ZeroSumPlayer;

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

    fn make_move(&mut self, m: &Self::Move) -> bool {
        let move_set = self.is_valid_move(m).unwrap();

        self.board.set(m.0.0, m.0.1, Some(self.player())).unwrap();

        for idx in move_set {
            self.board.set(idx.0.0, idx.0.1, Some(self.player())).unwrap();
        }

        self.move_count += 1;

        true
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if self.is_valid_move(&ReversiMove((x, y))).is_some() {
                    moves.push(ReversiMove((x, y)));
                }
            }
        }
        moves.into_iter()
    }

    fn is_winning_move(&self, m: &Self::Move) -> Option<Self::Player> {
        let mut board = self.clone();
        board.make_move(m);
        if board.possible_moves().next().is_none() {
            if board.winning_player() == Some(self.player()) {
                Some(self.player())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_draw(&self) -> bool {
        self.winning_player().is_none() && self.possible_moves().next().is_none()
    }
}
