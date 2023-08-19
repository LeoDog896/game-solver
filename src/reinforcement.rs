use std::collections::VecDeque;
use candle_core::{DType, Device, Tensor};
use rand::seq::SliceRandom;
use candle_nn::{Linear, linear, VarBuilder, VarMap};
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

struct DeepQNetwork {
    input_layer: Linear,
    hidden_layer: Linear,
    output_layer: Linear,
}

impl DeepQNetwork {
    const DETAIL: usize = 128;

    fn new(n_observations: usize, n_actions: usize) -> Self {
        let varmap = VarMap::new();
        let var_builder = VarBuilder::from_varmap(&varmap, DType::F32, &Device::Cpu);

        let input_layer = linear(n_observations, Self::DETAIL, var_builder.clone()).unwrap();
        let hidden_layer = linear(Self::DETAIL, Self::DETAIL, var_builder.clone()).unwrap();
        let output_layer = linear(Self::DETAIL, n_actions, var_builder).unwrap();
        DeepQNetwork {
            input_layer,
            hidden_layer,
            output_layer,
        }
    }

    fn forward(&mut self, image: Tensor) -> Tensor {
        let mut image = image;
        image = self.input_layer.forward(&image).unwrap().relu().unwrap();
        image = self.hidden_layer.forward(&image).unwrap().relu().unwrap();
        self.output_layer.forward(&image).unwrap()
    }
}