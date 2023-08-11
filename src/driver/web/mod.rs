use anyhow::Context;
use headless_chrome::{browser::tab::ModifierKey, Browser, LaunchOptionsBuilder, Tab};
use lazy_regex::regex;
use log::{debug, error, info, trace};
use ordered_float::NotNan;
use std::{collections::HashMap, sync::Arc, time::Instant};
use strum::EnumCount;
use unicode_segmentation::UnicodeSegmentation;

use super::{Driver, DriverError};
use crate::{
    game::{GameState, Rule},
    password::{
        format::{FontFamily, FontSize},
        Change, FormatChange,
    },
    solver::Solver,
};
use helpers::{extract_color_from_css_style, extract_fen_from_svg, parse_formatting};

mod helpers;
#[cfg(target_os = "macos")]
mod osascript;
#[cfg(test)]
mod tests;
#[cfg(target_os = "windows")]
mod winapi;

const RULE_VALIDATION_WAIT_TIME: std::time::Duration = std::time::Duration::from_millis(100);
const GAME_URL: &str = "https://neal.fun/password-game/";

/// A driver for the actual game at https://neal.fun/password-game/.
pub struct WebDriver {
    /// A browser handle. Needs to be kept around because if it's dropped the connection
    /// to the browser is closed.
    _browser: Browser,
    /// The active tab with the password game open.
    pub tab: Arc<Tab>,
    /// The solver which will attempt to play the game.
    solver: Solver,
    /// State of the game, synced to the actual game's state.
    pub game_state: GameState,
    /// Position of the cursor in the password field.
    cursor: usize,
    /// Time when we started playing the game.
    start_time: Option<Instant>,
    /// Time when Paul was last fed.
    paul_last_fed: Option<Instant>,
}

impl Driver for WebDriver {
    fn new(solver: crate::solver::Solver) -> Result<Self, DriverError> {
        let browser = Browser::new(
            LaunchOptionsBuilder::default()
                .headless(false)
                .idle_browser_timeout(std::time::Duration::from_secs(10 * 60))
                .build()
                .map_err(|_| DriverError::LaunchOptionsBuilderError)?,
        )?;

        let tabs = browser.get_tabs();
        let tab = if tabs
            .lock()
            .expect("failed to get lock on browser tabs")
            .is_empty()
        {
            browser.new_tab()?
        } else {
            tabs.lock()
                .expect("failed to get lock on browser tabs")
                .last()
                .unwrap()
                .clone()
        };
        tab.activate()?;

        tab.navigate_to(GAME_URL)?;
        tab.wait_for_element("div.ProseMirror")?.click()?;

        // Set focus to password field
        #[cfg(target_os = "windows")]
        for _ in 0..5 {
            winapi::press_and_release_key(winapi::KEYS.get("Tab").unwrap());
        }
        #[cfg(target_os = "macos")]
        osascript::press_key_code_multiple(*osascript::KEYS.get("Tab").unwrap(), 5)?;

        Ok(WebDriver {
            _browser: browser,
            tab,
            solver,
            game_state: GameState::default(),
            cursor: 0,
            start_time: None,
            paul_last_fed: None,
        })
    }

