use dfdx::prelude::*;

#[cfg(feature = "save")]
use dfdx::safetensors::SafeTensorError;

use self::agent::Agent;
use self::state::State;

use self::strategy::explore::ExplorationStrategy;
use self::strategy::terminate::TerminationStrategy;

pub mod agent;
pub mod state;
pub mod strategy;

const BATCH: usize = 64;

#[derive(Default, Clone, Debug, Sequential)]
#[built(QNetwork)]
struct QNetWorkConfig<const STATE_SIZE: usize, const ACTION_SIZE: usize, const INNER_SIZE: usize> {
    linear1: LinearConstConfig<STATE_SIZE, INNER_SIZE>,
    act1: ReLU,
    linear2: LinearConstConfig<INNER_SIZE, INNER_SIZE>,
    act2: ReLU,
    linear3: LinearConstConfig<INNER_SIZE, ACTION_SIZE>,
}

/// An `DQNAgentTrainer` can be trained for using a certain [Agent](mdp/trait.Agent.html). After
/// training, the `DQNAgentTrainer` contains learned knowledge about the process, and can be queried
/// for this. For example, you can ask the `DQNAgentTrainer` the expected values of all possible
/// actions in a given state.
///
/// The code is partially taken from <https://github.com/coreylowman/dfdx/blob/main/dfdx/examples/advanced-rl-dqn.rs>.
/// and <https://github.com/milanboers/rurel>.
pub struct DQNAgentTrainer<
    S,
    const STATE_SIZE: usize,
    const ACTION_SIZE: usize,
    const INNER_SIZE: usize,
> where
    S: State + Into<[f32; STATE_SIZE]>,
    S::A: Into<[f32; ACTION_SIZE]>,
    S::A: From<[f32; ACTION_SIZE]>,
{
    /// The [discount factor](https://en.wikipedia.org/wiki/Q-learning#Discount_factor) for future rewards.
    gamma: f32,
    /// The Q-network that is being trained.
    q_network: QNetwork<STATE_SIZE, ACTION_SIZE, INNER_SIZE, f32, AutoDevice>,
    /// The target Q-network that is used to compute the target Q-values.
    target_q_net: QNetwork<STATE_SIZE, ACTION_SIZE, INNER_SIZE, f32, AutoDevice>,
    /// The optimizer that is used to train the Q-network.
    sgd: Sgd<QNetwork<STATE_SIZE, ACTION_SIZE, INNER_SIZE, f32, AutoDevice>, f32, AutoDevice>,
    dev: AutoDevice,
    /// Preserves the type of the state.
    phantom: std::marker::PhantomData<S>,
}

impl<S, const STATE_SIZE: usize, const ACTION_SIZE: usize, const INNER_SIZE: usize>
    DQNAgentTrainer<S, STATE_SIZE, ACTION_SIZE, INNER_SIZE>
