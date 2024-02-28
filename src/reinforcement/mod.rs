mod collate;

use dfdx::{
    data::*,
    losses,
    prelude::*,
    shapes::{Const, Rank1, Shape},
    tensor::{AsArray, AutoDevice, Gradients, Storage, Tensor, TensorFrom, TensorToArray, Trace},
    tensor_ops::{AdamConfig, Device, WeightDecay},
};

use dfdx::data::IteratorBatchExt;
use itertools::Itertools;
use rand::seq::SliceRandom;
use std::{collections::VecDeque, f32::consts::E};

use crate::game::Game;
use crate::reinforcement::collate::IteratorCollateExt;

type ModelDtype = f32;

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
    done: bool,
}

impl<S: Clone, A: Clone> Transition<S, A> {
    fn new(state: S, action: A, next_state: S, reward: usize, done: bool) -> Self {
        Self {
            state,
            action,
            next_state,
            reward,
            done,
        }
    }
}

struct ReplayMemory<S: Clone, A: Clone>(VecDeque<Transition<S, A>>);

impl<S: Clone, A: Clone> ExactSizeDataset for ReplayMemory<S, A> {
    type Item<'a> = Transition<S, A> where S: 'a, A: 'a;
    fn get(&self, index: usize) -> Self::Item<'_> {
        self.0[index].clone()
    }
    fn len(&self) -> usize {
        self.len()
    }
}

