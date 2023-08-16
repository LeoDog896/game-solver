# Scoring

Writing a score algorithm helps the tree traversal algorithm decide what game state to pursue.

Scoring algorithms fully depend on your implementation. 

For example, in order to pursue a win state in the least amount of moves,
providing `game.max_score() - game.n_moves` (where `n_moves` is the number of moves made
throughout the game) can help guide the AI. However, this scoring system
is limited as it may ignore winning conditions that are easier to achieve
(e.g. the opponent won't notice how to counter it).

For example, in [Connect Four](https://en.wikipedia.org/wiki/Connect_Four), an optimal scoring algorithm is to try to find
multiple winning combinations (e.g. the winner could have won in 2 ways),
while also taking into account the number of moves as a penalty.

However, if you are only doing a search up to a limited depth (for complex games),
or are looking to pursue a different playstyle, you may need to provide more
complex scoring algorithms, such as the [one used in chessprogramming](https://www.chessprogramming.org/Score#Heuristic_Nodes).

Guessing scores can also help with optimizing move ordering, increasing the
performance of the tree traversal algorithm.
