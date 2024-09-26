//! `game_solver` is a library for solving games.
//!
//! If you want to read how to properly use this library,
//! [the book](https://leodog896.github.io/game-solver/book) is
//! a great place to start.

pub mod game;
pub mod player;
pub mod stats;
// TODO: reinforcement
// #[cfg(feature = "reinforcement")]
// pub mod reinforcement;
pub mod transposition;

use core::panic;
#[cfg(feature = "rayon")]
use std::hash::BuildHasher;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use game::{upper_bound, GameState};
use player::{ImpartialPlayer, TwoPlayer};
use stats::Stats;

use crate::game::Game;
use crate::transposition::{Score, TranspositionTable};
use std::hash::Hash;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameSolveError<T: Game> {
    #[error("could not make a move")]
    MoveError(T::MoveError),
    #[error("the game was cancelled by the token")]
    CancellationTokenError,
}

/// Runs the two-player minimax variant on a zero-sum game.
/// Since it uses alpha-beta pruning, you can specify an alpha beta window.
fn negamax<T: Game<Player = impl TwoPlayer + 'static> + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
    mut alpha: isize,
    mut beta: isize,
    stats: Option<&Stats<T::Player>>,
    cancellation_token: &Option<Arc<AtomicBool>>,
) -> Result<isize, GameSolveError<T>> {
    if let Some(token) = cancellation_token {
        if token.load(Ordering::Relaxed) {
            return Err(GameSolveError::CancellationTokenError);
        }
    }

    if let Some(stats) = stats {
        stats.states_explored.fetch_add(1, Ordering::Relaxed);
    }

    // TODO: debug-based depth counting
    // if let Some(stats) = stats {
    //     stats.max_depth.fetch_max(depth, Ordering::Relaxed);
    // }

    // TODO(perf): if find_immediately_resolvable_game satisfies its contract,
    // we can ignore this at larger depths.
    match game.state() {
        GameState::Playable => (),
        GameState::Tie => {
            if let Some(stats) = stats {
                stats.terminal_ends.tie.fetch_add(1, Ordering::Relaxed);
            }
            return Ok(0);
        }
        GameState::Win(winning_player) => {
            // TODO: can we not duplicate this
            if let Some(stats) = stats {
                if let Ok(player) = castaway::cast!(winning_player, ImpartialPlayer) {
                    if ImpartialPlayer::from_move_count(stats.original_move_count, game.move_count()) == player {
                        stats.terminal_ends.winning.fetch_add(1, Ordering::Relaxed);
                    } else {
                        stats.terminal_ends.losing.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    if stats.original_player == winning_player {
                        stats.terminal_ends.winning.fetch_add(1, Ordering::Relaxed);
                    } else {
                        stats.terminal_ends.losing.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            // if the next player is the winning player,
            // the score should be positive.
            if game.player() == winning_player {
                // we add one to make sure games that use up every move
                // aren't represented by ties.
                //
                // take the 2 heap game where each heap has one object in Nim, for example
                // player 2 will always win since 2 moves will always be used,
                // but since the upper bound is 2, 2 - 2 = 0,
                // but we reserve 0 for ties.
                return Ok(upper_bound(game) - game.move_count() as isize + 1);
            } else {
                return Ok(-(upper_bound(game) - game.move_count() as isize + 1));
            }
        }
    };

    // check if this is a winning configuration
    if let Ok(Some(board)) = game.find_immediately_resolvable_game() {
        match board.state() {
            GameState::Playable => panic!("A resolvable game should not be playable."),
            GameState::Tie => {
                if let Some(stats) = stats {
                    stats.terminal_ends.tie.fetch_add(1, Ordering::Relaxed);
                }
                return Ok(0);
            }
            GameState::Win(winning_player) => {
                if let Some(stats) = stats {
                    if let Ok(player) = castaway::cast!(winning_player, ImpartialPlayer) {
                        if ImpartialPlayer::from_move_count(stats.original_move_count, game.move_count()) == player {
                            stats.terminal_ends.winning.fetch_add(1, Ordering::Relaxed);
                        } else {
                            stats.terminal_ends.losing.fetch_add(1, Ordering::Relaxed);
                        }
                    } else {
                        if stats.original_player == winning_player {
                            stats.terminal_ends.winning.fetch_add(1, Ordering::Relaxed);
                        } else {
                            stats.terminal_ends.losing.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }

                if game.player().turn() == winning_player {
                    return Ok(upper_bound(&board) - board.move_count() as isize + 1);
                } else {
                    return Ok(-(upper_bound(&board) - board.move_count() as isize + 1));
                }
            }
        }
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
                        if let Some(stats) = stats {
                            stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                        }
                        return Ok(beta);
                    }
                }
            }
            Score::LowerBound(min) => {
                if alpha < min {
                    alpha = min;
                    if alpha >= beta {
                        if let Some(stats) = stats {
                            stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                        }
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
        board
            .make_move(&m)
            .map_err(|err| GameSolveError::MoveError::<T>(err))?;

        let score = if first_child {
            -negamax(
                &board,
                transposition_table,
                -beta,
                -alpha,
                stats,
                &cancellation_token,
            )?
        } else {
            let score = -negamax(
                &board,
                transposition_table,
                -alpha - 1,
                -alpha,
                stats,
                &cancellation_token,
            )?;
            if score > alpha {
                -negamax(
                    &board,
                    transposition_table,
                    -beta,
                    -alpha,
                    stats,
                    &cancellation_token,
                )?
            } else {
                score
            }
        };

        // alpha-beta pruning - we can return early
        if score >= beta {
            if let Some(stats) = stats {
                stats.pruning_cutoffs.fetch_add(1, Ordering::Relaxed);
            }
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
pub fn solve<T: Game<Player = impl TwoPlayer + 'static> + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
    stats: Option<&Stats<T::Player>>,
    cancellation_token: &Option<Arc<AtomicBool>>,
) -> Result<isize, GameSolveError<T>> {
    let mut alpha = -upper_bound(game);
    let mut beta = upper_bound(game) + 1;

    // we're trying to guess the score of the board via null windows
    while alpha < beta {
        let med = alpha + (beta - alpha) / 2;

        // do a [null window search](https://www.chessprogramming.org/Null_Window)
        let evaluation = negamax(
            game,
            transposition_table,
            med,
            med + 1,
            stats,
            cancellation_token,
        )?;

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
pub fn move_scores<'a, T: Game<Player = impl TwoPlayer + 'static> + Eq + Hash>(
    game: &'a T,
    transposition_table: &'a mut dyn TranspositionTable<T>,
    stats: Option<&'a Stats<T::Player>>,
    cancellation_token: &'a Option<Arc<AtomicBool>>,
) -> impl Iterator<Item = Result<(T::Move, isize), GameSolveError<T>>> + 'a {
    game.possible_moves().map(move |m| {
        let mut board = game.clone();
        board
            .make_move(&m)
            .map_err(|err| GameSolveError::MoveError(err))?;
        // We flip the sign of the score because we want the score from the
        // perspective of the player playing the move, not the player whose turn it is.
        Ok((
            m,
            -solve(&board, transposition_table, stats, cancellation_token)?,
        ))
    })
}

pub type CollectedMoves<T> = Vec<Result<(<T as Game>::Move, isize), GameSolveError<T>>>;

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
pub fn par_move_scores_with_hasher<
    T: Game<Player = impl TwoPlayer + Sync + 'static> + Eq + Hash + Sync + Send + 'static,
    S,
>(
    game: &T,
    stats: Option<&Stats<T::Player>>,
    cancellation_token: &Option<Arc<AtomicBool>>,
) -> CollectedMoves<T>
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
            board
                .make_move(m)
                .map_err(|err| GameSolveError::MoveError::<T>(err))?;
            // We flip the sign of the score because we want the score from the
            // perspective of the player pla`ying the move, not the player whose turn it is.
            let mut map = Arc::clone(&hashmap);
            Ok((
                (*m).clone(),
                -solve(&board, &mut map, stats, cancellation_token)?,
            ))
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
pub fn par_move_scores<T: Game<Player = impl TwoPlayer + Sync + 'static> + Eq + Hash + Sync + Send + 'static>(
    game: &T,
    stats: Option<&Stats<T::Player>>,
    cancellation_token: &Option<Arc<AtomicBool>>,
) -> CollectedMoves<T>
where
    T::Move: Sync + Send,
    T::MoveError: Sync + Send,
{
    if cfg!(feature = "xxhash") {
        use twox_hash::RandomXxHashBuilder64;
        par_move_scores_with_hasher::<T, RandomXxHashBuilder64>(game, stats, cancellation_token)
    } else {
        use std::collections::hash_map::RandomState;
        par_move_scores_with_hasher::<T, RandomState>(game, stats, cancellation_token)
    }
}
