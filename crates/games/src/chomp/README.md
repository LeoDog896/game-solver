Chomp is a two-player game played on a rectangular grid of squares.
The bottom right square is poisoned, and the players take turns eating squares.
Every square they eat, every square to the right and above it is also eaten (inclusively)

This is a flipped version of the traditional [Chomp](https://en.wikipedia.org/wiki/Chomp) game.

This is not the best example for analysis via a combinatorial game, as not only is it
impartial (making it analyzable via the Sprague-Grundy theorem), but it is also trivially
solved via the strategy-stealing argument.

However, it serves as a great test for the transposition table, as it is a game that commonly
repeats positions (as it only has nxm - 1 positions).
