use std::hash::Hash;

use rand::seq::SliceRandom;

/// A `State` is something which has a reward, and has a certain set of actions associated with it.
/// The type of the actions must be defined as the associated type `A`.
pub trait State: Eq + Hash + Clone {
    /// Action type associate with this `State`.
    type A: Eq + Hash + Clone;

    /// The reward for when an `Agent` arrives at this `State`.
    ///
    /// Rewards are relative to each other, and are traditionally smaller integers.
    fn reward(&self) -> f64;
    /// The set of actions that can be taken from this `State`, to arrive in another `State`.
    fn actions(&self) -> Vec<Self::A>;
    /// Selects a random action that can be taken from this `State`. The default implementation
    /// takes a uniformly distributed random action from the defined set of actions. You may want
    /// to improve the performance by only generating the necessary action.
    fn random_action(&self) -> Self::A {
        let actions = self.actions();
        actions
            .choose(&mut rand::thread_rng())
            .cloned()
            .expect("No actions available; perhaps use the SinkStates termination strategy?")
    }
}
