# game-solver

Solve any sequential game (currently only works with [combinatorial games](https://en.wikipedia.org/wiki/Combinatorial_game_theory), aka 2-player perfect-information sequential games).

More in-depth information can be found in [the book](https://leodog896.github.io/game-solver/book).

## Contribute

Rust nightly is required to compile the examples (as `game-solver` uses benches for examples)

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

### Profiling

(TODO: probably should set up benchmarks :3)

Reccomended profiling tools:

#### Flamegraph

`cargo install flamegraph` (requires linux `perf` or windows `dtrace`)

```sh
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --example <example> -- <args>
```

```ps
$env:CARGO_PROFILE_RELEASE_DEBUG='true'; cargo flamegraph --example <example> -- <args>; $env:CARGO_PROFILE_RELEASE_DEBUG=$null
```

## Credits

A lot of the algorithms have been inspired by [Pascal Pons's excellent blog](http://blog.gamesolver.org/solving-connect-four/)
and the general [Chessprogramming wiki](https://www.chessprogramming.org/Main_Page).
