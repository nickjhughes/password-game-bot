use log::info;

use super::{Driver, DriverError};
use crate::{
    game::{Game, Rule},
    solver::Solver,
};

mod game_logic;

/// A driver for direct interaction with an instance of `Game`.
/// Will spawn a random instance of the game on creation.
pub struct DirectDriver {
    /// The game itself.
    game: Game,
    /// The solver which will attempt to play the game.
    solver: Solver,
}

impl DirectDriver {
    fn get_violated_rules(&mut self) -> Result<Vec<Rule>, DriverError> {
        let mut violated_rules = Vec::new();
        for rule in &self.game.rules {
            if rule.number() - 1 < self.game.state.highest_rule {
                if !rule.validate(self.solver.password.raw_password(), &self.game.state) {
                    violated_rules.push(rule.clone());
                }
            } else if violated_rules.is_empty() {
                // Move up to the next rule if all below are satisfied
                self.game.state.highest_rule += 1;

                // Some rules require game state updates
                match rule {
                    Rule::Egg => {
                        self.game.state.egg_placed = true;
                    }
                    Rule::Fire => {
                        self.game.state.fire_started = true;
                        game_logic::start_fire(&mut self.solver.password);
                        // TODO: Implement fire spread logic. Every 1100ms fire should spread.
                    }
                    Rule::Hatch => {
                        self.game.state.paul_hatched = true;
                        game_logic::hatch_egg(&mut self.solver.password);
                        // TODO: Implement Paul eating logic:
                        //       Every 20 seconds, a bug is removed from the password.
                        //       If there aren't any bugs in the password, game over
                        //         (Paul has starved).
                        //       If there are >= 9 bugs, game over (Paul was overfed).
                    }
                    _ => {}
                }

                if !rule.validate(self.solver.password.raw_password(), &self.game.state) {
                    violated_rules.push(rule.clone());
                }
            }
        }
        Ok(violated_rules)
    }
}

impl Driver for DirectDriver {
    fn new(solver: Solver) -> Result<Self, DriverError> {
        Ok(DirectDriver {
            game: Game::new(),
            solver,
        })
    }

    fn play(&mut self) -> Result<(), DriverError> {
        let mut violated_rules = self.get_violated_rules()?;
        while !violated_rules.is_empty() {
            info!(
                "Password: {:?}, violated rules: {:?}",
                self.solver.password.as_str(),
                violated_rules
            );
            let first_rule = violated_rules.pop().unwrap();
            let changes = self.solver.solve_rule(&first_rule, &self.game.state, 0);
            if let Some(changes) = changes {
                for change in changes {
                    self.solver.password.queue_change(change);
                }
                self.solver.password.commit_changes();
            } else {
                return Err(DriverError::CouldNotSatisfyRule(first_rule));
            }
            if self.game.state.sacrificed_letters != self.solver.sacrificed_letters {
                self.game.state.sacrificed_letters.clear();
                self.game
                    .state
                    .sacrificed_letters
                    .extend(self.solver.sacrificed_letters.iter());
            }

            violated_rules = self.get_violated_rules()?;
        }
        info!("Game complete!");
        Ok(())
    }
}
