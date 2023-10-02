# Making Nim

To make combinatorial games in `game-solver`, you need to implement the `Game` trait on any arbitrary struct.

For starters, we'll make a basic `Nim` struct that contains two variables we need to keep track of:

- `heaps`, which will be a `Vec<usize>`, representing a list of heaps and the amount of objects in them (e.g. `[3, 5, 7]`).
- `move_count`, which is how we will keep track of which player is currently making the move
- `max_score`, which we need to store early on to keep track of the maximum amount of moves that can be made (we'll cover scoring in a bit)

```rs
// we derive some traits we'll need later -
// the hashability is necessary for it to be
// stored in a transposition table.
#[derive(Clone, Hash, Eq, PartialEq)]
struct Nim {
    heaps: Vec<usize>,
    move_count: usize,
}

impl Nim {
    /// Create a new game of Nim with the given heaps,
    /// where heaps is a list of the number of objects in each heap.
    fn new(heaps: Vec<usize>) -> Self {
        Self {
            heaps: heaps.clone(),
            move_count: 0,
            // sum of all the heaps is the upper bound for the amount of moves - add 1 to give a positive score
            max_score: heaps.iter().sum::<usize>() + 1,
        }
    }
}
```

Then, we'll derive the Game trait.

We'll start with:
- a `Move` type, which defines how the Game takes in moves
- an `Iter`, to satisfy Rust by providing the concrete iterator type for our `possible_moves` function
- a `Player` type to decide whether we're a 2-player or an N-player game
- `max_moves` (upper bound on the amount of moves - optional)
- `move_count` (the amount of moves we played)
- `player` (n_moves % 2, defines what player is currently moving)

```rs
impl Game for Nim {
    /// where Move is a tuple of the heap index and the number of objects to remove
    type Move = (usize, usize);
    type Iter<'a> = std::vec::IntoIter<Self::Move>;
    /// Define Nimbers as a zero-sum game
    type Player = ZeroSumPlayer;

    fn max_moves(&self) -> Option<usize> {
        Some(self.max_score)
    }

    fn player(&self) -> ZeroSumPlayer {
        if self.move_count % 2 == 0 {
            ZeroSumPlayer::One
        } else {
            ZeroSumPlayer::Two
        }
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    // ...
}
```

Then, we want our control functions:

- `make_move`, which applies a `Move` to the game.
- `possible_moves`, an iterator of every possible move given the current game state
- `is_winning_move`, to see whether a move will win for the current player
- `is_draw`, whether there is a condition where there are no more moves.

In our case, `is_draw` is always false (as Nim's winning condition *is* no more moves remaining):

```rs
    fn make_move(&mut self, m: &Self::Move) -> bool {
        let (heap, amount) = *m;
        // check for indexing OOB
        if heap >= self.heaps.len() {
            return false;
        }

        // check for removing too many objects
        if amount > self.heaps[heap] {
            return false;
        }
        
        self.heaps[heap] -= amount;
        self.move_count += 1;
        true
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();
        
        // loop through every heap and add every possible move
        for (i, &heap) in self.heaps.iter().enumerate() {
            for j in 1..=heap {
                moves.push((i, j));
            }
        }

        moves.into_iter()
    }

    // a move is winning if the next player
    // has no possible moves to make
    fn is_winning_move(&self, m: &Self::Move) -> bool {
        let mut board = self.clone();
        board.make_move(m);
        board.possible_moves().next().is_none()
    }

    // Nim can never be a draw - 
    // if there are no possible moves, the game is already won
    fn is_draw(&self) -> bool {
        false
    }
```

Full code:

```rs
use game_solver::{Game, Player};
use std::hash::Hash;

#[derive(Clone, Hash, Eq, PartialEq)]
struct Nim {
    heaps: Vec<usize>,
    move_count: usize,
    max_score: usize,
}

impl Nim {
    /// Create a new game of Nim with the given heaps,
    /// where heaps is a list of the number of objects in each heap.
    fn new(heaps: Vec<usize>) -> Self {
        Self {
            heaps: heaps.clone(),
            move_count: 0,
            // sum of all the heaps is the upper bound for the amount of moves
            max_score: heaps.iter().sum::<usize>(),
        }
    }
}

impl Game for Nim {
    /// where Move is a tuple of the heap index and the number of objects to remove
    type Move = (usize, usize);
    type Iter<'a> = std::vec::IntoIter<Self::Move>;

    fn max_moves(&self) -> Option<usize> {
        Some(self.max_score)
    }

    fn player(&self) -> Player {
        if self.move_count % 2 == 0 {
            Player::One
        } else {
            Player::Two
        }
    }

    fn move_count(&self) -> usize {
        self.move_count
    }

    fn make_move(&mut self, m: Self::Move) -> bool {
        let (heap, amount) = m;
        // check for indexing OOB
        if heap >= self.heaps.len() {
            return false;
        }

        // check for removing too many objects
        if amount > self.heaps[heap] {
            return false;
        }

        self.heaps[heap] -= amount;
        self.move_count += 1;
        true
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        let mut moves = Vec::new();

        // loop through every heap and add every possible move
        for (i, &heap) in self.heaps.iter().enumerate() {
            for j in 1..=heap {
                moves.push((i, j));
            }
        }

        moves.into_iter()
    }

    // a move is winning if the next player
    // has no possible moves to make (normal play for Nim)
    fn is_winning_move(&self, m: Self::Move) -> bool {
        let mut board = self.clone();
        board.make_move(m);
        board.possible_moves().next().is_none()
    }

    // Nim can never be a draw -
    // if there are no possible moves, the game is already won
    fn is_draw(&self) -> bool {
        false
    }
}

impl Display for Nim {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (i, &heap) in self.heaps.iter().enumerate() {
            writeln!(f, "Heap {}: {}", i, heap)?;
        }
        Ok(())
    }
}
```
