use game_solver::{game::Game, CollectedMoves, GameSolveError};

/// Takes a move list and returns a sorted list, from positive to negative, of
/// the scores.
pub fn normalize_move_scores<T: Game>(
    move_scores: CollectedMoves<T>,
) -> Result<Vec<(T::Move, isize)>, GameSolveError<T>> {
    let mut move_scores = move_scores
        .into_iter()
        .collect::<Result<Vec<_>, GameSolveError<T>>>()?;

    move_scores.sort_by_key(|m| m.1);
    move_scores.reverse();

    Ok(move_scores)
}

pub fn best_move_score<T: Game>(
    move_scores: CollectedMoves<T>,
) -> Result<Option<(T::Move, isize)>, GameSolveError<T>> {
    let move_scores = move_scores
        .into_iter()
        .collect::<Result<Vec<_>, GameSolveError<T>>>()?;

    Ok(move_scores.iter().max_by_key(|x| x.1).cloned())
}

#[cfg(test)]
pub fn best_move_score_testing<T: Game + std::fmt::Debug>(
    move_scores: CollectedMoves<T>,
) -> (T::Move, isize) {
    best_move_score(move_scores).unwrap().unwrap()
}
