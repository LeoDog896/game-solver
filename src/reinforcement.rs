use std::collections::VecDeque;
use rand::seq::SliceRandom;
use crate::game::Game;

#[derive(Debug, Clone)]
struct Transition<T: Game> {
    state: T,
    action: T::Move,
    next_state: T,
    reward: usize,
}

struct ReplayMemory<T: Game> {
    memory: VecDeque<Transition<T>>,
}

impl<T: Game + Clone> ReplayMemory<T> {
    fn new() -> Self {
        ReplayMemory {
            memory: VecDeque::new(),
        }
    }

    fn push(&mut self, transition: Transition<T>) {
        self.memory.push_back(transition);
    }

    fn sample(&self, batch_size: usize) -> Vec<Transition<T>> {
        let mut rng = rand::thread_rng();
        let mut samples = self.memory.iter().cloned().collect::<Vec<_>>();
        samples.shuffle(&mut rng);
        samples.truncate(batch_size);
        samples
    }

    fn len(&self) -> usize {
        self.memory.len()
    }
}