/// Represents a player.
pub trait Player: Sized + Eq {
    /// The max player count.
    #[must_use]
    fn count() -> usize;
    /// The current index of this player starting at 0.
    #[must_use]
    fn idx(&self) -> usize;
    /// The next player to play
    #[must_use]
    fn next(self) -> Self;
    /// The previous player to play
    #[must_use]
    fn previous(self) -> Self;
    /// How the player instance 'changes' on the next move.
    ///
    /// For partizan games, the player doesn't change:
    /// Left stays left; right stays right.
    ///
    /// For impartial games, the player does change:
    /// Next turns into previous, and previous turns into next
    fn turn(self) -> Self;
}

/// Represents a two player player.
pub trait TwoPlayer: Player {
    /// Gets the other player
    #[must_use]
    fn other(self) -> Self {
        self.next()
    }
}

/// Represents a player in a zero-sum (2-player) game,
/// where the game is partizan. That is,
/// a player can affect the `Game::possible_moves` function,
/// or players have different winning outcomes.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum PartizanPlayer {
    /// The first player.
    Left,
    /// The second player.
    Right,
}

impl Player for PartizanPlayer {
    fn count() -> usize {
        2
    }

    fn idx(&self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    fn previous(self) -> Self {
        self.next()
    }

    fn turn(self) -> Self {
        self
    }
}

impl TwoPlayer for PartizanPlayer {}

/// Represents a player in a zero-sum (2-player) game,
/// where the game is impartial. That is,
/// the only difference between players is who goes first.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum ImpartialPlayer {
    /// The player that will play on the current game state,
    Next,
    /// The player that has played previous to this game state
    /// (or will play after Next).
    Previous,
}

impl Player for ImpartialPlayer {
    fn count() -> usize {
        2
    }

    fn idx(&self) -> usize {
        match self {
            Self::Next => 0,
            Self::Previous => 1,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Next => Self::Previous,
            Self::Previous => Self::Next,
        }
    }

    fn previous(self) -> Self {
        self.next()
    }

    fn turn(self) -> Self {
        self.next()
    }
}

impl TwoPlayer for ImpartialPlayer {}

/// Represents a player in an N-player game.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct NPlayerPartizanConst<const N: usize>(usize);

impl<const N: usize> NPlayerPartizanConst<N> {
    pub fn new(index: usize) -> NPlayerPartizanConst<N> {
        assert!(index < N, "Player index {index} >= max player count {N}");
        Self(index)
    }

    pub fn new_unchecked(index: usize) -> NPlayerPartizanConst<N> {
        debug_assert!(index < N, "Player index {index} >= max player count {N}");
        Self(index)
    }
}

impl<const N: usize> Player for NPlayerPartizanConst<N> {
    fn count() -> usize {
        N
    }

    fn idx(&self) -> usize {
        self.0
    }

    fn next(self) -> Self {
        // This will always make index < N.
        Self::new_unchecked((self.0 + 1) % N)
    }

    fn previous(self) -> Self {
        if self.0 == 0 {
            Self::new_unchecked(N - 1)
        } else {
            Self::new_unchecked(self.0 - 1)
        }
    }

    fn turn(self) -> Self {
        self
    }
}
