# Optimization

The [Rust Performance Book](https://nnethercote.github.io/perf-book/) gives great general optimizations, but there are also important steps you can make when working with games.

## General tips:

- **Always remember to compile with --release**.
- `RUSTFLAGS="--emit=asm -C target-cpu=native"` is a great way to do basic compiler optimizations.

## Move Ordering

**This is arguably the most important**.
Making sure your `Game#possible_moves` function guesses what the best moves are first
can save a lot of time, since there are multiple tree-pruning related optimizations
that benefit from good moves being chosen first.

## Efficient Bitboards

Use efficient bitboards - you can look at the examples for inspiration, but make sure your board representation is fast, and *preferably* doesn't need allocation.

Good starting points:
- [BitVec](https://github.com/ferrilab/bitvec) for bool-only arrays
- [ndarray](https://github.com/rust-ndarray/ndarray) for nd arrays (instead of `Vec<Vec<...>>`)
