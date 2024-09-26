# Performance

`game-solver` takes performance seriously - any possible generic performance that can be applied should be applied.

## List of applied optimizations

- [Search algorithms](https://en.wikipedia.org/wiki/Search_algorithm):
  - [Negamax](https://en.wikipedia.org/wiki/Negamax) (for 2-player zero-sum games)
    - [Principal Variation Search](https://en.wikipedia.org/wiki/Principal_variation_search) (more popularly known as NegaScout)
  - [Alpha-Beta Pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning) (ignores suboptimal branches, depends on move order)
    - [Iterative Deepening](https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search)
    - [Null window search](https://www.chessprogramming.org/Null_Window)
- Memoization via [Transposition Tables](https://en.wikipedia.org/wiki/Transposition_table).
  - Both lower bound and upper bound
  - (Parallelization only):
    - Concurrent memory-based HashMap cache via [moka](https://github.com/moka-rs/moka).
      - TODO: Use depth-first cache removal
  - [xxHash](https://github.com/Cyan4973/xxHash) for fast hashing.
    - If you want to use xxHash without parallelization, pass it to your hashmap by using `hasher: std::hash::BuildHasherDefault<xxhash_rust::XxHash64>`.
    - You can disable xxhash by removing the `xxhash` feature.
      - More information about why you may want to do this can be found in the [hashing](#hashing) section
- Parallelization with [rayon](https://github.com/rayon-rs/rayon)
  - Note that this is under the `rayon` feature flag.
  - TODO: Use Lazy SMP (currently this is using naive parallelization on the `move_scores` level)

## Optimizing your own Games

The [Rust Performance Book](https://nnethercote.github.io/perf-book/) gives great general optimizations, but there are also important steps you can make when working with games.

**Always remember to compile with --release**.

### Move Ordering

**This is the most important algorithm-based optimization.**.
Making sure your `Game#possible_moves` function guesses what the best moves are first
can save a lot of time, since there are multiple tree-pruning related optimizations
that benefit from good moves being chosen first.

You can also use `game-solver`'s [reinforcement learning](./reinforcement_learning.md) method, which is highly recommended as it saves time on manual implementation.

If possible, try to "guess" the score of a move, and sort the moves by that score.

Since `game-solver` uses principal variation search, if the first move in the move ordering is great,
this solver will generally work very fast.

### Efficient Bitboards

Use efficient bitboards - you can look at the examples for inspiration, but make sure your board representation is fast, and *preferably* doesn't need allocation.

Good starting points:

- [BitVec](https://github.com/ferrilab/bitvec) for bool-only arrays
- [ndarray](https://github.com/rust-ndarray/ndarray) for nd arrays (instead of `Vec<Vec<...>>`)

### Hashing

Transposition tables require hashing to store the game board as a key and retrieve it later for efficiency.

Since the type of game board is not known, `game-solver` uses the fastest general-purpose hash function available: [xxHash](https://github.com/Cyan4973/xxHash).
However, if you know your game board can be hashed faster, you can provide your own hasher to the transposition table.

For example, in Chess, the most common way to hash a board is to use a [Zobrist Hash](https://en.wikipedia.org/wiki/Zobrist_hashing).
This can be generalized to any type of board, aka [Tabulation Hashing](https://en.wikipedia.org/wiki/Tabulation_hashing).
