# Reinforcement Learning

In order to make games in `game-solver` easy to make, without the hassle of a deep understanding of strategy, we introduce Reinforcement Learning. This is also referred to as the [Neural MoveMap Heuristic](https://www.chessprogramming.org/Neural_MoveMap_Heuristic). Essentially, we train a model via Reinforcement Learning to predict what the best order for the moves to make is.

## Implementation

We use [candle](https://github.com/huggingface/candle/) as the library for our reinforcement learning model. The model training is based on DQN (TODO: use DDPG).

Any `Game` that wants to implement a RL model needs to have:
- a constant list of every single possible move at any moment (for outputs)
- a set transformation to the **exact same input size**.

This is implemented in the `SizableGame` trait.
