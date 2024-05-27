use crate::reinforcement::state::State;

/// A termination strategy decides when to end training.
pub trait TerminationStrategy<S: State> {
    /// If `should_stop` returns `true`, training will end.
    fn should_stop(&mut self, state: &S) -> bool;
}
