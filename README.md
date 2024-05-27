# [game-solver](https://leodog896.github.io/game-solver)

![Crates.io](https://img.shields.io/crates/v/game-solver)
[![Discord](https://img.shields.io/discord/1140401094338556009)](https://discord.gg/VjbCyaX29C)

Solve any sequential game, including:
- [Combinatorial Games](https://en.wikipedia.org/wiki/Combinatorial_game_theory) - or 2-player perfect-information games

More in-depth information can be found in [the book](https://leodog896.github.io/game-solver/book).

## Background

[Game Theory](https://en.wikipedia.org/wiki/Game_theory) is a general study of games. Many of these games are solved without rigirous computation (for example, where [impartial](https://en.wikipedia.org/wiki/Impartial_game) [combinatorial games](https://en.wikipedia.org/wiki/Combinatorial_game_theory) are solved by generalizing the game to Nim).

However, in order to apply game theory to more complex games, computation is required. This is where `game-solver` comes in.

## Contribute

Rust nightly is required to compile the examples (as `game-solver` uses benches for examples)

If you want to contribute, new game implementations would be greately appreciated!
The more examples of games that are provided, the more examples that can be used
for benchmarks, analysis, and further optimization.

### Future Plans (Contributions Welcome!)

These are some future features that I've gathered:

- Game Tree Analysis
  - Visualization
  - [Game complexity](https://en.wikipedia.org/wiki/Game_complexity) information
- 2+ player games (multiple agents w/ minimax instead of negamax)
  - TODO: there is a second player option, but currently only the `ZeroSumPlayer` works. The `castaway` crate can help with this.
- Imperfect information games
- Games that involve chance (Expectiminimax / Expectinegamax)
- Benchmarks
- impartial Nim utilities

### Profiling

Recommended profiling tools:

#### Flamegraph

`cargo install flamegraph` (requires linux `perf` or windows `dtrace`)

```sh
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --example <example> -- <args>
```

```ps
$env:CARGO_PROFILE_RELEASE_DEBUG='true'; cargo flamegraph --example <example> -- <args>; $env:CARGO_PROFILE_RELEASE_DEBUG=$null
```
