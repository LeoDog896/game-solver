# combinatorial-game

negamax, alpha-beta pruning, transposition tables, and more for two-player perfect information games.

## Optimization Tips

Use efficient bitboards - you can look at the examples for inspiration, but make sure your board representation is fast, and *preferrably* doesn't need allocation.