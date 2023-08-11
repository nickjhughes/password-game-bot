use driver::Driver;
use log::{error, info};

mod driver;
mod game;
mod password;
mod solver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or(());

    loop {
        let solver = solver::Solver::default();
        let mut driver = driver::web::WebDriver::new(solver)?;
        match driver.play() {
            Ok(()) => {
                // Success! Sleep to give the user time to enjoy it
                std::thread::sleep(std::time::Duration::from_secs(1000));
                break;
            }
            Err(e) => {
                match e {
                    driver::DriverError::CouldNotSatisfyRule(rule) => {
                        // Try again
                        info!("Failed to satisfy rule {:?}, playing again...", rule);
                        continue;
                    }
                    driver::DriverError::GameOver => {
                        // Try again
                        info!("Game over, playing again...");
                        continue;
                    }
                    driver::DriverError::LostSync => {
                        // Try again
                        info!(
                            "Lost password sync for unknown reason, playing again in 30 seconds..."
                        );
                        std::thread::sleep(std::time::Duration::from_secs(30));
                        continue;
                    }
                    e => {
                        // Other error, give user time to debug
                        error!("An error occurred: {:?}", e);
                        std::thread::sleep(std::time::Duration::from_secs(1000));
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
