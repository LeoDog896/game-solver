use itertools::{Interleave, Itertools};
use thiserror::Error;

use std::{fmt::Debug, iter::Map};
use crate::{game::{Game, Normal, NormalImpartial}, player::ImpartialPlayer};

/// Represents the disjoint sum of
/// two impartial normal combinatorial games.
/// 
/// Since `Game` isn't object safe, we use `dyn Any` internally with downcast safety.
#[derive(Clone)]
pub struct DisjointImpartialNormalGame<L: Game, R: Game> {
    left: L,
    right: R
}

#[derive(Clone)]
pub enum DisjointMove<L: Game, R: Game> {
    LeftMove(L::Move),
    RightMove(R::Move)
}

#[derive(Debug, Error, Clone)]
pub enum DisjointMoveError<L: Game, R: Game> {
    #[error("Could not make the move on left: {0}")]
    LeftError(L::MoveError),
    #[error("Could not make the move on right: {0}")]
    RightError(R::MoveError)
}

type LeftMoveMap<L, R> = Box<dyn Fn(<L as Game>::Move) -> DisjointMove<L, R>>;
type RightMoveMap<L, R> = Box<dyn Fn(<R as Game>::Move) -> DisjointMove<L, R>>;

impl<L: Game + Debug + 'static, R: Game + Debug + 'static> Normal for DisjointImpartialNormalGame<L, R> {}
impl<L: Game + Debug + 'static, R: Game + Debug + 'static> NormalImpartial for DisjointImpartialNormalGame<L, R> {}
impl<L: Game + Debug + 'static, R: Game + Debug + 'static> Game for DisjointImpartialNormalGame<L, R> {
    type Move = DisjointMove<L, R>;
    type Iter<'a> = Interleave<
        Map<<L as Game>::Iter<'a>, LeftMoveMap<L, R>>,
        Map<<R as Game>::Iter<'a>, RightMoveMap<L, R>>
    > where L: 'a, R: 'a, L::Move: 'a, R::Move: 'a;

    type Player = ImpartialPlayer;
    type MoveError = DisjointMoveError<L, R>;

    fn move_count(&self) -> usize {
        self.left.move_count() + self.right.move_count()
    }

    fn max_moves(&self) -> Option<usize> {
        self.left.max_moves()
            .map(
                |l| self.right.max_moves()
                    .map(|r| l + r)
            ).flatten()
    }

    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError> {
        match m {
            DisjointMove::LeftMove(l) => 
                self.left.make_move(l).map_err(|err| DisjointMoveError::LeftError(err)),
            DisjointMove::RightMove(r) => 
                self.right.make_move(r).map_err(|err| DisjointMoveError::RightError(err))
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        fn as_left<L: Game, R: Game>(m: L::Move) -> DisjointMove<L, R> {
            DisjointMove::LeftMove(m)
        }
    
        fn as_right<L: Game, R: Game>(m: R::Move) -> DisjointMove<L, R> {
            DisjointMove::RightMove(m)
        }

        self.left.possible_moves()
            .map(Box::new(as_left) as LeftMoveMap<L, R>)
            .interleave(
                self.right.possible_moves()
                .map(Box::new(as_right) as RightMoveMap<L, R>)
            )
    }

    fn state(&self) -> crate::game::GameState<Self::Player> {
        <Self as Normal>::state(&self)
    }

    fn player(&self) -> Self::Player {
        ImpartialPlayer::Next
    }
}
