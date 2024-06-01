use super::state::State;

/// An `Agent` is something which hold a certain state, and is able to take actions from that
/// state. After taking an action, the agent arrives at another state.
pub trait Agent<S: State> {
    /// Returns the current state of this `Agent`.
    fn current_state(&self) -> &S;
    /// Takes the given action, possibly mutating the current `State`.
    fn take_action(&mut self, action: &S::A);
    /// Takes a random action from the set of possible actions from this `State`. The default
    /// implementation uses [State::random_action()](trait.State.html#method.random_action) to
    /// determine the action to be taken.
    fn pick_random_action(&mut self) -> S::A {
        let action = self.current_state().random_action();

        self.take_action(&action);

        action
    }
}
