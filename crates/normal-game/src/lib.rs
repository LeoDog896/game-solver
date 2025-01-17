use std::cmp::Ordering;

// counting: could i count specifically non-reversible and non-dominable positions?
use num_bigint::BigInt;

/// The outcome of a game
pub enum Outcome {
    Left,
    Right,
    Next,
    Previous
}

/// General game utilities that should be
/// implemented on every game implementation
pub trait Game {
    // TODO: transfinite games
    /// Returns the birthday of a game, starting at zero:
    /// `VecGame::zero().birthday() == 0`
    fn birthday(&self) -> BigInt;

    /// This operation is communative and associative.
    fn disjinctive_sum(&self, g: Box<dyn Game>) -> Box<dyn Game>;

    /// Negates a game, returning its negative variant.
    fn negate(&self) -> Box<dyn Game>;

    /// Gets the outcome of a game
    fn outcome(&self) -> Outcome;

    /// Gets the ordering of a game to another game
    /// Games define a partial ordering, and not a total ordering.
    fn partial_cmp(&self, other: &Box<dyn Game>) -> Option<Ordering>;

    /// Checks if two games are equal.
    fn eq(&self, other: &Box<dyn Game>) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialEq for Box<dyn Game> {
    fn eq(&self, other: &Self) -> bool {
        Game::eq(self.as_ref(), &Box::new(other))
    }
}

impl PartialOrd for Box<dyn Game> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Game::partial_cmp(self.as_ref(), &Box::new(other))
    }
}

/// We define a game by its left and right games,
/// representing G = {left|right} and all of its options.
/// 
/// This is not guaranteed to be the canonical game variant.
/// 
/// This is stored as a vector of both the left and right game options: while this
/// works for small games, this fails for big games. Other games that implement `Game` should be preferred.
pub struct VecGame {
    left: Vec<Box<dyn Game>>,
    right: Vec<Box<dyn Game>>
}

impl VecGame {
    /// Returns the only game born on day 0: the game {|}
    pub const fn zero() -> Self {
        VecGame::new(vec![], vec![])
    }

    /// Returns a game such that left and right only have one option
    pub fn singleton(left: Self, right: Self) -> Self {
        VecGame::new(vec![Box::new(left)], vec![Box::new(right)])
    }

    /// Returns a game where the left and right options are flipped
    pub fn flip(self) -> Self {
        VecGame::new(
            self.right,
            self.left
        )
    }

    /// Returns the infinitesimal star game: {0|0} = *
    pub fn star() -> Self {
        VecGame::singleton(VecGame::zero(), VecGame::zero())
    }

    /// Returns the up game: {0 | *} = ↑
    pub fn up() -> Self {
        VecGame::singleton(VecGame::zero(), VecGame::star())
    }

    /// Returns the down game: {* | 0} = ↓
    pub fn down() -> Self {
        Self::up().flip()
    }

    pub const fn new(left: Vec<Box<dyn Game>>, right: Vec<Box<dyn Game>>) -> Self {
        Self {
            left,
            right
        }
    }
}

impl Game for VecGame {
    fn birthday(&self) -> BigInt {
        if self.left.is_empty() && self.right.is_empty() {
            return BigInt::ZERO
        }

        unimplemented!()
    }

    fn disjinctive_sum(&self, g: Box<dyn Game>) -> Box<dyn Game> {
        unimplemented!()
    }

    fn outcome(&self) -> Outcome {
        unimplemented!()
    }

    fn negate(&self) -> Box<dyn Game> {
        Box::new(VecGame::new(
            self.left.iter().map(|g| g.negate()).collect(),
            self.right.iter().map(|g| g.negate()).collect()
        ))
    }

    fn partial_cmp(&self, other: &Box<dyn Game>) -> Option<Ordering> {
        unimplemented!()
    }
}
