//! Game trait and related types.

/// Represents a player.
pub trait Player {
    /// The max player count.
    #[must_use]
    fn count() -> usize;
    /// The current index of this player starting at 0.
    #[must_use]
    fn idx(&self) -> usize;
    /// The next player to play
    #[must_use]
    fn next(self) -> Self;
}

/// Represents a move outcome
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum GameState<P: Player> {
    /// It is still a player's turn - the game continues.
    Playable,
    /// The game ended in a tie - no players won
    Tie,
    // TODO: handling non-unique player wins.
    /// A player won.
    Win(P),
}

/// Represents a player in a zero-sum (2-player) game.
///
/// Allows for usage of `negamax` instead of minimax.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum ZeroSumPlayer {
    /// The first player.
    One,
    /// The second player.
    Two,
}

impl Player for ZeroSumPlayer {
    fn count() -> usize {
        2
    }

    fn idx(&self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
        }
    }

    fn next(self) -> Self {
        match self {
            ZeroSumPlayer::One => ZeroSumPlayer::Two,
            ZeroSumPlayer::Two => ZeroSumPlayer::One,
        }
    }
}

/// Represents a player in an N-player game.
pub struct NPlayerConst<const N: usize>(usize);

impl<const N: usize> NPlayerConst<N> {
    pub fn new(index: usize) -> NPlayerConst<N> {
        assert!(index < N, "Player index {index} >= max player count {N}");
        Self(index)
    }

    pub fn new_unchecked(index: usize) -> NPlayerConst<N> {
        debug_assert!(index < N, "Player index {index} >= max player count {N}");
        Self(index)
    }
}

impl<const N: usize> Player for NPlayerConst<N> {
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
}

/// Represents a combinatorial game.
pub trait Game: Clone {
    /// The type of move this game uses.
    type Move: Clone;

    /// The iterator type for possible moves.
    type Iter<'a>: Iterator<Item = Self::Move> + 'a
    where
        Self: 'a;

    /// The type of player this game uses.
    /// There are two types of players:
    ///
    /// - [`ZeroSumPlayer`] for two-player zero-sum games.
    /// - [`NPlayer`] for N-player games.
    ///
    /// If your game is a two-player zero-sum game, using [`ZeroSumPlayer`]
    /// allows `negamax` to be used instead of minimax.
    type Player: Player;

    type MoveError;

    /// Returns the player whose turn it is.
    /// The implementation of this should be
    /// similar to either
    ///
    /// ```ignore
    /// use game_solver::game::ZeroSumPlayer;
    ///
    /// fn player(&self) -> Self::Player {
    ///     if game.move_count % 2 == 0 {
    ///        ZeroSumPlayer::One
    ///     } else {
    ///         ZeroSumPlayer::Two
    ///     }
    /// }
    /// ```
    ///
    /// or
    ///
    /// ```ignore
    /// use game_solver::game::NPlayer;
    ///
    /// fn player(&self) -> Self::Player {
    ///     NPlayer(game.move_count % game.num_players)
    /// }
    /// ```
    ///
    /// depending on the type of game.
    ///
    /// However, no implementation is provided
    /// because this does not keep track of the move count.
    fn player(&self) -> Self::Player;

    // TODO: (move_count/max_moves) allow custom evaluation

    /// Returns the amount of moves that have been played
    fn move_count(&self) -> usize;

    /// Get the max number of moves in a game, if any.
    fn max_moves(&self) -> Option<usize>;

    /// Makes a move.
    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError>;

    /// Returns an iterator of all possible moves.
    ///
    /// If possible, this function should "guess" what the best moves are first.
    /// For example, if this is for tic tac toe, it should give the middle move first.
    /// Since "better" moves would be found first, this permits more alpha/beta cutoffs.
    fn possible_moves(&self) -> Self::Iter<'_>;

    // TODO: fn is_immediately_resolvable instead - better optimization for unwinnable games
    /// Returns the next state given a move.
    ///
    /// This has a default implementation and is mainly useful for optimization -
    /// this is used at every tree check and should be fast.
    fn next_state(&self, m: &Self::Move) -> Result<GameState<Self::Player>, Self::MoveError> {
        let mut new_self = self.clone();
        new_self.make_move(m)?;
        Ok(new_self.state())
    }

    /// Returns the current state of the game.
    /// Used for verifying initialization and isn't commonly called.
    fn state(&self) -> GameState<Self::Player>;
}

/// Utility function to get the upper bound of a game.
pub fn upper_bound<T: Game>(game: &T) -> isize {
    game.max_moves().map_or(isize::MAX, |m| m as isize)
}