    fn play(&mut self) -> Result<(), DriverError> {
        // Start playthrough timer
        self.start_time = Some(Instant::now());

        // Enter initial password to trigger rule evaluation
        let mut changes = self.solver.starting_password();
        self.update_password(&mut changes)?;

        let mut violated_rules = self.get_violated_rules()?;
        while !violated_rules.is_empty() {
            info!(
                "Password: {:?}, violated rules: {:?}",
                self.solver.password.as_str(),
                violated_rules
            );

            if violated_rules.len() == 1 && violated_rules[0] == Rule::Final {
                #[cfg(target_os = "macos")]
                let modifier = ModifierKey::Meta;
                #[cfg(not(target_os = "macos"))]
                let modifier = ModifierKey::Ctrl;

                // Copy our password, so we can quickly "retype" it
                self.tab.find_element("div.ProseMirror")?.click()?;
                self.tab.press_key_with_modifiers("A", Some(&[modifier]))?;
                self.tab.press_key_with_modifiers("C", Some(&[modifier]))?;

                // Click yes, this is our final password
                let buttons = self.tab.find_elements(".final-password button")?;
                for button in buttons {
                    if button.get_inner_text()?.trim() == "Yes" {
                        button.click()?;
                        break;
                    }
                }

                // Wait for the second box
                std::thread::sleep(std::time::Duration::from_millis(500));

                // Paste to "retype" our password
                let input_boxes = self.tab.find_elements("div.ProseMirror")?;
                for input_box in input_boxes.iter() {
                    if input_box.get_inner_text()?.trim().is_empty() {
                        input_box.click()?;
                        self.tab.press_key_with_modifiers("V", Some(&[modifier]))?;

                        break;
                    }
                }

                // Confirm success
                let _ = self.tab.wait_for_element(".end-screen")?;
                info!(
                    "Completed game in {:.2}",
                    self.time_since_start().unwrap().as_secs_f32()
                );
                return Ok(());
            } else if violated_rules.iter().any(|r| *r == Rule::Fire) {
                // Just delete the whole password and retype it to get rid of the fire
                self.delete_and_retype_passsword()?;
                // Wait a bit for rules to update
                std::thread::sleep(std::time::Duration::from_millis(500));
            } else {
                if violated_rules.iter().any(|r| *r == Rule::Hatch) {
                    // Paul hatched, so we need to resync the password
                    self.solver.password.raw_password_mut().replace(0, "🐔");
                    assert_eq!(self.solver.password.as_str(), self.get_password()?);
                }

                let first_rule = violated_rules.pop().unwrap();

                let changes = if first_rule == Rule::IncludeLength
                    && self.solver.length_string.is_some()
                    && (violated_rules.is_empty()
                        || (violated_rules.len() == 1 && violated_rules[0] == Rule::PrimeLength))
                {
                    // We're just waiting for the number of bugs to make the password length correct,
                    // so we can just adjust the number bugs manually
                    debug!("Manually adjusting bugs to match goal length");
                    let current_bugs = self
                        .get_password()?
                        .graphemes(true)
                        .filter(|g| *g == "🐛")
                        .count();
                    let current_length = self.solver.password.len();
                    let goal_length = *self.solver.goal_length.as_ref().unwrap();
                    if current_length + current_bugs < goal_length {
                        // Add bugs
                        let total_to_add = goal_length - (current_length + current_bugs);
                        let (bugs_to_add, padding_to_add) = if total_to_add + current_bugs > 8 {
                            // Don't overfeed Paul!
                            let bugs_to_add = total_to_add.min(8 - current_bugs);
                            (bugs_to_add, total_to_add - bugs_to_add)
                        } else {
                            (total_to_add, 0)
                        };
                        self.cursor_to(self.solver.password.len())?;
                        for _ in 0..bugs_to_add {
                            self.tab.send_character("🐛")?;
                        }
                        for _ in 0..bugs_to_add {
                            self.cursor_left(true)?;
                        }
                        self.paul_last_fed = Some(Instant::now());

                        if padding_to_add > 0 {
                            Some(vec![Change::Append {
                                string: "-".repeat(padding_to_add),
                                protected: false,
                            }])
                        } else {
                            None
                        }
                    } else if current_length + current_bugs > goal_length {
                        // Remove bugs
                        let to_remove = current_length + current_bugs - goal_length;
                        self.cursor_to(self.solver.password.len())?;
                        for _ in 0..to_remove {
                            self.cursor_right(true)?;
                        }
                        for _ in 0..to_remove {
                            self.tab.press_key("Backspace")?;
                        }
                        None
                    } else {
                        unreachable!();
                    }
                } else {
                    // Assume 3 extra bugs:
                    // - if currently fewer, we'll feed Paul eventually
                    // - if currently more, Paul will eat his way down to 3 eventually
                    self.solver.solve_rule(&first_rule, &self.game_state, 3)
                };

                if let Some(mut changes) = changes {
                    if first_rule == Rule::Hatch {
                        // Paul hatching is a special case
                        // To make keeping the password in sync much easier, we append
                        // the bugs to the input field, but _not_ to our internal
                        // representation of the password. Then we continue as normal,
                        // and when Paul eats a bug, it doesn't mess with our sync.
                        self.cursor_to(self.solver.password.len())?;
                        // We can insert up to 8 🐛's before Paul is overfed
                        for _ in 0..8 {
                            self.tab.send_character("🐛")?;
                        }
                        for _ in 0..8 {
                            self.cursor_left(true)?;
                        }
                        self.paul_last_fed = Some(Instant::now());
                    } else {
                        self.update_password(&mut changes)?;
                    }
                } else {
                    return Err(DriverError::CouldNotSatisfyRule(first_rule));
                }

                if self.game_state.sacrificed_letters != self.solver.sacrificed_letters {
                    assert_eq!(first_rule, Rule::Sacrifice);
                    self.game_state.sacrificed_letters.clear();
                    self.game_state
                        .sacrificed_letters
                        .extend(self.solver.sacrificed_letters.iter());

                    // Select sacrificed letters in game
                    let mut buttons_clicked = 0;
                    let button_elements = self.tab.find_elements("button.letter")?;
                    // This assumes the buttons appear in alphabetical order
                    for (i, button) in button_elements.iter().enumerate() {
                        for letter in &self.game_state.sacrificed_letters {
                            if i == *letter as usize - 'a' as usize {
                                button.click()?;
                                buttons_clicked += 1;
                            }
                        }
                    }
                    assert_eq!(buttons_clicked, 2);
                    let sacrifice_button = self.tab.find_element("button.sacrafice-btn")?;
                    sacrifice_button.click()?;

                    // Focus back on password field
                    self.tab
                        .find_element("div.ProseMirror")
                        .unwrap()
                        .click()
                        .unwrap();
                    // And move cursor to start (clicking back in the box seems to change the cursor
                    // position)
                    for _ in 0..self.solver.password.len() {
                        self.cursor_left(true)?;
                    }
                    trace!("Cursor {}->0", self.cursor);
                    self.cursor = 0;
                }
            }

            if self.game_state.highest_rule < Rule::Final.number() {
                // Make sure Paul doesn't starve
                self.feed_paul()?;
            }

            violated_rules = self.get_violated_rules()?;
            info!(
                "Play time: {:.2} seconds",
                self.time_since_start().unwrap().as_secs_f32()
            );
        }
        Ok(())
    }
}

