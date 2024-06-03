use game_solver::game::ZeroSumPlayer;

#[derive(Eq, PartialEq)]
pub enum State {
    Player(ZeroSumPlayer),
    Tie,
    Continuing,
}

pub trait GameState {
    fn state(&self) -> State;
}
