# game-solver

solve any sequential game (currently only works with combinatorial games, aka or 2-player perfect-information sequential games)

(more in-depth information can be found in [the book](https://leodog896.github.io/game-solver))

## Contribute

If you want to contribute, new game implementations would be greately appreciated!
The more examples of games that are provided, the more examples that can be used
for benchmarks, analysis, and further optimization.

### Future Plans (Contributions Welcome!)

These are some future features that I've gathered from the few games in examples:

- Game Tree Visualization
- Parallelization w/ rayon
- 2+ player games (multiple agents w/ minimax instead of negamax)
- Imperfect information games
- Games that involve chance
- Trained move ordering (e.g. via a neural network similar to the likes of Stockfish)

## Credits

A lot of the algorithms have been inspired by [Pascal Pons's excellent blog](http://blog.gamesolver.org/solving-connect-four/)
and the general [Chessprogramming wiki](https://www.chessprogramming.org/Main_Page).
