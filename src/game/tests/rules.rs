use chrono::prelude::*;
use ordered_float::NotNan;

use super::super::{
    rule::{Color, Coords},
    GameState, Rule,
};
use crate::password::{
    format::{FontFamily, FontSize},
    FormatChange, Password,
};

#[test]
fn rule_min_length() {
    let game_state = GameState::default();

    assert!(Rule::MinLength.validate(&Password::from_str("12345"), &game_state));
    assert!(Rule::MinLength.validate(&Password::from_str("123456789"), &game_state));

    // Length < 5 (but byte length > 5)
    assert!(!Rule::MinLength.validate(&Password::from_str("😀😀"), &game_state));

    // Length < 5 (but == 5 codepoints)
    assert!(!Rule::MinLength.validate(&Password::from_str("🏋️‍♂️"), &game_state));
}

#[test]
fn rule_number() {
    let game_state = GameState::default();

    for i in 0..=9 {
        assert!(Rule::Number.validate(&Password::from_str(&format!("{}", i)), &game_state));
    }

    assert!(!Rule::Number.validate(&Password::from_str("one"), &game_state));
}

#[test]
fn rule_uppercase() {
    let game_state = GameState::default();

    assert!(Rule::Uppercase.validate(&Password::from_str("Hello"), &game_state));

    assert!(!Rule::Uppercase.validate(&Password::from_str("hello"), &game_state));
}

#[test]
fn rule_special() {
    let game_state = GameState::default();

    assert!(Rule::Special.validate(&Password::from_str("$"), &game_state));
    // Anything non-ascii-alphanumeric counts as a special character
    assert!(Rule::Special.validate(&Password::from_str("😀"), &game_state));

    assert!(!Rule::Special.validate(&Password::from_str("hello123"), &game_state));
}

#[test]
fn rule_digits() {
    let game_state = GameState::default();

    assert!(Rule::Digits.validate(&Password::from_str("55555"), &game_state));

    // Each digit is considered individually
    assert!(!Rule::Digits.validate(&Password::from_str("25"), &game_state));
    assert!(!Rule::Digits.validate(&Password::from_str("hello"), &game_state));
}

