use thiserror::Error;

use crate::{game::Rule, solver::Solver};

pub mod direct;
pub mod web;

/// Defines a password game driver that a bot can use to play the game.
pub trait Driver {
    /// Construct a new instance of the driver with the given solver.
    fn new(solver: Solver) -> Result<Self, DriverError>
    where
        Self: Sized;

    /// Play the game.
    fn play(&mut self) -> Result<(), DriverError>;
}

/// Failure modes for drivers.
#[derive(Debug, Error)]
pub enum DriverError {
    #[error("could not satisfy rule {0:?}")]
    CouldNotSatisfyRule(Rule),
    #[error("game over")]
    GameOver,
    #[error("lost password sync")]
    LostSync,
    #[error("launch options builder failed")]
    LaunchOptionsBuilderError,
    #[cfg(target_os = "macos")]
    #[error("apple script error")]
    AppleScriptError,
    #[error("headless chrome error")]
    HeadlessChrome(#[from] anyhow::Error),
    #[error("failed to deserialize game rule")]
    RuleDeserialization(#[from] serde_plain::Error),
}
