# game-solver

solve any sequential game (currently only works with combinatorial games, or 2-player perfect-information sequential games)
## Features

- 2-player optimization with [Negamax](https://en.wikipedia.org/wiki/Negamax)
- Perfect information optimizations:
  - [Transposition Table](https://en.wikipedia.org/wiki/Transposition_table)
  - [Alpha-Beta Pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning)
    - [Iterative Deepening of alpha/beta search](https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search)

## Optimization Tips

### Move Ordering

**This is arguably the most important**.
Making sure your `Game#possible_moves` function guesses what the best moves are first
can save a lot of time on alpha-beta pruning and iterative deepening.

### Efficient Bitboards

Use efficient bitboards - you can look at the examples for inspiration, but make sure your board representation is fast, and *preferably* doesn't need allocation.

## Credits

A lot of the algorithms have been inspired by [Pascal Pons's excellent blog](http://blog.gamesolver.org/solving-connect-four/)
and the general [Chessprogramming wiki](https://www.chessprogramming.org/Main_Page).

## Future Plans (Contributions Welcome!)

- Game Tree Visualization
- Parallelization as a feature
- 2+ player games (multiple agents w/ minimax instead of negamax)
- Non-perfect information games
  - Games that involve chance
