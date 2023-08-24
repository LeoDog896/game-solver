# Links

## Credits

A lot of information here would not have been possible without these lovely resources:

- [Chessprogramming](https://www.chessprogramming.org/Main_Page) for their in-depth information about complex game solving
- Various reference implementations:
    - [Stockfish](https://stockfishchess.org/) for lists of heuristics used
    - [AlphaZero](https://www.deepmind.com/blog/alphazero-shedding-new-light-on-chess-shogi-and-go) as a reference for a reinforcement learning implementation.
- [Pascal Pons's guide](http://blog.gamesolver.org/solving-connect-four/04-alphabeta/) for a simple overview of topics covered in chessprogramming
- [Wikipedia](https://en.wikipedia.org/) for external references in regards to game optimization.

## Other engines

There are a few notable programs that also aim to solve specific portions of combinatorial games.

- [GamesCrafters](http://gamescrafters.berkeley.edu/), which solves lightweight combinatorial games with lovely graphic visualization.
- [Glop](https://sprouts.tuxfamily.org/wiki/doku.php?id=home), GNU-licensed software that solves specific combinatorial games by rigirous theory analysis.
- [cgt-rs](https://github.com/t4ccer/cgt-rs), a combinatorial game *theory* calculator, serving as a CAS for combinatorial game theory notation.

For reference, the purpose of this software is to solve generic but *heavy* combinatorial games as fast as possible.