/// The result of a sync check of the passwore.
#[derive(Debug)]
enum CheckResult {
    /// Password is in sync.
    Synced,
    /// Password out of sync due to fire.
    Fire,
    /// Password out of sync due to Paul hatching.
    Hatched,
}

impl WebDriver {
    /// Get the current duration of time since we started playing.
    /// Returns none if we haven't started playing yet.
    fn time_since_start(&self) -> Option<std::time::Duration> {
        self.start_time.map(|t| t.elapsed())
    }

    /// Check if Paul needs feeding, and if so, add some bugs.
    fn feed_paul(&mut self) -> Result<(), DriverError> {
        if !self.game_state.paul_hatched {
            return Ok(());
        }
        let time_since_last_fed = self.paul_last_fed.unwrap().elapsed();
        debug!(
            "Paul last fed {} seconds ago",
            time_since_last_fed.as_secs_f32()
        );

        // Every 60 seconds, top up his bugs
        if time_since_last_fed.as_secs_f32() >= 60.0 {
            let current_bugs = self
                .get_password()?
                .graphemes(true)
                .filter(|g| *g == "🐛")
                .count();
            let bugs_to_add = 8 - current_bugs;

            self.cursor_to(self.solver.password.len())?;

            self.reset_formatting()?;

            for _ in 0..bugs_to_add {
                self.tab.send_character("🐛")?;
            }
            for _ in 0..bugs_to_add {
                self.cursor_left(true)?;
            }
            self.paul_last_fed = Some(Instant::now());
        }

        Ok(())
    }

    /// Delete the whole password and retype it. Useful for putting out the fire.
    /// To avoid slaying Paul ("🥚"), we actually don't delete the whole password,
    /// but replace it with "🥚" in one go (then retype the rest of the password).
    pub fn delete_and_retype_passsword(&mut self) -> Result<(), DriverError> {
        #[cfg(target_os = "macos")]
        let modifier = ModifierKey::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = ModifierKey::Ctrl;

        self.tab.press_key_with_modifiers("A", Some(&[modifier]))?;
        self.tab.send_character("🥚")?;

        // The Ctrl/Cmd+A select all doesn't seem to always get the whole thing,
        // so clean up after it if necessary
        let remaining_password_len = self.get_password()?.graphemes(true).count();
        if remaining_password_len > 1 {
            for _ in 0..(remaining_password_len - 1) {
                self.cursor_right(true)?;
            }
            for _ in 0..(remaining_password_len - 1) {
                self.tab.press_key("Backspace")?;
            }
        }

        let formatting = self.solver.password.raw_password().formatting();
        // Start with bold in a known state
        if self.is_bold()? {
            self.toggle_bold()?;
        }
        for (i, grapheme) in self
            .solver
            .password
            .as_str()
            .graphemes(true)
            .enumerate()
            .skip(1)
        {
            if (formatting[i].bold && !formatting[i - 1].bold)
                || (!formatting[i].bold && formatting[i - 1].bold)
            {
                self.toggle_bold()?;
            }
            self.tab.send_character(grapheme)?;
        }
        if formatting.last().unwrap().bold {
            // Leave bold off
            self.toggle_bold()?;
        }
        trace!("Cursor {}->{}", self.cursor, self.solver.password.len());
        self.cursor = self.solver.password.len();

        assert_eq!(self.solver.password.as_str(), self.get_password()?);

        Ok(())
    }

    fn check_password_formatting(&mut self) -> Result<CheckResult, DriverError> {
        let password_box = self.tab.find_element("div.ProseMirror")?;
        let html = password_box.get_content()?;
        let formatting = parse_formatting(&html);

        if formatting == self.solver.password.raw_password().formatting() {
            Ok(CheckResult::Synced)
        } else {
            error!("Formatting mismatch:");
            error!(
                "Expected: {:?}",
                self.solver.password.raw_password().formatting()
            );
            error!("Actual: {:?}", formatting);
            Err(DriverError::LostSync)
        }
    }

