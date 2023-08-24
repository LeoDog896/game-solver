use dfdx::{
    nn::builders::*,
    optim::Adam,
    prelude::DeviceBuildExt,
    tensor::{AutoDevice, Gradients, Storage, Tensor, AsArray, TensorToArray},
    tensor_ops::{AdamConfig, Device, WeightDecay}, shapes::{Const, Rank1},
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

trait Learnable<T: Clone, Input, const O: usize> {
    fn all_actions() -> [T; O];
    fn observations(&self) -> Input;
}

type Built<D: Storage<Dtype>, N: BuildOnDevice<D, Dtype>> = N::Built;

/// An Agent is a DDQN agent that learns to play a game by playing against itself.
/// 
/// This is mainly intended for Move Ordering - essentially,
/// by training an AI via self-play, you can get a good move ordering for your game.
/// 
/// This means less fine tuning, and more focus on analysis.
struct Agent<
    T: Game + Clone + Learnable<T::Move, Input, O>,
    Input,
    const O: usize,
    NN: BuildOnDevice<D, Dtype> + ZeroSizedModule,
    D: Device<Dtype> + DeviceBuildExt + TensorToArray<Rank1<O>, Dtype, Array = [Dtype; O]> = AutoDevice,
> {
    /// List of all possible actions, legal or not given the current state.
    actions: [T::Move; O],
    /// The online network, which is the network that is used to make moves.
    online: Built<D, NN>,
    /// The target network, which is the network that is used to calculate the target values.
    target: Built<D, NN>,
    /// The optimizer used to update the online network.
    optimizer: Adam<Built<D, NN>, Dtype, D>,
    /// The replay memory used to store transitions.
    memory: ReplayMemory<T, T::Move>,
    /// The gradients of the online network.
    gradients: Gradients<f32, D>,
    /// The number of steps that the agent has taken. This is used to decide how much randomness to add to the agent's moves.
    steps_done: usize,
    /// The device that the agent is on. (CPU or GPU)
    device: D,
    _phantom: std::marker::PhantomData<Input>,
}

impl<
    T: Game + Clone + Learnable<T::Move, Input, O>,
    Input,
    const O: usize,
    NN: BuildOnDevice<D, f32> + ZeroSizedModule,
    D: Device<Dtype> + DeviceBuildExt + TensorToArray<Rank1<O>, Dtype, Array = [Dtype; O]>,
> Agent<T, Input, O, NN, D>
where
    // moves must be comparable - this allows us to return the actual move instead of the index of the move
    T::Move: PartialEq<T::Move>,
    // the network must be cloneable (for us to build a policy network and a target network),
    // and must return a 1d tensor (for the output of the network to be the move scores per move idx)
    Built<D, NN>: Clone + Module<Input, Output = Tensor<Rank1<O>, Dtype, D>>
{
    const BATCH_SIZE: usize = 128;

    fn new() -> Self {
        const LR: f64 = 1e-4;

        let actions = T::all_actions();

        let device: D = Default::default();
        let online = device.build_module::<NN, Dtype>();
        let target = online.clone();
        
        // we only need gradients for the online network
        let gradients = online.alloc_grads();

        let optimizer: Adam<_, Dtype, _> = Adam::new(
            &online,
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
            online,
            target,
            optimizer,
            memory,
            gradients,
            steps_done: 0,
            device,
            _phantom: Default::default(),
        }
    }

    fn act(&mut self, state: &T) -> T::Move {
        let legal_moves = state.possible_moves().collect::<Vec<_>>();
        let sample = rand::random::<f32>();

        // small decay function, where f(x) = p + (p_0 - p) * e^(-x / d)
        const EPS_START: f32 = 0.9;
        const EPS_END: f32 = 0.05;
        const EPS_DECAY: usize = 100;
        let eps_threshold = EPS_END
            + (EPS_START - EPS_END) * E.powf(-1. * self.steps_done as f32 / EPS_DECAY as f32);

        self.steps_done += 1;
        // use the neural network to make a move
        if sample > eps_threshold {
            let move_scores = self.online.forward(state.observations())
                .array().to_vec().iter()
                // convert idx -> (move, score)
                .enumerate()
                .map(|(i, s)| (self.actions[i].clone(), *s))
                // then filter by legal moves
                .filter(|(m, _)| legal_moves.contains(m))
                .collect::<Vec<_>>();

            // get the best move
            let best_move = move_scores
                .iter()
                .max_by(|(_, s1), (_, s2)| s1.partial_cmp(s2).unwrap())
                .unwrap()
                .0
                .clone();
            best_move
        } else {
            // otherwise, choose a random move
            legal_moves.choose(&mut rand::thread_rng()).unwrap().clone()
        }
    }

    fn cache(&mut self, state: &T, action: &T::Move, next_state: &T, reward: usize) {
        self.memory.push(Transition {
            state: state.clone(),
            action: action.clone(),
            next_state: next_state.clone(),
            reward,
        });
    }

    fn sync(&mut self) {
        self.target = self.online.clone();
    }

    fn td_estimate(&self, state: &T, action: &T::Move) -> f32 {
        let move_scores = self.online.forward(state.observations());
        let action_idx = self.actions.iter().position(|a| a == action).unwrap();
        move_scores.array().to_vec()[action_idx]
    }
    
    fn td_target(&self, reward: usize, next_state: &T, done: bool) -> f32 {
        let next_state_q = self.online.forward(next_state.observations());
        let best_action = next_state_q.array().to_vec().iter().enumerate().max_by(|(_, s1), (_, s2)| s1.partial_cmp(s2).unwrap()).unwrap().0;
        let next_q = self.target.forward(next_state.observations()).array().to_vec()[best_action];
        (reward as f32 + (1. - done as u8 as f32) * 0.99 * next_q).into()
    }
}
