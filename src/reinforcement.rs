use candle_core::{DType, Device, Tensor};
use candle_nn::{linear, Linear, VarBuilder, VarMap, AdamW, ParamsAdamW};
use rand::seq::SliceRandom;
use std::collections::VecDeque;

use crate::game::Game;

trait EveryMove<M: Eq> {
    fn every_move(&self) -> Vec<M>;
}

/// Represents a transition from 1 state to another,
/// and the action taken to get from that state,
/// as well as the reward associated with it.
#[derive(Debug, Clone)]
struct Transition<S: Clone, A: Clone> {
    state: S,
    action: A,
    next_state: S,
    reward: usize,
}

struct ReplayMemory<S: Clone, A: Clone>(VecDeque<Transition<S, A>>);

impl<S: Clone, A: Clone> ReplayMemory<S, A> {
    fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }

    fn push(&mut self, transition: Transition<S, A>) {
        self.0.push_back(transition);
    }

    fn sample(&self, batch_size: usize) -> Vec<Transition<S, A>> {
        let mut rng = rand::thread_rng();
        let mut samples = self.0.iter().cloned().collect::<Vec<_>>();
        samples.shuffle(&mut rng);
        samples.truncate(batch_size);
        samples
    }

    fn len(&self) -> usize {
        self.0.len()
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

trait Learnable<T: Clone> {
    fn observation_as_terser(&self) -> Tensor;
    fn all_actions() -> Vec<T>;
}

fn train<T: Game + Clone + Learnable<T::Move>>() {
    const GAMMA: f32 = 0.99;
    const EPS_START: f32 = 0.9;
    const EPS_END: f32 = 0.05;
    const EPS_DECAY: usize = 1000;
    const TAU: f32 = 0.005;
    const LR: f64 = 1e-4;

    let actions = T::all_actions();
    let num_observations = 5; // TODO: dont hardcode this - get it from the length of the default tensor

    let policy_net = DeepQNetwork::new(num_observations, actions.len());
    let mut target_net = DeepQNetwork::new(num_observations, actions.len());
    // target_net.load_state_dict(policy_net.state_dict())

    let optimizer = AdamW::new(policy_net.parameters(), ParamsAdamW {
        lr: LR,
        beta1: 0.9,
        beta2: 0.999,
        eps: 1e-8,
        weight_decay: 0.01,
    });

    let memory = ReplayMemory::with_capacity(10_000);
}
