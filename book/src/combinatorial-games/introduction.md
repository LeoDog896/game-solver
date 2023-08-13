# Combinatorial Games

[Combinatorial Games](https://en.wikipedia.org/wiki/Combinatorial_game_theory) are (as of writing) the only type of game that `game-solver` can solve. 

## List of applied optimizations

Combinatorial games are the most restricted in their feature set, being sequential and have perfect information. This leaves them open to multiple optimizations:

- [Negamax](https://en.wikipedia.org/wiki/Negamax) (2-player only fast scoring)
- [Transposition Table](https://en.wikipedia.org/wiki/Transposition_table)
    - both lower bound and upper bound
- [Alpha-Beta Pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning) (ignore suboptimal branches)
- [Iterative Deepening of alpha/beta search](https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search)
- [Null window search](https://www.chessprogramming.org/Null_Window)
- Parallelization with [rayon](https://github.com/rayon-rs/rayon)
    - Note that this is under the `rayon` feature flag.
