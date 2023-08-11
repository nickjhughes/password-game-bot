use chrono::prelude::*;
use lazy_regex::regex;
use ordered_float::NotNan;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use strum::EnumIter;
use unicode_segmentation::UnicodeSegmentation;

use super::{
    helpers::{
        get_country_from_coordinates, get_moon_phase, get_optimal_move, get_wordle_answer,
        get_youtube_duration, is_prime,
    },
    GameState,
};
use crate::password::{
    format::{FontFamily, FontSize},
    helpers::{get_digits, get_elements, get_roman_numerals, get_youtube_id},
    Password,
};

pub const SPONSORS: [&str; 3] = ["pepsi", "starbucks", "shell"];
pub const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];
pub const AFFIRMATIONS: [&str; 3] = ["i am loved", "i am worthy", "i am enough"];
pub const VOWELS: [&str; 12] = ["a", "e", "i", "o", "u", "y", "A", "E", "I", "O", "U", "Y"];

#[derive(Debug, Clone)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    pub fn emojis(&self) -> Vec<&'static str> {
        match self {
            MoonPhase::New => vec!["🌑", "🌚"],
            MoonPhase::WaxingCrescent => vec!["🌒", "🌘"],
            MoonPhase::FirstQuarter => vec!["🌓", "🌗", "🌛", "🌜"],
            MoonPhase::WaxingGibbous => vec!["🌔", "🌖"],
            MoonPhase::Full => vec!["🌕", "🌝"],
            MoonPhase::WaningGibbous => vec!["🌔", "🌖"],
            MoonPhase::LastQuarter => vec!["🌓", "🌗", "🌛", "🌜"],
            MoonPhase::WaningCrescent => vec!["🌒", "🌘"],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Coords {
    pub lat: NotNan<f64>,
    pub long: NotNan<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn to_hex_string(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
#[serde(rename_all = "kebab-case")]
pub enum Rule {
    /// Rule 1: Your password must be at least 5 characters.
    MinLength,
    /// Rule 2: Your password must include a number.
    Number,
    /// Rule 3: Your password must include an uppercase letter.
    Uppercase,
    /// Rule 4: Your password must include a special character.
    Special,
    /// Rule 5: The digits in your password must add up to 25.
    Digits,
    /// Rule 6: Your password must include a month of the year.
    Month,
    /// Rule 7: Your password must include a roman numeral.
    Roman,
    /// Rule 8: Your password must include one of our sponsors.
    Sponsors,
    /// Rule 9: The roman numerals in your password should multiply to 35.
    RomanMultiply,
    /// Rule 10: Your password must include this CAPTCHA.
    Captcha(#[serde(skip_deserializing)] String),
    /// Rule 11: Your password must include today's Wordle answer.
    Wordle,
    /// Rule 12: Your password must include a two letter symbol from the periodic table.
    PeriodicTable,
    /// Rule 13: Your password must include the current phase of the moon as an emoji.
    MoonPhase,
    /// Rule 14: Your password must include the name of this country.
    Geo(#[serde(skip_deserializing)] Coords),
    /// Rule 15: Your password must include a leap year.
    LeapYear,
    /// Rule 16: Your password must include the best move in algebraic chess notation.
    Chess(#[serde(skip_deserializing)] String),
    /// Rule 17: 🥚 This my chicken Paul. He hasn’t hatched yet. Please put him in your password and keep him safe.
    Egg,
    /// Rule 18: The elements in your password must have atomic numbers that add up to 200.
    AtomicNumber,
    /// Rule 19: All the vowels in your password must be bolded.
    BoldVowels,
    /// Rule 20:Oh no! Your password is on fire 🔥. Quick, put it out!
    Fire,
    /// Rule 21: Your password is not strong enough🏋️‍♂️.
    Strength,
    /// Rule 22: Your password must contain one of the following affirmations: I am loved|I am worthy|I am enough
    Affirmation,
    /// Rule 23: Paul has hatched🐔! Please don’t forget to feed him. He eats three 🐛 every minute.
    Hatch,
    /// Rule 24: Your password must include the URL of a YouTube video of this exact length.
    Youtube(#[serde(skip_deserializing)] u32),
    /// Rule 25: A sacrifice must be made. Pick 2 letters that you will no longer be able to use.
    #[serde(rename = "sacrafice")]
    Sacrifice,
    /// Rule 26: Your password must contain twice as many italic characters as bold.
    TwiceItalic,
    /// Rule 27: At least 30% of your password must be in the Wingdings font.
    Wingdings,
    /// Rule 28: Your password must include this color in hex.
    Hex(#[serde(skip_deserializing)] Color),
    /// Rule 29: All roman numerals must be in Times New Roman.
    TimesNewRoman,
    /// Rule 30: The font size of every digit must be equal to its square.
    DigitFontSize,
    /// Rule 31: Every instance of the same letter must have a different font size.
    LetterFontSize,
    /// Rule 32: Your password must include the length of your password.
    IncludeLength,
    /// Rule 33: The length of your password must be a prime number.
    PrimeLength,
    /// Rule 34: Uhhh let's skip this one.
    Skip,
    /// Rule 35: Your password must include the current time.
    Time,
    /// Rule 36: Is this your final password?
    Final,
}

impl Rule {
    /// The rule's number (starting at 1).
    pub fn number(&self) -> usize {
        match self {
            Rule::MinLength => 1,
            Rule::Number => 2,
            Rule::Uppercase => 3,
            Rule::Special => 4,
            Rule::Digits => 5,
            Rule::Month => 6,
            Rule::Roman => 7,
            Rule::Sponsors => 8,
            Rule::RomanMultiply => 9,
            Rule::Captcha(_) => 10,
            Rule::Wordle => 11,
            Rule::PeriodicTable => 12,
            Rule::MoonPhase => 13,
            Rule::Geo { .. } => 14,
            Rule::LeapYear => 15,
            Rule::Chess { .. } => 16,
            Rule::Egg => 17,
            Rule::AtomicNumber => 18,
            Rule::BoldVowels => 19,
            Rule::Fire => 20,
            Rule::Strength => 21,
            Rule::Affirmation => 22,
            Rule::Hatch => 23,
            Rule::Youtube { .. } => 24,
            Rule::Sacrifice => 25,
            Rule::TwiceItalic => 26,
            Rule::Wingdings => 27,
            Rule::Hex(_) => 28,
            Rule::TimesNewRoman => 29,
            Rule::DigitFontSize => 30,
            Rule::LetterFontSize => 31,
            Rule::IncludeLength => 32,
            Rule::PrimeLength => 33,
            Rule::Skip => 34,
            Rule::Time => 35,
            Rule::Final => 36,
        }
    }

    /// Does the given password satisfy this rule at the given time?
    pub fn validate_at_time(
        &self,
        password: &Password,
        game_state: &GameState,
        datetime: &DateTime<Local>,
    ) -> bool {
        match self {
            Rule::MinLength => password.as_str().graphemes(true).count() >= 5,
            Rule::Number => password.as_str().chars().any(|c| c.is_ascii_digit()),
            Rule::Uppercase => password.as_str().chars().any(|c| c.is_ascii_uppercase()),
            Rule::Special => password
                .as_str()
                .chars()
                .any(|c| !c.is_ascii_alphanumeric()),
            Rule::Digits => {
                get_digits(password.as_str())
                    .iter()
                    .map(|(d, _)| d)
                    .copied()
                    .reduce(|sum, d| sum + d)
                    .unwrap_or_default()
                    == 25
            }
            Rule::Month => {
                let lowercase_password = password.as_str().to_lowercase();
                MONTHS.iter().any(|m| lowercase_password.contains(m))
            }
            Rule::Roman => !get_roman_numerals(password.as_str()).is_empty(),
            Rule::Sponsors => {
                let lowercase_password = password.as_str().to_lowercase();
                SPONSORS.iter().any(|m| lowercase_password.contains(m))
            }
            Rule::RomanMultiply => {
                get_roman_numerals(password.as_str())
                    .iter()
                    .map(|(d, _, _)| d)
                    .copied()
                    .reduce(|a, b| a * b)
                    .unwrap_or_default()
                    == 35
            }
            Rule::Captcha(captcha) => password.as_str().contains(captcha),
            Rule::Wordle => {
                let wordle_answer = &get_wordle_answer(datetime.date_naive());
                let lowercase_password = password.as_str().to_lowercase();
                lowercase_password.contains(wordle_answer)
            }
            Rule::PeriodicTable => get_elements(password.as_str())
                .iter()
                .any(|(e, _)| e.symbol.len() == 2),
            Rule::MoonPhase => {
                let valid_emojis = get_moon_phase(*datetime).emojis();
                let mut found = false;
                for grapheme in password.as_str().graphemes(true) {
                    if valid_emojis.iter().any(|e| *e == grapheme) {
                        found = true;
                    }
                }
                found
            }
            Rule::Geo(geo) => {
                let country_name = get_country_from_coordinates(geo.lat, geo.long);
                let lowercase_password = password.as_str().to_lowercase();
                lowercase_password.contains(&country_name)
            }
            Rule::LeapYear => {
                let year_regex = regex!(r"(\d+)");
                let mut years = Vec::new();
                for (_, [year]) in year_regex
                    .captures_iter(password.as_str())
                    .map(|c| c.extract())
                {
                    years.push(year.parse::<u64>().unwrap());
                }
                years
                    .iter()
                    .any(|y| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0))
            }
            Rule::Chess(fen) => {
                let solution = get_optimal_move(fen.to_owned());
                password.as_str().contains(&solution)
            }
            Rule::Egg => {
                if game_state.paul_hatched {
                    password.as_str().graphemes(true).any(|g| g == "🐔")
                } else if game_state.egg_placed {
                    password.as_str().graphemes(true).any(|g| g == "🥚")
                } else {
                    true
                }
            }
            Rule::AtomicNumber => {
                get_elements(password.as_str())
                    .iter()
                    .map(|(e, _)| e.atomic_number)
                    .reduce(|sum, n| sum + n)
                    .unwrap_or_default()
                    == 200
            }
            Rule::BoldVowels => password
                .as_str()
                .graphemes(true)
                .enumerate()
                .filter(|(_, g)| VOWELS.contains(g))
                .all(|(i, _)| password.formatting()[i].bold),
            Rule::Fire => {
                game_state.fire_started && !password.as_str().graphemes(true).any(|g| g == "🔥")
            }
            Rule::Strength => {
                password
                    .as_str()
                    .graphemes(true)
                    .filter(|g| *g == "🏋️‍♂️")
                    .count()
                    >= 3
            }
            Rule::Affirmation => {
                let lowercase_password = password.as_str().to_lowercase();
                AFFIRMATIONS.iter().any(|m| {
                    lowercase_password.contains(m)
                        || lowercase_password.contains(&m.replace(' ', ""))
                })
            }
            Rule::Hatch => {
                if !game_state.paul_hatched {
                    true
                } else {
                    game_state.paul_eating || password.as_str().graphemes(true).any(|g| g == "🐛")
                }
            }
            Rule::Youtube(seconds) => {
                if let Some(video_id) = get_youtube_id(password.as_str()) {
                    let duration = get_youtube_duration(video_id);
                    duration <= *seconds + 1 && duration >= *seconds - 1
                } else {
                    false
                }
            }
            Rule::Sacrifice => {
                if game_state.sacrificed_letters.len() != 2 {
                    // First, ensure the user has chosen 2 letters
                    false
                } else {
                    // And if so, make sure those letters don't appear in the password
                    let lowercase_password = password.as_str().to_lowercase();
                    let mut found = false;
                    for letter in &game_state.sacrificed_letters {
                        if lowercase_password.contains(*letter) {
                            found = true;
                        }
                    }
                    !found
                }
            }
            Rule::TwiceItalic => {
                let italic_count = password.formatting().iter().filter(|f| f.italic).count();
                let bold_count = password.formatting().iter().filter(|f| f.bold).count();
                italic_count as f32 >= 2.0 * bold_count as f32
            }
            Rule::Wingdings => {
                let wingdings_count = password
                    .formatting()
                    .iter()
                    .filter(|f| f.font_family == FontFamily::Wingdings)
                    .count();
                wingdings_count as f32 / password.len() as f32 >= 0.3
            }
            Rule::Hex(Color { r, g, b }) => {
                let hex = format!("{:02x}{:02x}{:02x}", r, g, b);
                let lowercase_password = password.as_str().to_lowercase();
                lowercase_password.contains(&hex)
            }
            Rule::TimesNewRoman => {
                let formatting = password.formatting();
                get_roman_numerals(password.as_str())
                    .iter()
                    .all(|(_, index, length)| {
                        (0..*length)
                            .all(|i| formatting[index + i].font_family == FontFamily::TimesNewRoman)
                    })
            }
            Rule::DigitFontSize => {
                let formatting = password.formatting();
                get_digits(password.as_str())
                    .iter()
                    .all(|(d, i)| formatting[*i].font_size == FontSize::try_from(d * d).unwrap())
            }
            Rule::LetterFontSize => {
                let mut letter_font_sizes: HashMap<char, HashSet<FontSize>> = HashMap::new();
                let mut valid = true;
                for (i, grapheme) in password.as_str().graphemes(true).enumerate() {
                    if grapheme.len() != 1 {
                        continue;
                    }
                    let ch = grapheme.chars().next().unwrap().to_ascii_lowercase();
                    if !ch.is_ascii_alphabetic() {
                        continue;
                    }
                    let font_size = &password.formatting()[i].font_size;
                    let font_sizes = letter_font_sizes.entry(ch).or_default();
                    let is_new = font_sizes.insert(font_size.clone());
                    if !is_new {
                        valid = false;
                        break;
                    }
                }
                valid
            }
            Rule::IncludeLength => {
                let length = password.as_str().graphemes(true).count();
                password.as_str().contains(&length.to_string())
            }
            Rule::PrimeLength => {
                let length = password.as_str().graphemes(true).count();
                is_prime(length)
            }
            Rule::Skip => true,
            Rule::Time => {
                let time_string = datetime.format("%l:%M").to_string().trim().to_owned();
                password.as_str().contains(&time_string)
            }
            Rule::Final => true,
        }
    }

    /// Does the given password satisfy this rule at the current time?
    pub fn validate(&self, password: &Password, game_state: &GameState) -> bool {
        self.validate_at_time(password, game_state, &Local::now())
    }
}
