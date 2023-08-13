# Solving Nim

Now that we have our implementation, we can get to solving Nim with a little CLI.

We'll begin by writing a quick display function for us to show the current game:

```rs
impl Display for Nim {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (i, &heap) in self.heaps.iter().enumerate() {
            writeln!(f, "Heap {}: {}", i, heap)?;
        }
        Ok(())
    }
}
```

Then, write some parsing utilities for our arguments, and make our game:
```rs
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

   // ... 
}
```

Finally, we'll solve the game using the `par_move_scores` function, or the parallelized equivalent of `move_scores`: (this requires the `rayon` feature to be enabled)

```rs
    let mut move_scores = move_scores(&game, &mut HashMap::new()).collect::<Vec<_>>();

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
```

And to use the CLI, do:

```sh
cargo run --features rayon --example nim 3,5,7
```

```
Heap 0: 3
Heap 1: 5
Heap 2: 7
Player One to move


Best moves @ score 1:
(2, 1), (1, 1), (0, 1),

Best moves @ score -4:
(2, 3), (2, 2), (0, 3), (0, 2),

Best moves @ score -8:
(2, 7), (2, 6), (2, 5), (2, 4), (1, 5), (1, 4), (1, 3), (1, 2),
```

Full source on [GitHub](https://github.com/LeoDog896/game-solver/blob/master/examples/nim.rs).

## Further Room for Optimization

Since Nim is relatively simple, it runs pretty fast. However, for complex games, the [Optimization Tips](../optimization_tips.md) may help you.
