//! Tic Tac Toe is a traditional two-player game played on a 3x3 grid.
//! For the sake of complexity, this allows simulating any n-dimensional 3-in-a-row game
//! with the same bounds as the traditional game.

use ndarray::{ArrayD, IxDyn, Dim, IxDynImpl, iter::IndexedIter, IntoDimension, Dimension};
use game_solver::{move_scores, Game, Player};

use std::{
    env::args,
    fmt::{Display, Formatter},
    hash::Hash, iter::FilterMap, collections::HashSet,
};

/// The straight size of the board. E.g. if there were 2 dimensions, it would be a SIZE x SIZE board.
const SIZE: usize = 3;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
enum Square {
    Empty,
    X,
    O,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct TicTacToe {
    dim: usize,
    /// True represents a square that has not been eaten
    board: ArrayD<Square>,
    n_moves: u32,
}

fn add_checked(a: Dim<IxDynImpl>, b: Vec<i32>) -> Option<Dim<IxDynImpl>> {
    let mut result = a;
    for (i, j) in result.as_array_view_mut().iter_mut().zip(b.iter()) {
        if *i as i32 + *j < 0 {
            return None;
        }

        *i = ((*i) as i32 + *j).try_into().unwrap();
    }

    Some(result)
}


impl TicTacToe {
    fn new(dim: usize) -> Self {
        // we want [SIZE; dim] but dim isn't a const - we have to get the slice from a vec
        let board = ArrayD::from_elem(IxDyn(&vec![SIZE; dim]), Square::Empty);

        Self {
            dim,
            board,
            n_moves: 0,
        }
    }

    fn winning_line(&self, point: &Dim<IxDynImpl>, offset: &[i32]) -> bool {
        let square = self.board.get(point.clone()).unwrap();

        if *square == Square::Empty {
            return false;
        }

        let mut n = 1;

        let mut current = point.clone();
        while let Some(new_current) = add_checked(current.clone(), offset.to_owned()) {
            current = new_current;
            if self.board.get(current.clone()) == Some(square) {
                n += 1;
            } else {
                break;
            }
        }
        let mut current = point.clone();

        while let Some(new_current) = add_checked(current.clone(), offset.clone().iter().map(|x| -x).collect()) {
            current = new_current;
            if self.board.get(current.clone()) == Some(square) {
                n += 1;
            } else {
                break;
            }
        }


        n >= SIZE
    }

    fn won(&self) -> bool {
        // check every move 
        for (index, square) in self.board.indexed_iter() {
            if square == &Square::Empty {
                continue;
            }

            let point = index.into_dimension();
            for offset in offsets(&point) {
                if self.winning_line(&point, &offset) {
                    return true;
                }
            }
        }

        false
    }
}

impl Game for TicTacToe {
    type Move = Dim<IxDynImpl>;
    type Iter<'a> = FilterMap<IndexedIter<'a, Square, Self::Move>, fn((Self::Move, &Square)) -> Option<Self::Move>>;

    fn max_score(&self) -> u32 {
        (SIZE.pow(self.dim as u32)).try_into().unwrap()
    }

    fn min_score(&self) -> i32 {
        -(SIZE.pow(self.dim as u32) as i32)
    }

    fn player(&self) -> Player {
        if self.n_moves % 2 == 0 {
            Player::P1
        } else {
            Player::P2
        }
    }

    fn score(&self) -> u32 {
        self.max_score() - self.n_moves
    }

    fn make_move(&mut self, m: Self::Move) -> bool {
        if *self.board.get(m.clone()).unwrap() == Square::Empty {
            let square = if self.player() == Player::P1 {
                Square::X
            } else {
                Square::O
            };

            *self.board.get_mut(m).unwrap() = square;
            self.n_moves += 1;
            true
        } else {
            false
        }
    }

    fn possible_moves(&self) -> Self::Iter<'_> {
        self.board.indexed_iter().filter_map(move |(index, square)| {
            if square == &Square::Empty {
                Some(index)
            } else {
                None
            }
        })
    }