    /// Check if the password on the page is the same as what we've stored.
    /// This could fail if:
    ///  - Something went wrong when we updated the password
    ///  - Fire was started in the password
    ///  - Paul hatched from an egg into a chicken
    ///  - Paul ate a bug
    /// This function will resync the password in the latter three cases, or
    /// just panic in the first case.
    fn check_password(&mut self) -> Result<CheckResult, DriverError> {
        let actual_password = self.get_password()?.replace('🐛', "");
        if actual_password == self.solver.password.as_str() {
            return self.check_password_formatting();
        }

        // The fire was started – this is dealt with in the `play` function
        if actual_password.contains('🔥') {
            debug!("Password sync lost due to fire");
            return Ok(CheckResult::Fire);
        }

        // Paul hatched
        if self.solver.password.as_str().replace('🥚', "🐔") == actual_password {
            debug!("Password sync lost due to Paul hatching");
            // Paul is always at index 0, which makes this easier
            self.solver.password.raw_password_mut().replace(0, "🐔");
            return Ok(CheckResult::Hatched);
        }

        // Paul died
        if self.solver.password.as_str().replace('🐔', "🪦") == actual_password {
            debug!("Password sync lost due to Paul starving");
            // We can't recover from this, it's game over
            return Err(DriverError::GameOver);
        }

        // Otherwise, we've lost sync for some other reason, and don't know how to recover
        error!("Password sync lost due to unknown reason");
        error!(
            "Expected: {:?}, found: {:?}",
            self.solver.password.as_str(),
            actual_password
        );
        Err(DriverError::LostSync)
    }

    /// Update the password by processing the given changes.
    pub fn update_password(&mut self, changes: &mut [Change]) -> Result<(), DriverError> {
        if changes.is_empty() {
            return Ok(());
        }

        if self.game_state.highest_rule > Rule::BoldVowels.number() {
            // Don't bother checking until we get to a stage where the game can modify the password
            // underneath us
            self.check_password()?;
        }

        Self::sort_changes_for_entry(changes);

        // Combine formatting for speed if possible
        let deduped_formatting_changes = {
            let mut c = Vec::new();
            for change in changes.iter() {
                if let Change::Format { format_change, .. } = change {
                    c.push(format_change);
                }
            }
            c.sort();
            c.dedup();
            c
        };
        if changes.iter().all(|c| matches!(c, Change::Format { .. }))
            && deduped_formatting_changes.len() == 1
        {
            let (mut start_index, format_change) = match &changes[0] {
                Change::Format {
                    index,
                    format_change,
                } => (*index, format_change),
                _ => unreachable!(),
            };
            let mut length = 1;
            let mut combined_changes = Vec::new();
            for change in changes.iter().skip(1) {
                let index = match &change {
                    Change::Format { index, .. } => *index,
                    _ => unreachable!(),
                };
                if index > start_index + length {
                    combined_changes.push((start_index, length));
                    start_index = index;
                    length = 1;
                } else {
                    length += 1;
                }
            }
            combined_changes.push((start_index, length));

            let mut touched_bold = false;
            for (start_index, length) in combined_changes {
                self.cursor_to(start_index)?;
                // Select
                #[cfg(target_os = "windows")]
                {
                    winapi::press_key(winapi::KEYS.get("Shift").unwrap());
                    winapi::press_key(winapi::KEYS.get("RShift").unwrap());
                }
                for _ in 0..length {
                    #[cfg(target_os = "windows")]
                    winapi::press_and_release_key(winapi::KEYS.get("NumpadRight").unwrap());
                    #[cfg(not(target_os = "windows"))]
                    self.tab
                        .press_key_with_modifiers("ArrowRight", Some(&[ModifierKey::Shift]))?;
                    trace!("Cursor {}->{}", self.cursor, self.cursor + 1);
                    self.cursor += 1;
                }
                #[cfg(target_os = "windows")]
                {
                    winapi::release_key(winapi::KEYS.get("RShift").unwrap());
                    winapi::release_key(winapi::KEYS.get("Shift").unwrap());
                }
                // Format
                match format_change {
                    FormatChange::BoldOn => {
                        touched_bold = true;
                        self.toggle_bold()?;
                    }
                    FormatChange::ItalicOn => {
                        self.toggle_italic()?;
                    }
                    FormatChange::FontSize(font_size) => {
                        self.select_font_size(font_size, None)?;
                    }
                    FormatChange::FontFamily(font_family) => {
                        self.select_font(font_family)?;
                    }
                }
                // Deselect
                self.tab.press_key("ArrowRight")?;
            }
            if touched_bold && self.is_bold()? {
                self.toggle_bold()?;
            }
            for change in changes.iter() {
                self.solver.password.queue_change(change.clone());
            }
        } else {
            let mut removed_count = 0;
            let mut already_appended = false;
            let mut already_prepended = false;
            let mut touched_bold = false;
            for change in changes.iter() {
                debug!("Applying change {:?}", change);
                match change {
                    Change::Format {
                        index,
                        format_change,
                    } => {
                        self.cursor_to(*index)?;
                        // Select
                        self.tab
                            .press_key_with_modifiers("ArrowRight", Some(&[ModifierKey::Shift]))?;
                        // Format
                        match format_change {
                            FormatChange::BoldOn => {
                                touched_bold = true;
                                self.toggle_bold()?;
                            }
                            FormatChange::ItalicOn => {
                                self.toggle_italic()?;
                            }
                            FormatChange::FontSize(font_size) => {
                                self.select_font_size(
                                    font_size,
                                    Some(
                                        &self.solver.password.raw_password().formatting()[*index]
                                            .font_size
                                            .clone(),
                                    ),
                                )?;
                            }
                            FormatChange::FontFamily(font_family) => {
                                self.select_font(font_family)?;
                            }
                        }
                        // Deselect
                        self.tab.press_key("ArrowRight")?;
                        trace!("Cursor {}->{}", self.cursor, self.cursor + 1);
                        self.cursor += 1;
                    }
                    Change::Append { string, .. } => {
                        if !already_appended {
                            // All appends are done together, so we only need to move the cursor
                            // to the end for the first one.
                            // This seems like it'd be a no-op, but because we don't commit the changes
                            // to the password in `self.solver` until entering all the changes into
                            // the game, during this loop `self.solver.password.len()` is _not_ equal
                            // to the length of the password entered into the game.
                            self.cursor_to(self.solver.password.len())?;

                            self.reset_formatting()?;
                        }
                        // self.tab.type_str(string)?;
                        for grapheme in string.graphemes(true) {
                            self.tab.send_character(grapheme)?;
                        }
                        trace!(
                            "Cursor {}->{}",
                            self.cursor,
                            self.cursor + string.graphemes(true).count()
                        );
                        self.cursor += string.graphemes(true).count();
                        already_appended = true;
                    }
                    Change::Prepend { string, .. } => {
                        if !already_prepended {
                            self.cursor_to(0)?;
                        }

                        self.reset_formatting()?;

                        for grapheme in string.graphemes(true) {
                            self.tab.send_character(grapheme)?;
                        }
                        // self.tab.send_character(string)?;
                        trace!(
                            "Cursor {}->{}",
                            self.cursor,
                            self.cursor + string.graphemes(true).count()
                        );
                        self.cursor += string.graphemes(true).count();
                        already_prepended = true;
                    }
                    Change::Insert { index, string, .. } => {
                        self.cursor_to(*index)?;

                        self.reset_formatting()?;

                        for grapheme in string.graphemes(true) {
                            self.tab.send_character(grapheme)?;
                        }
                        trace!(
                            "Cursor {}->{}",
                            self.cursor,
                            self.cursor + string.graphemes(true).count()
                        );
                        self.cursor += string.graphemes(true).count();
                    }
                    Change::Replace {
                        index,
                        new_grapheme,
                        ..
                    } => {
                        self.cursor_to(*index + 1)?;
                        self.tab
                            .press_key_with_modifiers("ArrowLeft", Some(&[ModifierKey::Shift]))?;
                        self.tab.send_character(new_grapheme)?;
                    }
                    Change::Remove { index, .. } => {
                        // This works because we remove in order of index
                        // So whatever index we're supposed to remove, we're actually missing
                        // `removed_count` indices prior to that due to those removals
                        self.cursor_to(*index + 1 - removed_count)?;
                        self.tab.press_key("Backspace")?;
                        trace!("Cursor {}->{}", self.cursor, self.cursor + 1);
                        self.cursor -= 1;
                        removed_count += 1;
                    }
                }
                self.solver.password.queue_change(change.clone());
            }
            if touched_bold && self.is_bold()? {
                self.toggle_bold()?;
            }
        }
        self.solver.password.commit_changes();

        if self.game_state.highest_rule > Rule::BoldVowels.number() {
            // Don't bother checking until we get to a stage where the game can modify the password
            // underneath us
            self.check_password()?;
        }

        Ok(())
    }

