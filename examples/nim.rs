/// Nim is the heart of combinatorial game theory.
/// Its a game about removing objects from heaps.
/// Despite its ability to be rigidly analyzed,
/// it still makes a great example as an implementation of the `Game` trait.
///
/// Learn more about Nim here: <https://en.wikipedia.org/wiki/Nim>
use game_solver::{par_move_scores, Game, Player};
use std::{
    env::args,
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Clone, Hash, Eq, PartialEq)]
struct Nim {
    heaps: Vec<usize>,
    move_count: u32,
    max_score: u32,
}

impl Nim {
    /// Create a new game of Nim with the given heaps,
    /// where heaps is a list of the number of objects in each heap.
    fn new(heaps: Vec<usize>) -> Self {
        Self {
            heaps: heaps.clone(),
            move_count: 0,
            // sum of all the heaps is the upper bound for the amount of moves - add 1 to give a positive score
            max_score: (heaps.iter().sum::<usize>() + 1) as u32,
        }
    }
}

impl Game for Nim {
    /// where Move is a tuple of the heap index and the number of objects to remove
    type Move = (usize, usize);
    type Iter<'a> = std::vec::IntoIter<Self::Move>;

    fn max_score(&self) -> u32 {
        self.max_score
    }

    fn min_score(&self) -> i32 {
        -(self.max_score as i32)
    }

    fn player(&self) -> Player {
        if self.move_count % 2 == 0 {
            Player::One
        } else {
            Player::Two
        }
    }

    // to encourage the AI to win as fast as possible,
    // we want to minimize the amount of moves it takes to win.
    // thus, we penalize the AI for taking more moves
    // by removing points for every move it takes.
    fn score(&self) -> u32 {
        self.max_score() - self.move_count
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

fn main() {
    // parse the original configuration of the game from args
    // e.g. 3,5,7 for 3 heaps with 3, 5, and 7 objects respectively
    let config = args()
        .nth(1)
        .expect("Please provide a configuration of the game, e.g. 3,5,7")
        .split(',')
        .map(|num| num.parse::<usize>().expect("Not a number!"))
        .collect::<Vec<_>>();

    // create a new game of Nim with the given configuration
    let mut game = Nim::new(config);

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(2).for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move((numbers[0], numbers[1]));
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = par_move_scores(&game);

    // check for the win condition
    if move_scores.is_empty() {
        println!("Player {:?} won!", game.player().opponent());
    } else {
        // sort for the best moves first
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("({}, {}), ", game_move.0, game_move.1);
        }
        println!();
    }
}
