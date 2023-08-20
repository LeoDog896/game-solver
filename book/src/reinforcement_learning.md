# Reinforcement Learning

In order to make games in `game-solver` easy to make, without the hassle of a deep understanding of strategy, we introduce Reinforcement Learning.

## On Machine Learning in game solving

Machine Learning has recently entered programs that want to find [God's algorithm](https://en.wikipedia.org/wiki/God%27s_algorithm) for games. Usually, this machine learning is involved at the Evaluation step, since many human-written evaluation functions are prone to mistakes, simply because of unknown strategy elements. Stockfish is a notable example of this, who switched their evaluation function to NNUE-based.

However, by taking into account the advancements in research for solving perfect-information games done by AlphaZero (which only has the rules to learn off of), we can use this to optimize our Move Ordering, which in turn, (given that we're correct most of the time), can lead to massive alpha-beta cutoffs. For example, even in the simple case where a Connect 4 solver started moves from the center and then towards the end, traversal time was cut 10-fold.

Thus, if we train an elementary AI via reinforcement learning to get better at move ordering, we can decrease the traversal time by large magnitudes, allowing for games to be solved without implementation-expensive heuristics to be made.

This is not a noval idea, but rather it was not worked on because the goal at the time was to make a fast AI that was better than playing humans at complex games, not a *perfect* AI.

## Implementation

We use [candle](https://github.com/huggingface/candle/) as the library for our reinforcement learning model.

Any `Game` that wants to implement a RL model needs to have:
- a constant list of every single possible move at any moment (for outputs)
- a set transformation to the **exact same input size**.

This is implemented in the `SizableGame` trait.