    /// Check if bold formatting is on or off.
    pub fn is_bold(&self) -> Result<bool, DriverError> {
        let buttons = self.tab.find_elements("div.toolbar button")?;
        for button in buttons {
            if button.get_inner_text()?.contains("Bold") {
                let attribs = get_attributes(&button)?;
                if let Some(class) = attribs.get("class") {
                    return Ok(class.contains("is-active"));
                }
            }
        }
        panic!("no bold button found");
    }

    /// Check if italic formatting is on or off.
    pub fn is_italic(&self) -> Result<bool, DriverError> {
        let buttons = self.tab.find_elements("div.toolbar button")?;
        for button in buttons {
            if button.get_inner_text()?.contains("Italic") {
                let attribs = get_attributes(&button)?;
                if let Some(class) = attribs.get("class") {
                    return Ok(class.contains("is-active"));
                }
            }
        }
        panic!("no italic button found");
    }

    /// Toggle bold formatting.
    pub fn toggle_bold(&self) -> Result<(), DriverError> {
        #[cfg(target_os = "macos")]
        let modifier = ModifierKey::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = ModifierKey::Ctrl;
        self.tab.press_key_with_modifiers("B", Some(&[modifier]))?;
        Ok(())
    }

    // Toggle italic formatting.
    pub fn toggle_italic(&self) -> Result<(), DriverError> {
        #[cfg(target_os = "macos")]
        let modifier = ModifierKey::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier = ModifierKey::Ctrl;
        self.tab.press_key_with_modifiers("I", Some(&[modifier]))?;
        Ok(())
    }