impl<S: Clone, A: Clone> ReplayMemory<S, A> {
    fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }

    fn push(&mut self, transition: Transition<S, A>) {
        self.0.push_back(transition);
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

trait Learnable<T: Clone, Input, const O: usize> {
    fn all_actions() -> [T; O];
    fn observe(&self) -> Input;
}

/// Type utility to build a module on a device.
type Built<D: Storage<ModelDtype>, N: BuildOnDevice<D, ModelDtype>> = N::Built;

/// An Agent is a DDQN agent that learns to play a game by playing against itself.
///
/// This is mainly intended for Move Ordering - essentially,
/// by training an AI via self-play, you can get a good move ordering for your game.
///
/// This means less fine tuning, and more focus on analysis.
struct Agent<
    T: Game + Clone + Learnable<T::Move, Tensor<Input, ModelDtype, AutoDevice>, O>,
    Input: Shape + AddDim<usize>,
    const O: usize,
    NN: BuildOnDevice<ModelDtype, AutoDevice>,

> {
    /// List of all possible actions, legal or not given the current state.
    actions: [T::Move; O],
    /// The online network, which is the network that is used to make moves.
    online: Built<AutoDevice, NN>,
    /// The target network, which is the network that is used to calculate the target values.
    target: Built<AutoDevice, NN>,
    /// The optimizer used to update the online network.
    optimizer: Adam<Built<AutoDevice, NN>, ModelDtype, AutoDevice>,
    /// The replay memory used to store transitions.
    memory: ReplayMemory<T, T::Move>,
    /// The gradients of the online network.
    gradients: Gradients<ModelDtype, AutoDevice>,
    /// The number of steps that the agent has taken. This is used to decide how much randomness to add to the agent's moves.
    steps_done: usize,
    /// The device that the agent is on. (CPU or GPU)
    device: AutoDevice,
    _phantom: std::marker::PhantomData<Input>,
}

const BATCH_SIZE: usize = 128;

impl<
        T: Game + Clone + Learnable<T::Move, Tensor<Input, ModelDtype, AutoDevice>, O>,
        Input: Shape + AddDim<usize>,
        const O: usize,
        NN: BuildOnDevice<ModelDtype, AutoDevice>,
    > Agent<T, Input, O, NN>
where
    // moves must be comparable - this allows us to return the actual move instead of the index of the move
    T::Move: PartialEq<T::Move>,
    // the network must be cloneable (for us to build a policy network and a target network),
    // and must return a 1d tensor (for the output of the network to be the move scores per move idx)
    Built<AutoDevice, NN>:
        Clone + Module<Tensor<Input, ModelDtype, AutoDevice>, Output = Tensor<Rank1<O>, ModelDtype, AutoDevice>>,
{
    pub fn new() -> Self {
        const LR: f64 = 1e-4;

        let actions = T::all_actions();

        let device = AutoDevice::default();
        let online = device.build_module::<NN, ModelDtype>(device);
        let target = online.clone();

        // we only need gradients for the online network
        let gradients = online.alloc_grads();

        let optimizer: Adam<_, ModelDtype, _> = Adam::new(
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
        let sample = rand::random::<ModelDtype>();

        // small decay function, where f(x) = p + (p_0 - p) * e^(-x / d)
        const EPS_START: ModelDtype = 0.9;
        const EPS_END: ModelDtype = 0.05;
        const EPS_DECAY: usize = 100;
        let eps_threshold = EPS_END
            + (EPS_START - EPS_END) * E.powf(-1. * self.steps_done as ModelDtype / EPS_DECAY as ModelDtype);

        self.steps_done += 1;
        // use the neural network to make a move
        if sample > eps_threshold {
            let move_scores = self
                .online
                .forward(state.observe())
                .array()
                .to_vec()
                .iter()
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
            done: false,
        });
    }

    fn sync(&mut self) {
        self.target = self.online.clone();
    }

    fn td_estimate(&self, state: &T, action: &T::Move) -> ModelDtype {
        let move_scores = self.online.forward(state.observe());
        let action_idx = self.actions.iter().position(|a| a == action).unwrap();
        move_scores.array().to_vec()[action_idx]
    }

    fn td_target(&self, reward: usize, next_state: &T, done: bool) -> ModelDtype {
        const GAMMA: ModelDtype = 0.99;
        let next_state_q = self.online.forward(next_state.observe());
        let best_action = next_state_q
            .array()
            .to_vec()
            .iter()
            .enumerate()
            .max_by(|(_, s1), (_, s2)| s1.partial_cmp(s2).unwrap())
            .unwrap()
            .0;
        let next_q = self.target.forward(next_state.observe()).array().to_vec()[best_action];
        (reward as ModelDtype + (1. - done as u8 as ModelDtype) * GAMMA * next_q).into()
    }

    fn update_q_online(
        &mut self,
        predict: Tensor<Rank1<O>, ModelDtype, AutoDevice>,
        target: Tensor<Rank1<O>, ModelDtype, AutoDevice>,
    ) -> ModelDtype {
        let loss = losses::smooth_l1_loss(predict, target, 0.01);
        self.online.zero_grads(&mut self.gradients);
        self.gradients = loss.backward();
    }

    fn save(&self, path: &str) {
        // TODO: save the agent
    }

    fn sample(&self) {
        let mut rng = rand::thread_rng();
        let data = self
            .memory
            .shuffled(&mut rng)
            .map(|t| {
                let state = t.state.clone();
                let action = t.action.clone();
                let next_state = t.next_state.clone();
                let reward = t.reward;
                (
                    state.observe(),
                    next_state.observe(),
                    self.device
                        .tensor(self.actions.iter().position(|a| a == &action).unwrap() as ModelDtype),
                    self.device.tensor(reward as ModelDtype),
                )
            })
            .batch_exact(Const::<BATCH_SIZE>)
            .collate()
            .stack()
            .collect();

        (a, b, c, d)
    }

    fn learn(&mut self) {
        const LEARN_EVERY: usize = 3;
        const SYNC_EVERY: usize = 1000;
        const SAVE_EVERY: usize = 10000;

        if self.steps_done % SYNC_EVERY == 0 {
            self.sync();
        }

        if self.steps_done % SAVE_EVERY == 0 {
            self.save("");
        }

        if self.steps_done < BATCH_SIZE {
            return;
        }

        if self.steps_done % LEARN_EVERY != 0 {
            return;
        }

        let transitions = self.sample();

        // let td_est = states
        //     .iter()
        //     .zip(actions.iter())
        //     .map(|(s, a)| self.td_estimate(s, a))
        //     .collect::<Vec<_>>();
        // let td_tgt = rewards
        //     .iter()
        //     .zip(next_states.iter())
        //     .map(|(r, ns)| self.td_target(*r, ns, false))
        //     .collect::<Vec<_>>();

        // let loss = self.update_q_online(td_est, td_tgt);

        // (td_est.mean().item(), loss)
    }
}
