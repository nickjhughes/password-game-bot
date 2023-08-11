use lazy_static::lazy_static;
use log::trace;
use std::{collections::HashMap, process::Command};

use crate::driver::DriverError;

lazy_static! {
    pub static ref KEYS: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        // From https://eastmanreference.com/complete-list-of-applescript-key-codes
        m.insert("Tab", 48);
        m.insert("LeftArrow", 123);
        m.insert("RightArrow", 124);
        m.insert("UpArrow", 126);
        m.insert("DownArrow", 125);
        m
    };
}

fn run_applescript(script: &str) -> Result<(), DriverError> {
    trace!("Running AppleScript: {:?}", script);
    let process = Command::new("osascript")
        .arg("-l")
        .arg("AppleScript")
        .arg("-e")
        .arg(script)
        .spawn()
        .expect("Failed to run AppleScript");
    let output = process.wait_with_output().unwrap();
    if output.status.code().unwrap_or_default() == 0 {
        Ok(())
    } else {
        Err(DriverError::AppleScriptError)
    }
}

pub fn press_key_code(code: u8) -> Result<(), DriverError> {
    run_applescript(&format!(
        r#"tell application "System Events" to key code {}"#,
        code
    ))
}

pub fn press_key_code_multiple(code: u8, times: usize) -> Result<(), DriverError> {
    let mut script = String::from("tell application \"System Events\"\n");
    script.push_str(&format!("key code {}\ndelay 0.01\n", code).repeat(times));
    script.push_str("end tell");
    run_applescript(&script)
}
