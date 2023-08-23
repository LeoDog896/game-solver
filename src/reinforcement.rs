use dfdx::{
    nn::builders::*,
    optim::Adam,
    prelude::{DeviceBuildExt, Linear, ReLU},
    tensor::{AsArray, AutoDevice, Gradients, Storage, TensorFrom},
    tensor_ops::{AdamConfig, Device, WeightDecay},
};
use rand::seq::SliceRandom;
use std::{collections::VecDeque, f32::consts::E};

use crate::game::Game;

type Dtype = f32;

trait EveryMove<M: Eq> {
    fn every_move(&self) -> Vec<M>;
}

/// Represents a transition from 1 state to another,
/// and the action taken to get from that state,
/// as well as the reward associated with it.
#[derive(Debug, Clone)]
struct Transition<S: Clone, A: Clone> {
    state: Vec<S>,
    action: Vec<A>,
    next_state: Vec<S>,
    reward: Vec<usize>,
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

const DETAIL: usize = 128;
type DeepQNetwork<const I: usize, const O: usize> = (
    (Linear<I, DETAIL>, ReLU),
    (Linear<DETAIL, DETAIL>, ReLU),
    Linear<DETAIL, O>,
);

trait Learnable<T: Clone, const I: usize, const O: usize> {
    fn all_actions() -> [T; O];
    fn observations(&self) -> [Dtype; I];
}

type Built<D: Storage<Dtype>, N: BuildOnDevice<D, Dtype>> = N::Built;

struct Agent<
    T: Game + Clone + Learnable<T::Move, I, O>,
    const I: usize,
    const O: usize,
    D: Device<Dtype> = AutoDevice,
> where
    DeepQNetwork<I, O>: BuildOnDevice<D, f32>,
{
    /// List of all possible actions, legal or not given the current state.
    actions: [T::Move; O],
    /// The policy network, which is the network that we are training.
    net: Built<D, DeepQNetwork<I, O>>,
    optimizer: Adam<Built<D, DeepQNetwork<I, O>>, Dtype, D>,
    memory: ReplayMemory<T, T::Move>,
    gradients: Gradients<f32, D>,
    steps_done: usize,
    device: D,
}

impl<T: Game + Clone + Learnable<T::Move, I, O>, const I: usize, const O: usize> Agent<T, I, O>
where
    T::Move: PartialEq<T::Move>,
{
    fn new() -> Self {
        const BATCH_SIZE: usize = 128;
        const LR: f64 = 1e-4;

        let actions = T::all_actions();

        let device = AutoDevice::default();
        let net = device.build_module::<DeepQNetwork<I, O>, Dtype>();

        let gradients = net.alloc_grads();

        let optimizer: Adam<_, Dtype, AutoDevice> = Adam::new(
            &net,
            AdamConfig {
                lr: LR,
                betas: [0.5, 0.25],
                eps: 1e-6,
                weight_decay: Some(WeightDecay::Decoupled(1e-2)),
            },
        );

        let memory = ReplayMemory::<T, T::Move>::with_capacity(10_000);

        Self {
            actions,
            net,
            optimizer,
            memory,
            gradients,
            steps_done: 0,
            device,
        }
    }

    fn act(&mut self, state: &T) -> T::Move {
        let legal_moves = state.possible_moves().collect::<Vec<_>>();
        let sample = rand::random::<f32>();
        const EPS_START: f32 = 0.9;
        const EPS_END: f32 = 0.05;
        const EPS_DECAY: usize = 100;
        let eps_threshold = EPS_END
            + (EPS_START - EPS_END) * E.powf(-1. * self.steps_done as f32 / EPS_DECAY as f32);
        self.steps_done += 1;
        // use the neural network to make a move
        if sample > eps_threshold {
            let move_scores = self
                .net
                .forward(self.device.tensor(state.observations()))
                .array();
            let best_move = move_scores
                .iter()
                .enumerate()
                // only legal moves
                .filter(|(i, _)| legal_moves.contains(&self.actions[*i]))
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .unwrap()
                .0;
            self.actions[best_move].clone()
        } else {
            legal_moves.choose(&mut rand::thread_rng()).unwrap().clone()
        }
    }
}
