# Introduction

[Game Theory](https://en.wikipedia.org/wiki/Game_theory) is a general study of games. Many of these games are solved without rigirious computation (for example, where [impartial](https://en.wikipedia.org/wiki/Impartial_game) [combinatorial games](https://en.wikipedia.org/wiki/Combinatorial_game_theory) are solved by generalizing the game to Nim).

However, computation is still important in mathematics as it helps mathematicians find underlying patterns to build said heuristics.

That is the purpose of `game-solver`. It helps solve various games, and allows users to play against the AI, analyze the game tree, and more. It allows programmers to derive the [God's algorithm](https://en.wikipedia.org/wiki/God%27s_algorithm) for any game, as well as derive meaningful statistics about the game. By utilizing powerful tree traversal algorithms and modern machine learning heuristics, we hope to lower the computational bar in combinatorial game theory research.

**The goal of `game-solver` is not to make an *estimated* AI, like Stockfish or AlphaZero**. The goal of this is to make a *perfect* AI. If you want to make an estimated AI, it may be a better idea to use general reinforcement learning instead.

As of now, `game-solver` can only solve 2-player perfect information games. However, the goal is to eventually support more players and imperfect information games.

## Capabilities

**This library is not meant to serve as a substitute for complex game engines** (go, chess).

The thing it does right is God's algorithm computation, to find perfect, full play for generic games.
