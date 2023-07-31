# combinatorial-game

negamax, alpha-beta pruning, transposition tables, and more for two-player perfect information games.

## Optimization Tips

### Move Ordering

**This is arguably the most important**. Making sure your `Game#possible_moves` function guesses what the best moves are first
can save a lot of time on alpha-beta pruning and iterative deepening.

### Efficient Bitboards

Use efficient bitboards - you can look at the examples for inspiration, but make sure your board representation is fast, and *preferrably* doesn't need allocation.
