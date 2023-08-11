use chrono::prelude::*;
use lazy_static::lazy_static;
use log::{debug, info};
use numerals::roman::Roman;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    game::{
        helpers::{
            get_country_from_coordinates, get_moon_phase, get_optimal_move, get_wordle_answer,
            is_prime,
        },
        GameState,
        {
            rule::{AFFIRMATIONS, MONTHS, SPONSORS, VOWELS},
            Rule,
        },
    },
    password::{
        helpers::{get_digits, get_elements, get_letters, get_roman_numerals},
        Change, MutablePassword,
        {
            format::{FontFamily, FontSize, FontSizeIter},
            FormatChange,
        },
    },
};

#[cfg(test)]
mod tests;

#[derive(Deserialize)]
struct Video {
    id: &'static str,
    duration: u32,
}

lazy_static! {
    pub static ref VIDEOS: HashMap<u32, &'static str> = {
        let videos: Vec<Video> =
            serde_json::from_str(include_str!("../youtube/videos.json")).unwrap();

        let mut m = HashMap::new();
        for video in &videos {
            m.insert(video.duration, video.id);
        }
        m
    };
}

#[derive(Default)]
pub struct Solver {
    /// The current password as entered into the game.
    pub password: MutablePassword,
    /// The rules which the current password violates.
    pub violated_rules: Vec<Rule>,
    /// Letters we've chosen to sacrifice.
    pub sacrificed_letters: Vec<char>,
    /// Grapheme index and length of the password length string.
    pub length_string: Option<InnerString>,
    /// Grapheme index and length of the time string.
    pub time_string: Option<InnerString>,
    /// Goal password length we've chosen.
    pub goal_length: Option<usize>,
}

/// Essentially a string slice in the password.
#[derive(Debug)]
pub struct InnerString {
    /// Grapheme index of the first grapheme in the string.
    index: usize,
    /// Length of the string in grapheme clusters.
    length: usize,
}

impl InnerString {
    pub fn new(index: usize, length: usize) -> Self {
        InnerString { index, length }
    }
}

