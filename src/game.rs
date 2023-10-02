//! Game trait and related types.

/// Represents a player.
pub trait Player {
    /// Whether this Player implementation is for a two-player game.
    fn is_two_player() -> bool;
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

impl ZeroSumPlayer {
    /// Get the player opposite to this one.
    #[must_use]
    pub const fn opponent(&self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
        }
    }
}

impl Player for ZeroSumPlayer {
    fn is_two_player() -> bool {
        true
    }
}

/// Represents a player in an N-player game.
pub struct NPlayer(pub usize);

impl Player for NPlayer {
    fn is_two_player() -> bool {
        false
    }
}

/// Represents a combinatorial game.
pub trait Game {
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

    /// Returns the player whose turn it is.
    /// The implementation of this should be
    /// similar to either
    ///
    /// ```
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
    /// ```
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

    /// Returns the amount of moves that have been played
    fn move_count(&self) -> usize;

    /// Get the max number of moves in a game, if any.
    fn max_moves(&self) -> Option<usize>;

    /// Returns true if the move was valid, and makes the move if it was.
    fn make_move(&mut self, m: &Self::Move) -> bool;

    /// Returns a vector of all possible moves.
    ///
    /// If possible, this function should "guess" what the best moves are first.
    /// For example, if this is for tic tac toe, it should give the middle move first.
    /// This allows alpha-beta pruning to move faster.
    fn possible_moves(&self) -> Self::Iter<'_>;

    /// Returns true if the move is a winning move.
    fn is_winning_move(&self, m: &Self::Move) -> bool;

    /// Returns true if the game is a draw.
    /// This function must exist for the current game,
    /// e.g. with tic tac toe, it must check if the board is full.
    fn is_draw(&self) -> bool;
}

/// Utility function to get the upper bound of a game.
pub fn upper_bound<T: Game>(game: &T) -> isize {
    return game.max_moves().map(|m| m as isize).unwrap_or(isize::MAX);
}
