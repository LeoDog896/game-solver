# Example: Nim

As an example of implementation, we will be implementing [Nim](https://en.wikipedia.org/wiki/Nim).
Nim is a game that is the heart of modern combinatorial game theory, given its deep presence
in analysing any [impartial](https://en.wikipedia.org/wiki/Impartial_game) game.

However, due to its ease of implementation and flexibility, we will be implementing it
for demonstration.

## How to play Nim

Nim is a game about taking objects from "heaps". Two players play, and each turn, one player gets to remove as many items as they want from any one heap. For example, if there are 3 heaps with `(3, 5, 7)`, player 1 can take 4 objects from the second heap, leaving player two with `(3, 1, 7)`.

There are two variants of Nim, but for this implementation, the winning scenario is the last person to move - aka, once there's `(0, 0, 0)`, the last person who moved wins (aka normal play).