impl Solver {
    /// Produce a change (or series of changes) which solves the given rule.
    /// If no solution can be found, return None.
    pub fn solve_rule(
        &mut self,
        rule: &Rule,
        game_state: &GameState,
        bugs: usize,
    ) -> Option<Vec<Change>> {
        debug!("Solving rule {:?}", rule);

        let mut changes = Vec::new();

        match rule {
            Rule::Wingdings | Rule::IncludeLength | Rule::PrimeLength => {
                // Ignore these, as the password length is messed with by the "keep bugs for Paul
                // outside the password" thing the WebDriver does.
            }
            _ => {
                if rule.validate(self.password.raw_password(), game_state) {
                    return Some(changes);
                }
            }
        }

        match rule {
            Rule::MinLength => {
                let to_add = 5 - self.password.len();
                changes.push(Change::Append {
                    protected: false,
                    string: "z".repeat(to_add),
                });
            }
            Rule::Number => {
                changes.push(Change::Append {
                    protected: false,
                    string: "9".into(),
                });
            }
            Rule::Uppercase => {
                changes.push(Change::Append {
                    protected: false,
                    string: "Z".into(),
                });
            }
            Rule::Special => {
                changes.push(Change::Append {
                    protected: false,
                    string: "!".into(),
                });
            }
            Rule::Digits => {
                let digits = {
                    let mut d = get_digits(self.password.as_str());
                    // For the sum, we don't care about the digit 0
                    d.retain(|(d, _)| *d > 0);
                    d
                };
                let mut digits_sum = digits
                    .iter()
                    .map(|(d, _)| d)
                    .copied()
                    .reduce(|sum, d| sum + d)
                    .unwrap_or_default();
                if digits_sum > 25 {
                    // Need to remove or reduce digits
                    let mut unprotected_digits = digits
                        .iter()
                        .filter(|(_, i)| !self.password.protected_graphemes()[*i])
                        .collect::<Vec<_>>();

                    let unprotected_sum = unprotected_digits
                        .iter()
                        .map(|(d, _)| d)
                        .copied()
                        .reduce(|sum, d| sum + d)
                        .unwrap_or_default();
                    if digits_sum - unprotected_sum > 25 {
                        // The digits in strings which must appear in the password
                        // sum to more than 25 :(
                        // There are solutions here, but for now, just bail
                        return None;
                    }

                    // We have a number of digits, and we need to reduce their sum by `to_reduce`
                    let mut to_reduce = digits_sum - 25;
                    unprotected_digits.sort_by(|a, b| a.0.cmp(&b.0).reverse());

                    // First remove digits to reduce the sum, largest first
                    let mut removed_digits = Vec::new();
                    for (d, i) in &unprotected_digits {
                        if *d <= to_reduce {
                            changes.push(Change::Remove {
                                index: *i,
                                ignore_protection: false,
                            });
                            removed_digits.push(i);
                            to_reduce -= d;
                            if to_reduce == 0 {
                                break;
                            }
                        }
                    }
                    unprotected_digits.retain(|(_, i)| !removed_digits.contains(&i));

                    // If the sum is still too big, reduce an arbitrary digit appropriately
                    if to_reduce > 0 {
                        let (digit, i) = unprotected_digits[0];
                        let new_digit = digit - to_reduce;
                        changes.push(Change::Replace {
                            index: *i,
                            new_grapheme: new_digit.to_string(),
                            ignore_protection: false,
                        });
                    }
                } else {
                    // Just add the largest digits possible until we hit 25
                    let mut append = String::new();
                    while digits_sum < 25 {
                        let next_digit = (25 - digits_sum).min(9);
                        append.push_str(&next_digit.to_string());
                        digits_sum += next_digit;
                    }
                    changes.push(Change::Append {
                        protected: false,
                        string: append,
                    });
                }
            }
            Rule::Month => {
                // let month = "may";
                let mut rng = thread_rng();
                let month = MONTHS.choose(&mut rng).unwrap();
                changes.push(Change::Append {
                    protected: true,
                    string: month.to_string(),
                });
            }
            Rule::Roman => {
                changes.push(Change::Append {
                    protected: false,
                    string: "XXXV".into(),
                });
            }
            Rule::Sponsors => {
                // let sponsor = "pepsi";
                let mut rng = thread_rng();
                let sponsor = SPONSORS.choose(&mut rng).unwrap();
                changes.push(Change::Append {
                    protected: true,
                    string: sponsor.to_string(),
                });
            }
            Rule::RomanMultiply => {
                // The factors of 35 are 1, 5, 7, 35
                // The password must only contain, in addition to an unlimited number of "I":
                //  - XXXV, or
                //  - V and VII
                let numbers = get_roman_numerals(self.password.as_str());

                let mut number_counts: HashMap<u64, usize> = HashMap::new();
                for (number, _, _) in &numbers {
                    *number_counts.entry(*number).or_default() += 1;
                }
                let mut goal_numbers = if number_counts.contains_key(&35) {
                    // Aim for 35 only
                    vec![35]
                } else {
                    // Aim for 5 and 7
                    vec![5, 7]
                };

                for (number, start, length) in &numbers {
                    if *number == 1 {
                        // Leave it
                        continue;
                    }
                    if goal_numbers.contains(number) {
                        // Leave it, but remove from goals
                        goal_numbers.remove(goal_numbers.iter().position(|x| x == number).unwrap());
                    } else {
                        // Remove it
                        for i in 0..*length {
                            if self.password.protected_graphemes()[*start + i] {
                                // A numeral we can't have is in a protected range :(
                                return None;
                            }
                            changes.push(Change::Remove {
                                index: *start + i,
                                ignore_protection: false,
                            });
                        }
                    }
                }

                // If there are remaining goal numbers, append them
                // (with a space to ensure they don't combine with a roman numeral already
                // at the end of the password)
                // TODO: Only append that space if it's actually necessary
                for goal in &goal_numbers {
                    let numeral = format!(" {:X}", Roman::from(*goal as i16));
                    changes.push(Change::Append {
                        protected: false,
                        string: numeral,
                    });
                }
            }
            Rule::Captcha(captcha) => {
                changes.push(Change::Append {
                    protected: true,
                    string: captcha.clone(),
                });
            }
            Rule::Wordle => {
                let wordle = get_wordle_answer(Local::now().date_naive());
                changes.push(Change::Append {
                    protected: true,
                    string: wordle,
                });
            }
            Rule::PeriodicTable => {
                // Otherwise just add any element
                changes.push(Change::Append {
                    protected: true,
                    string: "He".into(),
                });
            }
            Rule::MoonPhase => {
                changes.push(Change::Append {
                    protected: true,
                    string: get_moon_phase(Local::now())
                        .emojis()
                        .first()
                        .unwrap()
                        .to_string(),
                });
            }
            Rule::Geo(geo) => {
                let country_name = get_country_from_coordinates(geo.lat, geo.long);
                changes.push(Change::Append {
                    protected: true,
                    string: country_name.replace(' ', ""),
                });
            }
            Rule::LeapYear => {
                // 0 is a valid leap year, and doesn't affect the digit sum rule
                changes.push(Change::Append {
                    protected: true,
                    string: "0".into(),
                })
            }
            Rule::Chess(fen) => {
                let optimal_move = get_optimal_move(fen.to_owned());
                changes.push(Change::Append {
                    protected: true,
                    string: optimal_move,
                })
            }
            Rule::Egg => changes.push(Change::Prepend {
                protected: true,
                string: "🥚".into(),
            }),
            Rule::AtomicNumber => {
                let elements = get_elements(self.password.as_str());
                let mut sum = elements
                    .iter()
                    .map(|(e, _)| e.atomic_number)
                    .reduce(|sum, d| sum + d)
                    .unwrap_or_default();

                let nonroman_elements = periodic_table::periodic_table()
                    .iter()
                    .filter(|e| get_roman_numerals(e.symbol).is_empty())
                    .collect::<Vec<_>>();

                if sum > 200 {
                    // See which elements we can remove
                    let elements = get_elements(self.password.as_str());
                    let mut unprotected_elements = Vec::new();
                    for (element, index) in &elements {
                        if !self.password.protected_graphemes()[*index]
                            && (element.symbol.len() == 1
                                || !self.password.protected_graphemes()[*index + 1])
                        {
                            unprotected_elements.push((element, index));
                        }
                    }
                    unprotected_elements.sort_by(|a, b| a.0.atomic_number.cmp(&b.0.atomic_number));

                    // Remove unprotected elements until we get <= 200, largest first
                    // Also avoid touching roman numeral element symbols
                    for (element, index) in unprotected_elements
                        .iter()
                        .filter(|(e, _)| nonroman_elements.iter().any(|e2| e2.symbol == e.symbol))
                        .rev()
                    {
                        if sum <= 200 {
                            break;
                        }
                        changes.push(Change::Remove {
                            index: **index,
                            ignore_protection: false,
                        });
                        if element.symbol.len() == 2 {
                            changes.push(Change::Remove {
                                index: *index + 1,
                                ignore_protection: false,
                            });
                        }
                        sum -= element.atomic_number;
                    }

                    // If now under < 200, the next part will take care of it
                    // Otherwise, bail
                    if sum > 200 {
                        debug!("Atomic number sum is > 200 and we can't remove any more :(");
                        return None;
                    }
                }

                let mut to_add = 200 - sum;
                while to_add > 0 {
                    // Add the largest non-roman-numeral element that fits
                    let element = nonroman_elements
                        .iter()
                        .filter(|e| e.atomic_number <= to_add)
                        .last()
                        .unwrap();
                    changes.push(Change::Append {
                        string: element.symbol.to_owned(),
                        protected: false,
                    });
                    to_add -= element.atomic_number;
                }
            }
            Rule::BoldVowels => {
                for (index, grapheme) in self.password.as_str().graphemes(true).enumerate() {
                    if VOWELS.contains(&grapheme)
                        && !self.password.raw_password().formatting()[index].bold
                    {
                        changes.push(Change::Format {
                            index,
                            format_change: FormatChange::BoldOn,
                        });
                    }
                }
            }
            Rule::Fire => {
                for (index, grapheme) in self.password.as_str().graphemes(true).enumerate() {
                    if grapheme == "🔥" {
                        changes.push(Change::Remove {
                            index,
                            ignore_protection: false,
                        });
                    }
                }
            }
            Rule::Strength => {
                changes.push(Change::Append {
                    string: "🏋️‍♂️🏋️‍♂️🏋️‍♂️".into(),
                    protected: true,
                });
            }
            Rule::Affirmation => {
                let mut rng = thread_rng();
                let affirmation = AFFIRMATIONS.choose(&mut rng).unwrap();
                changes.push(Change::Append {
                    protected: true,
                    string: affirmation.replace(' ', ""),
                });
            }
            Rule::Hatch => {
                // We can insert up to 8 🐛's before Paul is overfed
                changes.push(Change::Append {
                    string: "🐛🐛🐛🐛🐛🐛🐛🐛".into(),
                    protected: false,
                });
            }
            Rule::Youtube(seconds) => {
                let video_id = VIDEOS.get(seconds).expect("no video of length");
                let url = format!("youtu.be/{}", video_id);
                changes.push(Change::Append {
                    string: url,
                    protected: true,
                });
            }
            Rule::Sacrifice => {
                if self.sacrificed_letters.is_empty() {
                    // Choose letters to sacrifice

                    // First find all absent and unprotected letters
                    // Start at g to immediately exclude hex digits (to avoid making the hex color
                    //   rule harder to satisfy)
                    // Also immediately exclude roman numerals V and X
                    let mut absent_letters = ('g'..='z').collect::<HashSet<char>>();
                    let mut unprotected_letters = ('g'..='z').collect::<HashSet<char>>();
                    absent_letters.remove(&'v');
                    absent_letters.remove(&'x');
                    unprotected_letters.remove(&'v');
                    unprotected_letters.remove(&'x');
                    for (ch, index) in get_letters(self.password.as_str()) {
                        let ch = ch.to_ascii_lowercase();
                        absent_letters.remove(&ch);
                        if self.password.protected_graphemes()[index] {
                            unprotected_letters.remove(&ch);
                        }
                    }
                    if absent_letters.union(&unprotected_letters).count() < 2 {
                        // Can't find 2 letters to sacrifice
                        return None;
                    }
                    while !absent_letters.is_empty() && self.sacrificed_letters.len() < 2 {
                        #[allow(clippy::clone_on_copy)]
                        let letter = absent_letters.iter().next().unwrap().clone();
                        absent_letters.remove(&letter);
                        unprotected_letters.remove(&letter);
                        self.sacrificed_letters.push(letter);
                    }
                    while !unprotected_letters.is_empty() && self.sacrificed_letters.len() < 2 {
                        #[allow(clippy::clone_on_copy)]
                        let letter = unprotected_letters.iter().next().unwrap().clone();
                        unprotected_letters.remove(&letter);
                        self.sacrificed_letters.push(letter);
                    }
                    if self.sacrificed_letters.len() < 2 {
                        // Failed :(
                        return None;
                    }

                    debug!("Sacrificing {:?}", self.sacrificed_letters);
                }

                // Remove sacrificed letters
                debug_assert_eq!(self.sacrificed_letters.len(), 2);
                for (ch, index) in get_letters(self.password.as_str()) {
                    let ch = ch.to_ascii_lowercase();
                    if self.sacrificed_letters.contains(&ch) {
                        if self.password.protected_graphemes()[index] {
                            panic!("We sacrificed a protected letter");
                        }
                        changes.push(Change::Remove {
                            index,
                            ignore_protection: false,
                        });
                    }
                }
            }
            Rule::TwiceItalic => {
                let formatting = self.password.raw_password().formatting();
                let bold_count = formatting.iter().filter(|f| f.bold).count();
                let italic_count = formatting.iter().filter(|f| f.italic).count();
                let needed_italic = 2 * bold_count - italic_count;

                let mut i = 0;
                while changes.len() < needed_italic {
                    if i == formatting.len() {
                        return None;
                    }
                    if !formatting[i].italic {
                        changes.push(Change::Format {
                            index: i,
                            format_change: FormatChange::ItalicOn,
                        });
                    }
                    i += 1;
                }
            }
            Rule::Wingdings => {
                let numerals = get_roman_numerals(self.password.as_str());
                let mut roman_numeral_indices = Vec::new();
                for (_, i, len) in &numerals {
                    for j in *i..*i + *len {
                        roman_numeral_indices.push(j);
                    }
                }

                let formatting = self.password.raw_password().formatting();
                let wingdings_count = formatting
                    .iter()
                    .filter(|f| f.font_family == FontFamily::Wingdings)
                    .count();
                // The extra 8 accounts for Paul's food that we store at the end of the password,
                // rather than _in_ the password, in the web driver
                let needed_wingdings =
                    (0.3 * (self.password.len() + 8) as f32).ceil() as usize - wingdings_count;
                debug!(
                    "Current wingdings percent <= {}",
                    wingdings_count as f32 / (self.password.len() + 8) as f32
                );

                let mut i = 0;
                while changes.len() < needed_wingdings {
                    if i == formatting.len() {
                        return None;
                    }
                    // Don't change font of roman numerals, they must be times new roman
                    if roman_numeral_indices.contains(&i) {
                        i += 1;
                        continue;
                    }

                    if formatting[i].font_family != FontFamily::Wingdings {
                        changes.push(Change::Format {
                            index: i,
                            format_change: FormatChange::FontFamily(FontFamily::Wingdings),
                        });
                    }
                    i += 1;
                }
            }
            Rule::Hex(color) => {
                changes.push(Change::Append {
                    string: color.to_hex_string(),
                    protected: true,
                });
            }
            Rule::TimesNewRoman => {
                let formatting = self.password.raw_password().formatting();
                let numerals = get_roman_numerals(self.password.as_str());
                for (_, i, len) in &numerals {
                    for (j, format) in formatting.iter().enumerate().skip(*i).take(*len) {
                        if format.font_family != FontFamily::TimesNewRoman {
                            changes.push(Change::Format {
                                index: j,
                                format_change: FormatChange::FontFamily(FontFamily::TimesNewRoman),
                            });
                        }
                    }
                }
            }
            Rule::DigitFontSize => {
                let formatting = self.password.raw_password().formatting();
                let digits = get_digits(self.password.as_str());
                for (digit, i) in &digits {
                    let square_font_size = FontSize::try_from(digit * digit).unwrap();
                    if formatting[*i].font_size != square_font_size {
                        changes.push(Change::Format {
                            index: *i,
                            format_change: FormatChange::FontSize(square_font_size),
                        });
                    }
                }
            }
            Rule::LetterFontSize => {
                // For all letters, start at size 28 (the default) and work up one size for each
                // instance of that letter found
                let current_formatting = self.password.raw_password().formatting();
                let mut letter_sizes: HashMap<char, FontSizeIter> = HashMap::new();
                for (letter, index) in get_letters(self.password.as_str()) {
                    let letter = letter.to_ascii_lowercase();
                    let size_iter = letter_sizes.entry(letter).or_insert(FontSize::iter());
                    if let Some(font_size) = size_iter.next() {
                        if current_formatting[index].font_size != font_size {
                            changes.push(Change::Format {
                                index,
                                format_change: FormatChange::FontSize(font_size),
                            });
                        }
                    } else {
                        // We've run out of font sizes for this letter :(
                        return None;
                    }
                }
            }
            Rule::IncludeLength => {
                if self.length_string.is_none() {
                    // Pick a length we want to aim for
                    let mut padding = 0;
                    self.goal_length = {
                        // 3 for length string, 5 for time string
                        let mut l = self.password.len() + 3 + 5 + bugs;
                        // TODO: Maybe try to minimize the digit sum of `l` here too
                        while l < 100 || !is_prime(l) {
                            padding += 1;
                            l += 1;
                        }
                        Some(l)
                    };
                    info!(
                        "Password length will be {}",
                        self.goal_length.as_ref().unwrap()
                    );

                    // Append the length string to the end
                    let length_string = self.goal_length.as_ref().unwrap().to_string();
                    let length_length = length_string.len();
                    assert_eq!(length_length, 3);
                    self.length_string = Some(InnerString::new(self.password.len(), length_length));
                    changes.push(Change::Append {
                        string: length_string,
                        protected: true,
                    });

                    // Add in time string
                    let time = Local::now().format("%l:%M").to_string().trim().to_owned();
                    changes.push(Change::Append {
                        string: time.clone(),
                        protected: true,
                    });
                    self.time_string = Some(InnerString::new(
                        self.password.len() + length_length,
                        time.len(),
                    ));

                    // Add padding
                    changes.push(Change::Append {
                        string: "-".repeat(padding),
                        protected: false,
                    });

                    // At this point, the password may or may not be `goal_length` in length, but:
                    // - If it's too long, Paul will eat bugs until it's right
                    // - If it's too short, we'll eventually feed Paul more bugs until it's right
                }
            }
            Rule::PrimeLength => {
                // We don't need to do anything here, because in solving `IncludeLength`, we
                // specified a goal length that is prime.
            }
            Rule::Skip => {}
            Rule::Time => {
                let time = Local::now().format("%l:%M").to_string().trim().to_owned();
                if let Some(InnerString { index, length }) = self.time_string {
                    if length != time.len() {
                        todo!("length of time string changed");
                    }
                    for (i, ch) in time.chars().enumerate() {
                        changes.push(Change::Replace {
                            index: index + i,
                            new_grapheme: ch.to_string(),
                            ignore_protection: true,
                        });
                    }
                } else {
                    // Just append time to the end
                    changes.push(Change::Append {
                        string: time.clone(),
                        protected: true,
                    });
                    self.time_string = Some(InnerString::new(self.password.len(), time.len()));
                }
            }
            Rule::Final => {}
        }

        // Update location of length string if necessary
        if let Some(InnerString {
            index: length_string_index,
            ..
        }) = self.length_string.as_mut()
        {
            for change in changes.iter() {
                match change {
                    Change::Insert { index, string, .. } => {
                        if index < length_string_index {
                            *length_string_index += string.graphemes(true).count();
                        }
                    }
                    Change::Prepend { string, .. } => {
                        *length_string_index += string.graphemes(true).count();
                    }
                    Change::Remove { index, .. } => {
                        if index < length_string_index {
                            *length_string_index -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update location of time string if necessary
        if let Some(InnerString {
            index: time_string_index,
            ..
        }) = self.time_string.as_mut()
        {
            for change in changes.iter() {
                match change {
                    Change::Insert { index, string, .. } => {
                        if index < time_string_index {
                            *time_string_index += string.graphemes(true).count();
                        }
                    }
                    Change::Prepend { string, .. } => {
                        *time_string_index += string.graphemes(true).count();
                    }
                    Change::Remove { index, .. } => {
                        if index < time_string_index {
                            *time_string_index -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        Some(changes)
    }

    /// Solve for the given rule and updates the password in one go.
    /// Panics if a solution can't be found.
    #[cfg(test)]
    pub fn solve_rule_and_commit(&mut self, rule: &Rule, game_state: &GameState) {
        let changes = self
            .solve_rule(rule, game_state, 0)
            .expect("could not find a solution");
        for change in changes {
            self.password.queue_change(change);
        }
        self.password.commit_changes();
    }

    /// Generate the best starting password we can via a series of changes to the empty password.
    pub fn starting_password(&self) -> Vec<Change> {
        vec![
            Change::Append {
                protected: true,
                string: "🥚0mayXXXVshell".into(),
            },
            Change::Append {
                protected: true,
                string: get_moon_phase(Local::now())
                    .emojis()
                    .first()
                    .unwrap()
                    .to_string(),
            },
            Change::Append {
                protected: false,
                string: "He997".into(),
            },
        ]
    }
}
