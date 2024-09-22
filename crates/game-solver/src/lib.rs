//! `game_solver` is a library for solving games.
//!
//! If you want to read how to properly use this library,
//! [the book](https://leodog896.github.io/game-solver/book) is
//! a great place to start.

pub mod game;
pub mod player;
// TODO: reinforcement
// #[cfg(feature = "reinforcement")]
// pub mod reinforcement;
pub mod transposition;

#[cfg(feature = "rayon")]
use std::hash::BuildHasher;

use game::{upper_bound, GameState};
use player::TwoPlayer;

use crate::game::Game;
use crate::transposition::{Score, TranspositionTable};
use std::hash::Hash;

/// Runs the two-player minimax variant on a zero-sum game.
/// Since it uses alpha-beta pruning, you can specify an alpha beta window.
fn negamax<T: Game<Player = impl TwoPlayer> + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
    mut alpha: isize,
    mut beta: isize
) -> Result<isize, T::MoveError> {
    match game.state() {
        GameState::Playable => (),
        GameState::Tie => return Ok(0),
        GameState::Win(_) => return Ok(0),
    };

    // check if this is a winning configuration
    if let Ok(Some(board)) = game.find_immediately_resolvable_game() {
        return Ok(upper_bound(&board) - board.move_count() as isize - 1);
    }

    // fetch values from the transposition table
    {
        let score = transposition_table
            .get(game)
            .unwrap_or_else(|| Score::UpperBound(upper_bound(game)));

        match score {
            Score::UpperBound(max) => {
                if beta > max {
                    beta = max;
                    if alpha >= beta {
                        return Ok(beta);
                    }
                }
            }
            Score::LowerBound(min) => {
                if alpha < min {
                    alpha = min;
                    if alpha >= beta {
                        return Ok(alpha);
                    }
                }
            }
        };
    }

    // for [principal variation search](https://www.chessprogramming.org/Principal_Variation_Search)
    let mut first_child = true;

    for m in &mut game.possible_moves() {
        let mut board = game.clone();
        board.make_move(&m)?;

        let score = if first_child {
            -negamax(&board, transposition_table, -beta, -alpha)?
        } else {
            let score = -negamax(&board, transposition_table, -alpha - 1, -alpha)?;
            if score > alpha {
                -negamax(&board, transposition_table, -beta, -alpha)?
            } else {
                score
            }
        };

        // alpha-beta pruning - we can return early
        if score >= beta {
            transposition_table.insert(game.clone(), Score::LowerBound(score));
            return Ok(beta);
        }

        if score > alpha {
            alpha = score;
        }

        first_child = false;
    }

    transposition_table.insert(game.clone(), Score::UpperBound(alpha));

    Ok(alpha)
}

/// Solves a game, returning the evaluated score.
///
/// The score of a position is defined by the best possible end result for the player whose turn it is.
/// In 2 player games, if a score > 0, then the player whose turn it is has a winning strategy.
/// If a score < 0, then the player whose turn it is has a losing strategy.
/// Else, the game is a draw (score = 0).
pub fn solve<T: Game<Player = impl TwoPlayer> + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
) -> Result<isize, T::MoveError> {
    let mut alpha = -upper_bound(game);
    let mut beta = upper_bound(game) + 1;

    // we're trying to guess the score of the board via null windows
    while alpha < beta {
        let med = alpha + (beta - alpha) / 2;

        // do a [null window search](https://www.chessprogramming.org/Null_Window)
        let evaluation = negamax(game, transposition_table, med, med + 1)?;

        if evaluation <= med {
            beta = evaluation;
        } else {
            alpha = evaluation;
        }
    }

    Ok(alpha)
}

/// Utility function to get a list of the move scores of a certain game.
/// Since its evaluating the same game, you can use the same transposition table.
///
/// If you want to evaluate the score of a board as a whole, use the `solve` function.
///
/// # Returns
///
/// An iterator of tuples of the form `(move, score)`.
pub fn move_scores<'a, T: Game<Player = impl TwoPlayer> + Eq + Hash>(
    game: &'a T,
    transposition_table: &'a mut dyn TranspositionTable<T>,
) -> impl Iterator<Item = Result<(T::Move, isize), T::MoveError>> + 'a {
    game.possible_moves().map(move |m| {
        let mut board = game.clone();
        board.make_move(&m)?;
        // We flip the sign of the score because we want the score from the
        // perspective of the player playing the move, not the player whose turn it is.
        Ok((m, -solve(&board, transposition_table)?))
    })
}

type CollectedMoves<T> = Vec<Result<(<T as Game>::Move, isize), <T as Game>::MoveError>>;

/// Parallelized version of `move_scores`. (faster by a large margin)
/// This requires the `rayon` feature to be enabled.
/// It uses rayon's parallel iterators to evaluate the scores of each move in parallel.
///
/// This also allows you to pass in your own hasher, for transposition table optimization.
///
/// # Returns
///
/// A vector of tuples of the form `(move, score)`.
#[cfg(feature = "rayon")]
pub fn par_move_scores_with_hasher<T: Game<Player = impl TwoPlayer> + Eq + Hash + Sync + Send + 'static, S>(game: &T) -> CollectedMoves<T>
where
    T::Move: Sync + Send,
    T::MoveError: Sync + Send,
    S: BuildHasher + Default + Sync + Send + Clone + 'static,
{
    use crate::transposition::TranspositionCache;
    use rayon::prelude::*;
    use std::sync::Arc;

    // we need to collect it first as we cant parallelize an already non-parallel iterator
    let all_moves = game.possible_moves().collect::<Vec<_>>();
    let hashmap = Arc::new(TranspositionCache::<T, S>::new());

    all_moves
        .par_iter()
        .map(move |m| {
            let mut board = game.clone();
            board.make_move(m)?;
            // We flip the sign of the score because we want the score from the
            // perspective of the player pla`ying the move, not the player whose turn it is.
            let mut map = Arc::clone(&hashmap);
            Ok(((*m).clone(), -solve(&board, &mut map)?))
        })
        .collect::<Vec<_>>()
}

/// Parallelized version of `move_scores`. (faster by a large margin)
/// This requires the `rayon` feature to be enabled.
/// It uses rayon's parallel iterators to evaluate the scores of each move in parallel.
///
/// By default, this uses the cryptograpphically unsecure `XxHash64` hasher.
/// If you want to use your own hasher, use [`par_move_scores_with_hasher`].
///
/// # Returns
///
/// A vector of tuples of the form `(move, score)`.
#[cfg(feature = "rayon")]
pub fn par_move_scores<T: Game<Player = impl TwoPlayer> + Eq + Hash + Sync + Send + 'static>(game: &T) -> CollectedMoves<T>
where
    T::Move: Sync + Send,
    T::MoveError: Sync + Send,
{
    if cfg!(feature = "xxhash") {
        use twox_hash::RandomXxHashBuilder64;
        par_move_scores_with_hasher::<T, RandomXxHashBuilder64>(game)
    } else {
        use std::collections::hash_map::RandomState;
        par_move_scores_with_hasher::<T, RandomState>(game)
    }
}
