Zener is a loopy finite combinatorial game played on a five by seven grid.
Past `game-solver`'s demo, you can play it online at [boardgamearena](https://en.boardgamearena.com/gamepanel?game=zener).

The first row is where Right's unique starting pieces are located,
and the last row is where Left's unique starting pieces are located.

The first player to move is Left, and the allowed moves per piece are only
adjacent horizontal and vertical tiles.

After the first turn, a player makes two moves: they must move the 'type' of piece
that the other player moved (there are five types: Left has the orientation `1 2 3 4 5`
and Right has the orientation `5 4 3 2 1` relative to a side of the board) first,
then they can freely move another piece. This alternates until either no player can make a move,
or a player gets a piece past the first or last row.

Players can 'lock' a piece by moving on top of it: only the piece on top can move, and multiple
pieces can be stacked at a time.
