# Reinforcement Learning

In order to make games in `game-solver` easy to make, without the hassle of a deep understanding of strategy, we introduce Reinforcement Learning.

## On Machine Learning in game solving

Machine Learning has recently entered programs that want to find [God's algorithm](https://en.wikipedia.org/wiki/God%27s_algorithm) for games. Usually, this machine learning is involved at the Evaluation step, since many human-written evaluation functions are prone to mistakes, simply because of unknown strategy elements. 

However, by taking into account the advancements in research for solving perfect-information games done by AlphaZero (which only has the rules to learn off of), we can use this to optimize our Move Ordering, which in turn, given that we're correct most of the time, can lead to massive alpha-beta cutoffs. For example, even in the simple case where a Connect 4 solver started moves from the center and then towards the end, traversal time was cut by 10-fold.

Thus, if we train an elementary AI via reinforcement learning to get better at move ordering, we can decrease the traversal time by large magnitudes, allowing for games to be solved without implementation-expensive heuristics to be made.

## Implementation

We use [candle](https://github.com/huggingface/candle/) as the library for our reinforcement learning model.

Since we're dealing with any generic input, and any sort of Move for an output, 