where
    S: State + Into<[f32; STATE_SIZE]>,
    S::A: Into<[f32; ACTION_SIZE]>,
    S::A: From<[f32; ACTION_SIZE]>,
{
    /// Creates a new `DQNAgentTrainer` with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `gamma` - The [discount factor](https://en.wikipedia.org/wiki/Q-learning#Discount_factor) for future rewards.
    /// * `learning_rate` - The learning rate for the stochastic gradient descent optimizer.
    ///
    /// # Returns
    ///
    /// A new `DQNAgentTrainer` with the given parameters.
    ///
    pub fn new(
        gamma: f32,
        learning_rate: f64,
    ) -> DQNAgentTrainer<S, STATE_SIZE, ACTION_SIZE, INNER_SIZE> {
        let dev = AutoDevice::default();

        // initialize model
        let architecture: QNetWorkConfig<STATE_SIZE, ACTION_SIZE, INNER_SIZE> = Default::default();
        let q_net = dev.build_module::<f32>(architecture);
        let target_q_net = q_net.clone();

        // initialize optimizer
        let sgd = Sgd::new(
            &q_net,
            SgdConfig {
                lr: learning_rate,
                momentum: Some(Momentum::Nesterov(0.9)),
                weight_decay: None,
            },
        );

        DQNAgentTrainer {
            gamma,
            q_network: q_net,
            target_q_net,
            sgd,
            dev,
            phantom: std::marker::PhantomData,
        }
    }

    /// Fetches the learned value for the given `Action` in the given `State`, or `None` if no
    /// value was learned.
    pub fn expected_value(&self, state: &S) -> [f32; ACTION_SIZE] {
        let state_: [f32; STATE_SIZE] = (state.clone()).into();
        let states: Tensor<Rank1<STATE_SIZE>, f32, _> =
            self.dev.tensor(state_).normalize::<Axis<0>>(0.001);
        let actions = self.target_q_net.forward(states).nans_to(0f32);
        actions.array()
    }

    /// Returns a clone of the entire learned state to be saved or used elsewhere.
    pub fn export_learned_values(
        &self,
    ) -> QNetwork<STATE_SIZE, ACTION_SIZE, INNER_SIZE, f32, AutoDevice> {
        self.learned_values().clone()
    }

    // Returns a reference to the learned state.
    pub fn learned_values(
        &self,
    ) -> &QNetwork<STATE_SIZE, ACTION_SIZE, INNER_SIZE, f32, AutoDevice> {
        &self.q_network
    }

    /// Imports a model, completely replacing any learned progress
    pub fn import_model(
        &mut self,
        model: QNetwork<STATE_SIZE, ACTION_SIZE, INNER_SIZE, f32, AutoDevice>,
    ) {
        self.q_network.clone_from(&model);
        self.target_q_net.clone_from(&self.q_network);
    }

    /// Returns the best action for the given `State`, or `None` if no values were learned.
    pub fn best_action(&self, state: &S) -> Option<S::A> {
        let target = self.expected_value(state);

        Some(target.into())
    }

    #[allow(clippy::boxed_local)]
    pub fn train_dqn(
        &mut self,
        states: [[f32; STATE_SIZE]; BATCH],
        actions: [[f32; ACTION_SIZE]; BATCH],
        next_states: [[f32; STATE_SIZE]; BATCH],
        rewards: [f32; BATCH],
        dones: [bool; BATCH],
    ) {
        self.target_q_net.clone_from(&self.q_network);
        let mut grads = self.q_network.alloc_grads();

        let dones: Tensor<Rank1<BATCH>, f32, _> =
            self.dev.tensor(dones.map(|d| if d { 1f32 } else { 0f32 }));
        let rewards = self.dev.tensor(rewards);

        // Convert to tensors and normalize the states for better training
        let states: Tensor<Rank2<BATCH, STATE_SIZE>, f32, _> =
            self.dev.tensor(states).normalize::<Axis<1>>(0.001);

        // Convert actions to tensors and get the max action for each batch
        let actions: Tensor<Rank1<BATCH>, usize, _> = self.dev.tensor(actions.map(|a| {
            let mut max_idx = 0;
            let mut max_val = 0f32;
            for (i, v) in a.iter().enumerate() {
                if *v > max_val {
                    max_val = *v;
                    max_idx = i;
                }
            }
            max_idx
        }));

        // Convert to tensors and normalize the states for better training
        let next_states: Tensor<Rank2<BATCH, STATE_SIZE>, f32, _> =
            self.dev.tensor(next_states).normalize::<Axis<1>>(0.001);

        // Compute the estimated Q-value for the action
        for _step in 0..20 {
            let q_values = self.q_network.forward(states.trace(grads));

            let action_qs = q_values.select(actions.clone());

            // targ_q = R + discount * max(Q(S'))
            // curr_q = Q(S)[A]
            // loss = huber(curr_q, targ_q, 1)
            let next_q_values = self.target_q_net.forward(next_states.clone());
            let max_next_q = next_q_values.max::<Rank1<BATCH>, _>();
            let target_q = (max_next_q * (-dones.clone() + 1.0)) * self.gamma + rewards.clone();

            let loss = huber_loss(action_qs, target_q, 1.0);

            grads = loss.backward();

            // update weights with optimizer
            self.sgd
                .update(&mut self.q_network, &grads)
                .expect("Unused params");
            self.q_network.zero_grads(&mut grads);
        }
        self.target_q_net.clone_from(&self.q_network);
    }

    /// Trains this [DQNAgentTrainer] using the given [ExplorationStrategy] and
    /// [Agent] until the [TerminationStrategy] decides to stop.
    pub fn train(
        &mut self,
        agent: &mut dyn Agent<S>,
        termination_strategy: &mut dyn TerminationStrategy<S>,
        exploration_strategy: &dyn ExplorationStrategy<S>,
    ) {
        loop {
            // Initialize batch
            let mut states: [[f32; STATE_SIZE]; BATCH] = [[0.0; STATE_SIZE]; BATCH];
            let mut actions: [[f32; ACTION_SIZE]; BATCH] = [[0.0; ACTION_SIZE]; BATCH];
            let mut next_states: [[f32; STATE_SIZE]; BATCH] = [[0.0; STATE_SIZE]; BATCH];
            let mut rewards: [f32; BATCH] = [0.0; BATCH];
            let mut dones = [false; BATCH];

            let mut s_t_next = agent.current_state();

            for i in 0..BATCH {
                let s_t = agent.current_state().clone();
                let action = exploration_strategy.pick_action(agent);

                // current action value
                s_t_next = agent.current_state();
                let r_t_next = s_t_next.reward();

                states[i] = s_t.into();
                actions[i] = action.into();
                next_states[i] = (*s_t_next).clone().into();
                rewards[i] = r_t_next as f32;

                if termination_strategy.should_stop(s_t_next) {
                    dones[i] = true;
                    break;
                }
            }

            // train the network
            self.train_dqn(states, actions, next_states, rewards, dones);

            // terminate if the agent is done
            if termination_strategy.should_stop(s_t_next) {
                break;
            }
        }
    }

    #[cfg(feature = "save")]
    pub fn save(&self, path: &str) -> Result<(), SafeTensorError> {
        Ok(self.q_network.save_safetensors(&path)?)
    }

    #[cfg(feature = "save")]
    pub fn load(&mut self, path: &str) -> Result<(), SafeTensorError> {
        Ok(self.q_network.load_safetensors(&path)?)
    }
}

impl<S, const STATE_SIZE: usize, const ACTION_SIZE: usize, const INNER_SIZE: usize> Default
    for DQNAgentTrainer<S, STATE_SIZE, ACTION_SIZE, INNER_SIZE>
where
    S: State + Into<[f32; STATE_SIZE]>,
    S::A: Into<[f32; ACTION_SIZE]>,
    S::A: From<[f32; ACTION_SIZE]>,
{
    fn default() -> Self {
        Self::new(0.99, 1e-3)
    }
}