    // Select font.
    pub fn select_font(&mut self, font_family: &FontFamily) -> Result<(), DriverError> {
        debug!("Selecting font {:?}", font_family);

        // Tab to font select
        let tabs = if self.game_state.highest_rule >= Rule::DigitFontSize.number() {
            4
        } else {
            3
        };
        for _ in 0..tabs {
            #[cfg(target_os = "windows")]
            winapi::press_and_release_key(winapi::KEYS.get("Tab").unwrap());
            #[cfg(not(target_os = "windows"))]
            self.tab.press_key("Tab")?;
        }
        // Open menu
        self.tab.press_key("Enter")?;
        // Move to top of menu
        for _ in 0..FontFamily::COUNT {
            #[cfg(target_os = "windows")]
            winapi::press_and_release_key(winapi::KEYS.get("NumpadUp").unwrap());
            #[cfg(not(target_os = "windows"))]
            self.tab.press_key("ArrowUp")?;
        }
        // Move down to font
        for _ in 0..font_family.index() {
            #[cfg(target_os = "windows")]
            winapi::press_and_release_key(winapi::KEYS.get("NumpadDown").unwrap());
            #[cfg(not(target_os = "windows"))]
            self.tab.press_key("ArrowDown")?;
        }
        // Select font
        self.tab.press_key("Enter")?;

        Ok(())
    }

    // Select font size.
    pub fn select_font_size(
        &mut self,
        font_size: &FontSize,
        current_font_size: Option<&FontSize>,
    ) -> Result<(), DriverError> {
        debug!("Selecting font size {:?}", font_size);

        // Tab to font size select
        for _ in 0..3 {
            #[cfg(target_os = "windows")]
            winapi::press_and_release_key(winapi::KEYS.get("Tab").unwrap());
            #[cfg(not(target_os = "windows"))]
            self.tab.press_key("Tab")?;
        }
        // Open menu
        self.tab.press_key("Enter")?;
        if let Some(current_font_size) = current_font_size {
            // Move to font size
            if font_size.index() < current_font_size.index() {
                let steps = current_font_size.index() - font_size.index();
                for _ in 0..steps {
                    #[cfg(target_os = "windows")]
                    winapi::press_and_release_key(winapi::KEYS.get("NumpadUp").unwrap());
                    #[cfg(not(target_os = "windows"))]
                    self.tab.press_key("ArrowUp")?;
                }
            } else {
                let steps = font_size.index() - current_font_size.index();
                for _ in 0..steps {
                    #[cfg(target_os = "windows")]
                    winapi::press_and_release_key(winapi::KEYS.get("NumpadDown").unwrap());
                    #[cfg(not(target_os = "windows"))]
                    self.tab.press_key("ArrowDown")?;
                }
            }
        } else {
            // Move to top of menu
            for _ in 0..FontSize::COUNT {
                #[cfg(target_os = "windows")]
                winapi::press_and_release_key(winapi::KEYS.get("NumpadUp").unwrap());
                #[cfg(not(target_os = "windows"))]
                self.tab.press_key("ArrowUp")?;
            }
            // Move down to font size
            for _ in 0..font_size.index() {
                #[cfg(target_os = "windows")]
                winapi::press_and_release_key(winapi::KEYS.get("NumpadDown").unwrap());
                #[cfg(not(target_os = "windows"))]
                self.tab.press_key("ArrowDown")?;
            }
        }
        // Select font size
        self.tab.press_key("Enter")?;

        Ok(())
    }

    /// Reset all available formatting
    fn reset_formatting(&mut self) -> Result<(), DriverError> {
        self.reset_bold()?;
        self.reset_italic()?;
        self.reset_font()?;
        self.reset_font_size()?;

        Ok(())
    }

    /// Reset bold formatting to the default (if bold formatting is available)
    fn reset_bold(&mut self) -> Result<(), DriverError> {
        if self.game_state.highest_rule > Rule::BoldVowels.number() && self.is_bold()? {
            self.toggle_bold()?;
        }
        Ok(())
    }

    /// Reset italic formatting to the default (if italic formatting is available)
    fn reset_italic(&mut self) -> Result<(), DriverError> {
        if self.game_state.highest_rule > Rule::TwiceItalic.number() && self.is_italic()? {
            // Make sure italic is off before we start typing
            self.toggle_italic()?;
        }
        Ok(())
    }

    /// Reset font size to the default (if font size formatting is available)
    fn reset_font_size(&mut self) -> Result<(), DriverError> {
        if self.game_state.highest_rule > Rule::DigitFontSize.number() {
            // Type and delete something to make sure we're focused on password field
            self.tab.send_character("-")?;
            self.tab.press_key("Backspace")?;
            self.select_font_size(&FontSize::default(), None)?;
        }

        Ok(())
    }

