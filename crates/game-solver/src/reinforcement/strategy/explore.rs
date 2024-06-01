use crate::reinforcement::{agent::Agent, state::State};

/// Trait for exploration strategies. An exploration strategy decides, based on an `Agent`, which
/// action to take next.
pub trait ExplorationStrategy<S: State> {
    /// Selects the next action to take for this `Agent`.
    fn pick_action(&self, agent: &mut dyn Agent<S>) -> S::A;
}

/// The random exploration strategy.
/// This strategy always takes a random action, as defined for the
/// Agent by
/// Agent::take_random_action()
pub struct RandomExploration;

impl Default for RandomExploration {
    fn default() -> Self {
        Self
    }
}

impl<S: State> ExplorationStrategy<S> for RandomExploration {
    fn pick_action(&self, agent: &mut dyn Agent<S>) -> S::A {
        agent.pick_random_action()
    }
}
