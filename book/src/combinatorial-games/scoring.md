# Scoring

Writing a score algorithm helps the tree traversal algorithm decide what game state to pursue.

Scoring algorithms fully depend on your implementation. 

For example, in order to pursue a win state in the least amount of moves,
providing `game.max_score() - game.n_moves` (where `n_moves` is the number of moves made
throughout the game) can help guide the AI.

However, if you are only doing a search up to a limited depth (for complex games),
or are looking to pursue a different playstyle, you may need to provide more
complex scoring algorithms, such as the [one used in chessprogramming](https://www.chessprogramming.org/Score#Heuristic_Nodes).