#[test]
fn rule_month() {
    let game_state = GameState::default();

    assert!(Rule::Month.validate(&Password::from_str("january"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("february"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("march"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("april"), &game_state));
    // Case insensitive
    assert!(Rule::Month.validate(&Password::from_str("May"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("june"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("july"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("aUgUst"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("september"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("october"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("november"), &game_state));
    assert!(Rule::Month.validate(&Password::from_str("december"), &game_state));

    // Abbrevations not accepted
    assert!(!Rule::Month.validate(&Password::from_str("dec"), &game_state));
}

#[test]
fn rule_roman() {
    let game_state = GameState::default();

    assert!(Rule::Roman.validate(&Password::from_str("V"), &game_state));
    assert!(Rule::Roman.validate(&Password::from_str("M"), &game_state));
    assert!(Rule::Roman.validate(&Password::from_str("CII"), &game_state));

    // Case sensitive
    assert!(!Rule::Roman.validate(&Password::from_str("i"), &game_state));
    assert!(!Rule::Roman.validate(&Password::from_str("hello"), &game_state));
}

#[test]
fn rule_sponsors() {
    let game_state = GameState::default();

    assert!(Rule::Sponsors.validate(&Password::from_str("pepsicola"), &game_state));
    assert!(Rule::Sponsors.validate(&Password::from_str("starbucks"), &game_state));
    assert!(Rule::Sponsors.validate(&Password::from_str("shell"), &game_state));

    assert!(!Rule::Sponsors.validate(&Password::from_str("coke"), &game_state));
    assert!(!Rule::Sponsors.validate(&Password::from_str("exxon"), &game_state));
}

#[test]
fn rule_roman_multiply() {
    let game_state = GameState::default();

    assert!(Rule::RomanMultiply.validate(&Password::from_str("VII V"), &game_state));
    assert!(Rule::RomanMultiply.validate(&Password::from_str("XXXV"), &game_state));
    assert!(Rule::RomanMultiply.validate(&Password::from_str("VII V I"), &game_state));
    assert!(Rule::RomanMultiply.validate(&Password::from_str("VII V I I"), &game_state));

    assert!(!Rule::RomanMultiply.validate(&Password::from_str("xxxv"), &game_state));
    assert!(!Rule::RomanMultiply.validate(&Password::from_str("VII V C"), &game_state));
}

#[test]
fn rule_periodic_table() {
    let game_state = GameState::default();

    assert!(Rule::PeriodicTable.validate(&Password::from_str("Au"), &game_state));
    // "He" counts as helium, not hydrogen with an unrelated "e"
    assert!(Rule::PeriodicTable.validate(&Password::from_str("He"), &game_state));

    assert!(!Rule::PeriodicTable.validate(&Password::from_str("I"), &game_state));
    // Case sensitive
    assert!(!Rule::PeriodicTable.validate(&Password::from_str("ag"), &game_state));
}

#[test]
fn rule_moon_phase() {
    let game_state = GameState::default();

    let full_moon_datetime = Local.with_ymd_and_hms(2023, 7, 4, 0, 0, 0).unwrap();
    assert!(Rule::MoonPhase.validate_at_time(
        &Password::from_str("🌕"),
        &game_state,
        &full_moon_datetime
    ));
    assert!(Rule::MoonPhase.validate_at_time(
        &Password::from_str("🌝"),
        &game_state,
        &full_moon_datetime
    ));
    assert!(!Rule::MoonPhase.validate_at_time(
        &Password::from_str("🌑🌗"),
        &game_state,
        &full_moon_datetime
    ));

    let waning_crescent_datetime = Local.with_ymd_and_hms(2023, 7, 12, 0, 0, 0).unwrap();
    assert!(Rule::MoonPhase.validate_at_time(
        &Password::from_str("🌒"),
        &game_state,
        &waning_crescent_datetime
    ));
    assert!(Rule::MoonPhase.validate_at_time(
        &Password::from_str("🌘"),
        &game_state,
        &waning_crescent_datetime
    ));
    assert!(!Rule::MoonPhase.validate_at_time(
        &Password::from_str("🌕🌑🌖🌗"),
        &game_state,
        &waning_crescent_datetime
    ));
}

#[test]
fn rule_leap_year() {
    let game_state = GameState::default();

    assert!(Rule::LeapYear.validate(&Password::from_str("2000"), &game_state));
    assert!(Rule::LeapYear.validate(&Password::from_str("0"), &game_state));

    // 1900 is divisible by four, but 100 and not 400
    assert!(!Rule::LeapYear.validate(&Password::from_str("1900"), &game_state));
    assert!(!Rule::LeapYear.validate(&Password::from_str("1990"), &game_state));
}

#[test]
fn rule_atomic_number() {
    let game_state = GameState::default();

    assert!(Rule::AtomicNumber.validate(&Password::from_str("Nd Zr Fm"), &game_state));
    assert!(Rule::AtomicNumber.validate(&Password::from_str("FmFm"), &game_state));

    assert!(!Rule::AtomicNumber.validate(&Password::from_str("He"), &game_state));
    assert!(!Rule::AtomicNumber.validate(&Password::from_str("fmfm"), &game_state));
}

#[test]
fn rule_fire() {
    let mut game_state = GameState::default();

    // Fire is okay if it hasn't been started by the game yet
    assert!(!Rule::Fire.validate(&Password::from_str("hello🔥"), &game_state));

    game_state.fire_started = true;
    assert!(Rule::Fire.validate(&Password::from_str("hello"), &game_state));
    assert!(!Rule::Fire.validate(&Password::from_str("hello🔥"), &game_state));
}

#[test]
fn rule_strength() {
    let game_state = GameState::default();

    assert!(Rule::Strength.validate(&Password::from_str("🏋️‍♂️🏋️‍♂️🏋️‍♂️"), &game_state));
    assert!(Rule::Strength.validate(&Password::from_str("foo🏋️‍♂️🏋️‍♂️🏋️‍♂️🏋️‍♂️🏋️‍♂️"), &game_state));

    assert!(!Rule::Strength.validate(&Password::from_str("hello"), &game_state));
    assert!(!Rule::Strength.validate(&Password::from_str("🏋️‍♂️🏋️‍♂️bar"), &game_state));
}

#[test]
fn rule_affirmation() {
    let game_state = GameState::default();

    assert!(Rule::Affirmation.validate(&Password::from_str("i am loved123"), &game_state));
    // Missing whitespace is allowed...
    assert!(Rule::Affirmation.validate(&Password::from_str("iamloved"), &game_state));
    assert!(Rule::Affirmation.validate(&Password::from_str("i am worthy456"), &game_state));
    assert!(Rule::Affirmation.validate(&Password::from_str("789i am enough"), &game_state));

    assert!(!Rule::Affirmation.validate(&Password::from_str("i am not loved"), &game_state));
    // ...but only if it's all missing
    assert!(!Rule::Affirmation.validate(&Password::from_str("iam loved"), &game_state));
    assert!(!Rule::Affirmation.validate(&Password::from_str("i amloved"), &game_state));
    assert!(!Rule::Affirmation.validate(&Password::from_str("i am not enough"), &game_state));
}

#[test]
fn rule_include_length() {
    let game_state = GameState::default();

    assert!(Rule::IncludeLength.validate(&Password::from_str("12345"), &game_state));
    assert!(Rule::IncludeLength.validate(&Password::from_str("14 hello there"), &game_state));

    assert!(!Rule::IncludeLength.validate(&Password::from_str("12346"), &game_state));
}

#[test]
fn rule_time() {
    let game_state = GameState::default();

    let datetime_now = chrono::Local::now();
    assert!(Rule::Time.validate(
        &Password::from_str(&datetime_now.format("%l:%M").to_string().trim().to_owned()),
        &game_state
    ));
    let datetime_future = chrono::Local::now() + chrono::Duration::seconds(1_000_000);
    assert!(!Rule::Time.validate(
        &Password::from_str(
            &datetime_future
                .format("%l:%M")
                .to_string()
                .trim()
                .to_owned()
        ),
        &game_state
    ));

    let datetime = Local.with_ymd_and_hms(2023, 7, 12, 4, 8, 20).unwrap();
    assert!(Rule::Time.validate_at_time(&Password::from_str("4:08"), &game_state, &datetime));
    assert!(!Rule::Time.validate_at_time(&Password::from_str("12:34"), &game_state, &datetime));
}

#[test]
fn rule_egg() {
    let mut game_state = GameState::default();

    assert!(Rule::Egg.validate(&Password::from_str("egg: 🥚"), &game_state));
    assert!(Rule::Egg.validate(&Password::from_str("no egg"), &game_state));

    game_state.egg_placed = true;
    assert!(Rule::Egg.validate(&Password::from_str("egg: 🥚"), &game_state));
    assert!(!Rule::Egg.validate(&Password::from_str("no egg"), &game_state));

    game_state.paul_hatched = true;
    assert!(Rule::Egg.validate(&Password::from_str("paul: 🐔"), &game_state));
    assert!(!Rule::Egg.validate(&Password::from_str("no paul"), &game_state));
    // Paul has been slain
}

#[test]
fn rule_captcha() {
    let game_state = GameState::default();
    let rule = Rule::Captcha("d22bd".into());

    assert!(rule.validate(&Password::from_str("d22bd"), &game_state));
    assert!(rule.validate(&Password::from_str("food22bdbar"), &game_state));

    // Case sensitive
    assert!(!rule.validate(&Password::from_str("D22bd"), &game_state));
    assert!(!rule.validate(&Password::from_str("hello"), &game_state));
}

#[test]
#[ignore]
fn rule_wordle() {
    let game_state = GameState::default();

    // 2023-07-09's answer was "enter"
    let datetime = Local.with_ymd_and_hms(2023, 7, 9, 0, 0, 0).unwrap();

    assert!(Rule::Wordle.validate_at_time(&Password::from_str("enter"), &game_state, &datetime));
    assert!(Rule::Wordle.validate_at_time(
        &Password::from_str("123enterfoo"),
        &game_state,
        &datetime
    ));
    // Case insensitive
    assert!(Rule::Wordle.validate_at_time(&Password::from_str("enTeR"), &game_state, &datetime));

    assert!(!Rule::Wordle.validate_at_time(&Password::from_str(""), &game_state, &datetime));
    assert!(!Rule::Wordle.validate_at_time(&Password::from_str("hello"), &game_state, &datetime));
}

#[test]
fn rule_hatch() {
    let mut game_state = GameState::default();

    // Paul hasn't hatched yet
    assert!(Rule::Hatch.validate(&Password::from_str("🐛"), &game_state));
    assert!(Rule::Hatch.validate(&Password::from_str("nobugs"), &game_state));

    // Paul is hatched and hungry
    game_state.paul_hatched = true;
    assert!(Rule::Hatch.validate(&Password::from_str("🐛"), &game_state));
    assert!(Rule::Hatch.validate(&Password::from_str("bugs🐛🐛🐛"), &game_state));

    assert!(!Rule::Hatch.validate(&Password::from_str(""), &game_state));

    // Paul is hatched and currently eating
    game_state.paul_eating = true;
    assert!(Rule::Hatch.validate(&Password::from_str("🐛"), &game_state));
    assert!(Rule::Hatch.validate(&Password::from_str("nobugs"), &game_state));
}

#[test]
fn rule_prime_length() {
    let game_state = GameState::default();

    assert!(Rule::PrimeLength.validate(&Password::from_str("12"), &game_state));
    assert!(Rule::PrimeLength.validate(&Password::from_str("1234567"), &game_state));

    assert!(!Rule::PrimeLength.validate(&Password::from_str(""), &game_state));
    assert!(!Rule::PrimeLength.validate(&Password::from_str("1"), &game_state));
    assert!(!Rule::PrimeLength.validate(&Password::from_str("123456789"), &game_state));
}

#[test]
fn rule_sacrifice() {
    let mut game_state = GameState::default();

    // No letters sacrified yet, which is a failure
    assert!(!Rule::Sacrifice.validate(
        &Password::from_str("abcdefghijklmnopqrstuvwxyz"),
        &game_state
    ));

    game_state.sacrificed_letters.push('a');
    game_state.sacrificed_letters.push('b');
    assert!(Rule::Sacrifice.validate(&Password::from_str("cdefghijklmnopqrstuvwxyz"), &game_state));
    assert!(!Rule::Sacrifice.validate(&Password::from_str("a"), &game_state));
    assert!(!Rule::Sacrifice.validate(&Password::from_str("b"), &game_state));
}

#[test]
fn rule_skip() {
    let game_state = GameState::default();

    // Anything passes, it's a no-op rule
    assert!(Rule::Skip.validate(&Password::from_str(""), &game_state));
    assert!(Rule::Skip.validate(&Password::from_str("12345"), &game_state));
    assert!(Rule::Skip.validate(&Password::from_str("hello😀"), &game_state));
}

#[test]
fn rule_final() {
    let game_state = GameState::default();

    // Anything passes, it's a no-op rule
    assert!(Rule::Final.validate(&Password::from_str(""), &game_state));
    assert!(Rule::Final.validate(&Password::from_str("12345"), &game_state));
    assert!(Rule::Final.validate(&Password::from_str("hello😀"), &game_state));
}

#[test]
fn rule_bold_vowels() {
    let game_state = GameState::default();

    let password = {
        let mut p = Password::from_str("bueioak");
        for i in 1..6 {
            p.format(i, &FormatChange::BoldOn);
        }
        p
    };
    assert!(Rule::BoldVowels.validate(&password, &game_state));
    // No vowels
    assert!(Rule::BoldVowels.validate(&Password::from_str("bcdmnp"), &game_state));

    assert!(!Rule::BoldVowels.validate(&Password::from_str("ueioa"), &game_state));
}

#[test]
fn rule_hex() {
    let game_state = GameState::default();

    // #f76bf6
    let rule = Rule::Hex(Color {
        r: 247,
        g: 107,
        b: 246,
    });
    assert!(rule.validate(&Password::from_str("#f76bf6"), &game_state));
    // Case insensitive, and no hash required
    assert!(rule.validate(&Password::from_str("f76BF6"), &game_state));

    assert!(!rule.validate(&Password::from_str("f7 6b f6"), &game_state));
    assert!(!rule.validate(&Password::from_str("247,107,246"), &game_state));
}

#[test]
#[ignore]
fn rule_youtube() {
    let game_state = GameState::default();

    let rule = Rule::Youtube(14);
    assert!(rule.validate(
        &Password::from_str("youtube.com/watch?v=Hc6J5rlKhIc"),
        &game_state
    ));
    assert!(!rule.validate(
        &Password::from_str("youtube.com/watch?v=FiARsQSlzDc"),
        &game_state
    ));
}

#[test]
fn rule_chess() {
    let game_state = GameState::default();

    let rule =
        Rule::Chess("r2qkb1r/pp2nppp/3p4/2pNN1B1/2BnP3/3P4/PPP2PPP/R2bK2R w KQkq - 0 1".into());
    assert!(rule.validate(&Password::from_str("Nf6+"), &game_state));

    // Invalid notation (missing "+" for check)
    assert!(!rule.validate(&Password::from_str("Nf6"), &game_state));
    // Case sensitive
    assert!(!rule.validate(&Password::from_str("nf6"), &game_state));
}

#[test]
fn rule_geo() {
    let game_state = GameState::default();

    let rule = Rule::Geo(Coords {
        lat: NotNan::new(-25.35068396746521).unwrap(),
        long: NotNan::new(131.0463222711639).unwrap(),
    });
    assert!(rule.validate(&Password::from_str("australia"), &game_state));
    assert!(rule.validate(&Password::from_str("ausTraLiA"), &game_state));

    assert!(!rule.validate(&Password::from_str("austria"), &game_state));
}

#[test]
fn rule_times_new_roman() {
    let game_state = GameState::default();

    let password = {
        let mut p = Password::from_str("MCM");
        for i in 0..3 {
            p.format(i, &FormatChange::FontFamily(FontFamily::TimesNewRoman))
        }
        p
    };
    assert!(Rule::TimesNewRoman.validate(&password, &game_state));
    // No roman numerals
    assert!(Rule::TimesNewRoman.validate(&Password::from_str("foo"), &game_state));

    assert!(!Rule::TimesNewRoman.validate(&Password::from_str("VII"), &game_state));
}

#[test]
fn rule_digit_font_size() {
    let game_state = GameState::default();

    // No digits
    assert!(Rule::DigitFontSize.validate(&Password::from_str("foo"), &game_state));

    // Default font size (not a square)
    let mut password = Password::from_str("023");
    assert!(!Rule::DigitFontSize.validate(&password, &game_state));

    password.format(0, &FormatChange::FontSize(FontSize::try_from(0).unwrap()));
    password.format(1, &FormatChange::FontSize(FontSize::try_from(4).unwrap()));
    password.format(2, &FormatChange::FontSize(FontSize::try_from(9).unwrap()));
    assert!(Rule::DigitFontSize.validate(&password, &game_state));
}

#[test]
fn rule_letter_font_size() {
    let game_state = GameState::default();
    let mut password = Password::from_str("aAb");

    // Both a's have the same font size (the default)
    assert!(!Rule::LetterFontSize.validate(&password, &game_state));

    password.format(0, &FormatChange::FontSize(FontSize::Px16));
    assert!(Rule::LetterFontSize.validate(&password, &game_state));
}

#[test]
fn rule_twice_italic() {
    let game_state = GameState::default();
    let mut password = Password::from_str("foobar");

    // No bold or italic characters
    assert!(Rule::TwiceItalic.validate(&password, &game_state));

    // italic == 2 * bold
    password.format(0, &FormatChange::ItalicOn);
    password.format(1, &FormatChange::ItalicOn);
    password.format(2, &FormatChange::BoldOn);
    assert!(Rule::TwiceItalic.validate(&password, &game_state));

    // italic < 2 * bold
    password.format(0, &FormatChange::BoldOn);
    assert!(!Rule::TwiceItalic.validate(&password, &game_state));
}

#[test]
fn rule_wingdings() {
    let game_state = GameState::default();
    let mut password = Password::from_str("foobar");

    // 1/6 < 0.3
    password.format(0, &FormatChange::FontFamily(FontFamily::Wingdings));
    assert!(!Rule::Wingdings.validate(&password, &game_state));

    // 2/6 >= 0.3
    password.format(3, &FormatChange::FontFamily(FontFamily::Wingdings));
    assert!(Rule::Wingdings.validate(&password, &game_state));
}
