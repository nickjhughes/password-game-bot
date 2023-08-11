use ordered_float::NotNan;
use rand::{prelude::*, seq::SliceRandom};
use strum::IntoEnumIterator;

pub use rule::Rule;
pub use state::GameState;

use data::{CAPTCHAS, CHESS_PUZZLES, GEO_GAMES};
use rule::{Color, Coords};

pub mod data;
pub mod helpers;
pub mod rule;
mod state;
#[cfg(test)]
mod tests;

/// An instance of the password game.
#[derive(Debug, Default)]
pub struct Game {
    /// Rules that define this instance of the game.
    pub rules: Vec<Rule>,
    /// Game state.
    pub state: GameState,
}

impl Game {
    /// Start a new game. Instance-specific rules will be chosen randomly.
    pub fn new() -> Self {
        Game {
            rules: Game::random_rules(),
            state: GameState::default(),
        }
    }

    /// Get a full set of game rules, with any instance-specific rules chosen randomly.
    fn random_rules() -> Vec<Rule> {
        let mut rng = thread_rng();
        let mut rules = Vec::new();
        for rule in Rule::iter() {
            match rule {
                Rule::Captcha(_) => rules.push(Rule::Captcha(
                    CAPTCHAS.choose(&mut rng).unwrap().to_string(),
                )),
                Rule::Geo { .. } => {
                    let game = GEO_GAMES.choose(&mut rng).unwrap().clone();
                    rules.push(Rule::Geo(Coords {
                        lat: NotNan::new(game.coordindates.0).unwrap(),
                        long: NotNan::new(game.coordindates.1).unwrap(),
                    }))
                }
                Rule::Chess { .. } => rules.push(Rule::Chess(
                    CHESS_PUZZLES.choose(&mut rng).unwrap().fen.clone(),
                )),
                Rule::Hex(_) => rules.push(Rule::Hex(Color {
                    r: rng.gen::<u8>(),
                    g: rng.gen::<u8>(),
                    b: rng.gen::<u8>(),
                })),
                Rule::Youtube { .. } => rules.push(Rule::Youtube(
                    (2000.0 * rng.gen::<f64>()).floor() as u32 + 180,
                )),
                _ => rules.push(rule),
            }
        }
        rules
    }
}