    /// Reset font family to the default (if font family formatting is available)
    fn reset_font(&mut self) -> Result<(), DriverError> {
        if self.game_state.highest_rule > Rule::Wingdings.number() {
            // Type and delete something to make sure we're focused on password field
            self.tab.send_character("-")?;
            self.tab.press_key("Backspace")?;
            self.select_font(&FontFamily::default())?;
        }

        Ok(())
    }

    /// Move the cursor to the given index.
    pub fn cursor_to(&mut self, index: usize) -> Result<(), DriverError> {
        trace!("Cursor {}->{}", self.cursor, index);
        if index > self.solver.password.len() {
            panic!("invalid cursor index");
        }

        #[cfg(target_os = "macos")]
        {
            if index > self.cursor {
                let times = index - self.cursor;
                osascript::press_key_code_multiple(
                    *osascript::KEYS.get("RightArrow").unwrap(),
                    times,
                )?;
                self.cursor += times;
            } else if index < self.cursor {
                let times = self.cursor - index;
                osascript::press_key_code_multiple(
                    *osascript::KEYS.get("LeftArrow").unwrap(),
                    times,
                )?;
                self.cursor -= times;
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            while self.cursor < index {
                self.cursor_right(false)?;
            }
            while self.cursor > index {
                self.cursor_left(false)?;
            }
        }

        assert_eq!(self.cursor, index);
        Ok(())
    }

    /// Move the cursor one grapheme to the left.
    /// If `direct` is true, this will just hit the left arrow without updating
    /// or checking our internal cursor state.
    fn cursor_left(&mut self, direct: bool) -> Result<(), DriverError> {
        if !direct && self.cursor == 0 {
            // Cursor is already at the start of the password
            return Ok(());
        }

        trace!("Cursor left");

        #[cfg(target_os = "windows")]
        winapi::press_and_release_key(winapi::KEYS.get("NumpadLeft").unwrap());
        #[cfg(target_os = "macos")]
        osascript::press_key_code(*osascript::KEYS.get("LeftArrow").unwrap())?;
        // #[cfg(not(or(target_os = "window", target_os = "macos")))]
        // self.tab.press_key("ArrowLeft")?;

        if !direct {
            trace!("Cursor {}->{}", self.cursor, self.cursor - 1);
            self.cursor -= 1;
        }
        Ok(())
    }

    /// Move the cursor one grapheme to the right.
    /// If `direct` is true, this will just hit the right arrow without updating
    /// or checking our internal cursor state.
    fn cursor_right(&mut self, direct: bool) -> Result<(), DriverError> {
        if !direct && self.cursor == self.solver.password.len() {
            // Cursor is already at the end of the password
            return Ok(());
        }

        trace!("Cursor right");

        #[cfg(target_os = "windows")]
        winapi::press_and_release_key(winapi::KEYS.get("NumpadRight").unwrap());
        #[cfg(target_os = "macos")]
        osascript::press_key_code(*osascript::KEYS.get("RightArrow").unwrap())?;
        // #[cfg(not(target_os = "windows"))]
        // self.tab.press_key("ArrowRight")?;

        if !direct {
            trace!("Cursor {}->{}", self.cursor, self.cursor + 1);
            self.cursor += 1;
        }
        Ok(())
    }

    /// Sort changes such that they can be entered into the game.
    fn sort_changes_for_entry(changes: &mut [Change]) {
        // Default sort is correct for this
        changes.sort();
    }

    /// Get the password as entered into the game.
    pub fn get_password(&self) -> Result<String, DriverError> {
        let password_box = self.tab.find_element("div.ProseMirror")?;
        Ok(password_box
            .get_inner_text()?
            .trim_end_matches('\n')
            .to_owned())
    }

    /// Get the list of all currently violated rules.
    fn get_violated_rules(&mut self) -> Result<Vec<Rule>, DriverError> {
        std::thread::sleep(RULE_VALIDATION_WAIT_TIME);

        let mut violated_rules = Vec::new();

        let rule_errors = self.tab.find_elements("div.rule-error")?;
        for rule_element in &rule_errors {
            let attribs = get_attributes(rule_element)?;
            let classes = attribs
                .get("class")
                .map(|c| {
                    c.split_ascii_whitespace()
                        .filter(|c| *c != "rule" && *c != "rule-error")
                        .collect::<Vec<&str>>()
                })
                .unwrap_or_else(Vec::new);
            for class in classes {
                let mut rule = serde_plain::from_str::<Rule>(class)?;

                if self.game_state.highest_rule < rule.number() {
                    self.game_state.highest_rule = rule.number();
                }

                // Special cases
                match &mut rule {
                    Rule::Egg => {
                        self.game_state.egg_placed = true;
                    }
                    Rule::Fire => {
                        self.game_state.fire_started = true;
                    }
                    Rule::Hatch => {
                        self.game_state.paul_hatched = true;
                    }
                    Rule::Captcha(captcha) => {
                        let captcha_refresh = self.tab.find_element("img.captcha-refresh")?;

                        // Captcha solution is in the image filename
                        // Re-roll until we avoid a large digit sum
                        let captcha_img = self.tab.find_element("img.captcha-img")?;
                        let mut captcha_answer = get_img_src(&captcha_img)?;
                        let mut rerolled = false;
                        while captcha_answer
                            .chars()
                            .filter(|ch| ch.is_ascii_digit())
                            .fold(0, |sum, ch| sum + ch.to_string().parse::<u32>().unwrap())
                            > 2
                        {
                            debug!("Rerolling captcha...");
                            captcha_refresh.click()?;
                            captcha_answer = get_img_src(&captcha_img)?;
                            rerolled = true;
                        }
                        if rerolled {
                            self.tab.send_character("-")?;
                            self.tab.press_key("Backspace")?;
                        }
                        *captcha = captcha_answer;
                    }
                    Rule::Geo(geo) => {
                        // Lat/long are in the embed URL
                        let geo_iframe = self
                            .tab
                            .find_element("iframe.geo")
                            .expect("failed to get iframe.geo element");
                        let attribs = geo_iframe.get_attributes()?.unwrap();
                        for i in (0..attribs.len()).step_by(2) {
                            if attribs[i] == "src" {
                                let url = &attribs[i + 1];
                                let parts = url.split('!').collect::<Vec<&str>>();
                                geo.lat = NotNan::new(
                                    parts[6].replace("1d", "").parse::<f64>().context(
                                        "failed to parse latitude from Google Maps embed URL",
                                    )?,
                                )
                                .unwrap();
                                geo.long = NotNan::new(
                                    parts[7].replace("2d", "").parse::<f64>().context(
                                        "failed to parse longitude from Google Maps embed URL",
                                    )?,
                                )
                                .unwrap();
                            }
                        }
                    }
                    Rule::Chess(fen) => {
                        // Player to move is in the text
                        let move_div = self.tab.find_element("div.move")?;
                        let text = move_div.get_inner_text()?;
                        let to_move = if text.contains("White") { 'w' } else { 'b' };
                        // FEN notation for the position is in the SVG
                        let chess_img = self.tab.find_element("img.chess-img")?;
                        let attribs = get_attributes(&chess_img)?;
                        let path = attribs.get("src").unwrap();
                        let url = format!("https://neal.fun{}", path);
                        let body = reqwest::blocking::get(url)
                            .context("failed to request chess SVG")?
                            .text()
                            .context("failed to get chess SVG request response body")?;
                        *fen = extract_fen_from_svg(&body, to_move);
                    }
                    Rule::Youtube(duration) => {
                        let rule_text = rule_element.get_inner_text()?;
                        let re = regex!(r"(\d+) minute(?: (\d+) second)?");
                        let captures = re.captures(&rule_text).unwrap();
                        let minutes = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
                        let seconds = captures
                            .get(2)
                            .map(|m| m.as_str().parse::<u32>().unwrap())
                            .unwrap_or_default();
                        *duration = minutes * 60 + seconds;
                    }
                    Rule::Hex(color) => {
                        let color_refresh = self.tab.find_element("img.refresh")?;

                        let color_div = self.tab.find_element("div.rand-color")?;

                        let attribs = get_attributes(&color_div)?;
                        let style = attribs.get("style").unwrap();
                        let mut current_color = extract_color_from_css_style(style);
                        let mut rerolled = false;
                        while current_color
                            .to_hex_string()
                            .chars()
                            .filter(|ch| ch.is_ascii_digit())
                            .fold(0, |sum, ch| sum + ch.to_string().parse::<u32>().unwrap())
                            > 2
                        {
                            debug!("Rerolling color...");
                            color_refresh.click()?;
                            let attribs = get_attributes(&color_div)?;
                            let style = attribs.get("style").unwrap();
                            current_color = extract_color_from_css_style(style);
                            rerolled = true;
                        }
                        if rerolled {
                            self.tab.send_character("-")?;
                            self.tab.press_key("Backspace")?;
                        }
                        *color = current_color;
                    }
                    _ => {}
                }

                violated_rules.push(rule);
            }
        }
        violated_rules.sort();
        violated_rules.reverse();
        Ok(violated_rules)
    }
}

/// Get the src of an img element.
fn get_img_src(element: &headless_chrome::Element) -> Result<String, DriverError> {
    let attribs = get_attributes(element)?;
    let path = attribs.get("src").unwrap();
    for part in path.split('/') {
        if part.contains(".png") {
            return Ok(part.split('.').next().unwrap().to_owned());
        }
    }
    panic!("image has no src")
}

/// Get the attributes of the given element as a HashMap.
fn get_attributes(
    element: &headless_chrome::Element,
) -> Result<HashMap<String, String>, DriverError> {
    let attribs_vec = element.get_attributes().unwrap().unwrap();
    let mut attribs = HashMap::new();
    for i in (0..attribs_vec.len()).step_by(2) {
        attribs.insert(attribs_vec[i].clone(), attribs_vec[i + 1].clone());
    }
    Ok(attribs)
}
