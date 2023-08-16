# Performance

`game-solver` takes performance seriously, and demands that all games are as fast as possible.
However, since some games are small (and may run on WASM), all parallelization is optional
and is behind the `rayon` feature flag.

## Internal optimizations

There are a few distinct categories of game optimization:
- Board Representation
- Search algorithms
- Evaluation

The user has most of the control over board representation and evaluation, but `game-solver` does its best to optimize the search algorithms.

### List of applied optimizations

- [Search algorithms](https://en.wikipedia.org/wiki/Search_algorithm):
  - [Negamax](https://en.wikipedia.org/wiki/Negamax) (for 2-player zero-sum games)
    - [Principal Variation Search](https://en.wikipedia.org/wiki/Principal_variation_search) (more popularly known as NegaScout)
  - [Alpha-Beta Pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning) (ignores suboptimal branches, depends on move order)
      - [Iterative Deepening](https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search)
      - [Null window search](https://www.chessprogramming.org/Null_Window)
- Memoization via [Transposition Table](https://en.wikipedia.org/wiki/Transposition_table)
    - Both lower bound and upper bound
    - (Parallelization only):
      - Concurrent hashmap (with [dashmap](https://github.com/xacrimon/dashmap))
      - [xxHash](https://github.com/Cyan4973/xxHash) for hashing.
        - If you want to use xxHash without parallelization, pass it to your hashmap by using `hasher: std::hash::BuildHasherDefault<xxhash_rust::XxHash64>`.
        - If you don't want xxHash at all, it can be disabled by removing the `xxhash` feature flag.
- Parallelization with [rayon](https://github.com/rayon-rs/rayon)
    - Note that this is under the `rayon` feature flag.

## Optimizing your own Games

The [Rust Performance Book](https://nnethercote.github.io/perf-book/) gives great general optimizations, but there are also important steps you can make when working with games.

### General tips:

- **Always remember to compile with --release**.
- `RUSTFLAGS="--emit=asm -C target-cpu=native"` is a great way to do basic compiler optimizations.

### Move Ordering

**This is arguably the most important**.
Making sure your `Game#possible_moves` function guesses what the best moves are first
can save a lot of time, since there are multiple tree-pruning related optimizations
that benefit from good moves being chosen first.

If possible, try to "guess" the score of a move, and sort the moves by that score.

### Efficient Bitboards

Use efficient bitboards - you can look at the examples for inspiration, but make sure your board representation is fast, and *preferably* doesn't need allocation.

Good starting points:
- [BitVec](https://github.com/ferrilab/bitvec) for bool-only arrays
- [ndarray](https://github.com/rust-ndarray/ndarray) for nd arrays (instead of `Vec<Vec<...>>`)
