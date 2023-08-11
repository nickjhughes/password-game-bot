/// Game state.
#[derive(Debug, Default)]
pub struct GameState {
    /// The highest numbered rule currently being checked.
    pub highest_rule: usize,
    /// The password fire has been started.
    pub fire_started: bool,
    /// Paul's egg has been placed into the password.
    pub egg_placed: bool,
    /// Paul has hatched.
    pub paul_hatched: bool,
    /// Paul is currently eating.
    pub paul_eating: bool,
    /// The letters the player has chosen to sacrifice.
    pub sacrificed_letters: Vec<char>,
}