    fn is_winning_move(&self, m: Self::Move) -> bool {
        let mut board = self.clone();
        board.make_move(m.clone());

        // check if the board has any matches of SIZE in a row
        // horizontal, diagonal, and vertical
        // wins whenever it meets the following conditions:
        // where (a1, a2, ... an) is the move
        // each an has to follow a set rule:
        // - it stays the same
        // - it increases
        // - it decreases
        // e.g. (0, 0, 2), (0, 1, 1), (0, 2, 0) wins
        for offset in offsets(&m) {
            if board.winning_line(&m, &offset) {
                return true;
            }
        }

        false
    }

    fn is_draw(&self) -> bool {
        self.n_moves == SIZE.pow(self.dim as u32) as u32
    }
}


fn offsets(dim: &Dim<IxDynImpl>) -> HashSet<Vec<i32>> {
    // TODO: this is ridiculously inefficient
    dim
        .as_array_view()
        .iter()
        .map(|&i| (i as i32 - 1)..=(i as i32 + 1))
        .fold(vec![vec![]], |acc, range| {
            let mut new_acc = vec![];
            for i in range {
                for mut vec in acc.clone() {
                    vec.push(i);
                    new_acc.push(vec);
                }
            }
            new_acc
        })
        .into_iter()
        // check if in bounds (all numbers must be between [0..SIZE))
        .filter(|vec|
            vec.iter().all(|&i| i >= 0 && i < SIZE as i32)
        )
        // convert back to usize
        .map(|vec| vec.into_iter().map(|i| i as usize).collect::<Vec<_>>())
        // filter out the current move (we only want neighbors)
        .filter(|vec| vec.iter().ne(dim.as_array_view().iter()))
        // then convert back to i32
        .map(|vec| vec.into_iter().map(|i| i as i32).collect::<Vec<_>>())
        // then subtract the current dimension
        .map(|vec| {
            let mut new_vec = dim.as_array_view().iter().map(|&i| i as i32).collect::<Vec<_>>();
            for (i, &j) in vec.iter().enumerate() {
                new_vec[i] -= j;
            }
            new_vec
        })
        // then filter for duplicate vectors (turn into set)
        .collect::<HashSet<_>>()
}

fn format_dim(dim: &Dim<IxDynImpl>) -> String {
    format!("{:?}", dim.as_array_view().as_slice().unwrap())
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (index, square) in self.board.indexed_iter() {
            writeln!(f, "{:?} @ {}", square, format_dim(&index))?;
        }
        Ok(())
    }
}

fn main() {
    // get the amount of dimensions from the first argument
    let dim = args()
        .nth(1)
        .expect("Please provide a dimension!")
        .parse::<usize>()
        .expect("Not a number!");

    let mut game = TicTacToe::new(dim);

    // parse every move in args, e.g. 0-0 1-1 in args
    args().skip(2).for_each(|arg| {
        let numbers: Vec<usize> = arg
            .split('-')
            .map(|num| num.parse::<usize>().expect("Not a number!"))
            .collect();

        game.make_move(numbers.into_dimension());
    });

    print!("{}", game);
    println!("Player {:?} to move", game.player());

    let mut move_scores = move_scores(&game).collect::<Vec<_>>();

    if game.won() {
        println!("Player {:?} won!", game.player().opposite());
    } else if move_scores.is_empty() {
        println!("No moves left! Game tied!");
    } else {
        move_scores.sort_by_key(|m| m.1);
        move_scores.reverse();

        let mut current_move_score = None;
        for (game_move, score) in move_scores {
            if current_move_score != Some(score) {
                println!("\n\nBest moves @ score {}:", score);
                current_move_score = Some(score);
            }
            print!("{}, ", format_dim(&game_move));
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn best_moves(game: &TicTacToe) -> Option<Dim<IxDynImpl>> {
        move_scores(game).max_by(|(_, a), (_, b)| a.cmp(b)).map(|(m, _)| m)
    }

    #[test]
    fn test_tictactoe() {
        let mut game = TicTacToe::new(2);
        game.make_move(vec![0, 0].into_dimension());

        let best_move = best_moves(&game).unwrap();

        assert_eq!(best_move, vec![1, 1].into_dimension());
    }
}
