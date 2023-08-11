use super::Solver;
use crate::{
    game::{
        Game,
        {rule::Color, Rule},
    },
    password::{Change, FormatChange, MutablePassword},
};

fn test_setup(rule: Rule, password: &str) -> (Game, Solver) {
    let game = Game::default();
    let solver = Solver {
        password: MutablePassword::from_str(password),
        violated_rules: vec![rule],
        sacrificed_letters: Vec::new(),
        length_string: None,
        time_string: None,
        goal_length: None,
    };
    (game, solver)
}

#[test]
fn rule_min_length() {
    let rule = Rule::MinLength;

    let (game, mut solver) = test_setup(rule.clone(), "🏋️‍♂️1");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_number() {
    let rule = Rule::Number;

    let (game, mut solver) = test_setup(rule.clone(), "On🏋️‍♂️e!");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_uppercase() {
    let rule = Rule::Uppercase;

    let (game, mut solver) = test_setup(rule.clone(), "hello🏋️‍♂️");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_special() {
    let rule = Rule::Special;

    let (game, mut solver) = test_setup(rule.clone(), "Hello23");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_digits() {
    let rule = Rule::Digits;

    // Current sum < 25
    let (game, mut solver) = test_setup(rule.clone(), "1🏋️‍♂️");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));

    // Current sum == 25
    let (game, mut solver) = test_setup(rule.clone(), "9🏋️‍♂️97");
    assert!(rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
    assert_eq!(solver.password.len(), 4);

    // Current sum > 25
    let (game, mut solver) = test_setup(rule.clone(), "55🏋️‍♂️5546");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));

    // Current sum > 25 and some digits are protected
    let (game, mut solver) = test_setup(rule.clone(), "155555");
    solver.password.protect(0);
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_month() {
    let rule = Rule::Month;

    let (game, mut solver) = test_setup(rule.clone(), "🏋️‍♂️Dec@");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_roman() {
    let rule = Rule::Roman;

    let (game, mut solver) = test_setup(rule.clone(), "eci$ 🏋️‍♂️");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_sponsors() {
    let rule = Rule::Sponsors;

    let (game, mut solver) = test_setup(rule.clone(), "dew123 test 🏋️‍♂️");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_roman_multiply() {
    let rule = Rule::RomanMultiply;

    let (game, mut solver) = test_setup(rule.clone(), "VIIXDIaIaI");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_atomic_number() {
    let rule = Rule::AtomicNumber;

    // Atomic number sum < 200
    let (game, mut solver) = test_setup(rule.clone(), "FooBar");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));

    // Atomic number sum > 200
    let (game, mut solver) = test_setup(rule.clone(), "FooBarHeIOU");
    solver.password.protect(0);
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));

    // Don't add elements which contain roman numerals
    let (game, mut solver) = test_setup(rule.clone(), "FmAg");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(!solver.password.as_str().contains("I"));
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_skip() {
    let (game, mut solver) = test_setup(Rule::Skip, "foo");
    let changes = solver.solve_rule(&Rule::Skip, &game.state, 0);
    assert!(changes.unwrap().is_empty());
}

#[test]
fn rule_bold_vowels() {
    let rule = Rule::BoldVowels;

    let (game, mut solver) = test_setup(rule.clone(), "foobar");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_fire() {
    let rule = Rule::Fire;

    let (mut game, mut solver) = test_setup(rule.clone(), "f🔥🔥ooba🔥r");
    game.state.fire_started = true;
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_strength() {
    let rule = Rule::Strength;

    let (game, mut solver) = test_setup(rule.clone(), "nostrength");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_egg() {
    let rule = Rule::Egg;

    let (mut game, mut solver) = test_setup(rule.clone(), "noegg");
    game.state.egg_placed = true;
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_hatch() {
    let rule = Rule::Hatch;

    let (mut game, mut solver) = test_setup(rule.clone(), "paul: 🐔");
    game.state.egg_placed = true;
    game.state.paul_hatched = true;
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_youtube() {
    let rule = Rule::Youtube(13 * 60 + 3);

    let (game, mut solver) = test_setup(rule.clone(), "foo");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_sacrifice() {
    let rule = Rule::Sacrifice;

    let (mut game, mut solver) = test_setup(rule.clone(), "abcdefghijklmnopqrstuvwxyz");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    game.state
        .sacrificed_letters
        .extend(solver.sacrificed_letters.iter());
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_hex() {
    let rule = Rule::Hex(Color {
        r: 127,
        g: 0,
        b: 54,
    });

    let (game, mut solver) = test_setup(rule.clone(), "#123");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_twice_italic() {
    let rule = Rule::TwiceItalic;

    let (game, mut solver) = test_setup(rule.clone(), "abcdef");
    solver.password.queue_change(Change::Format {
        index: 0,
        format_change: FormatChange::BoldOn,
    });
    solver.password.queue_change(Change::Format {
        index: 1,
        format_change: FormatChange::BoldOn,
    });
    solver.password.commit_changes();
    assert!(!rule.validate(solver.password.raw_password(), &game.state));

    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_wingdings() {
    let rule = Rule::Wingdings;

    let (game, mut solver) = test_setup(rule.clone(), "0123456789");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_times_new_roman() {
    let rule = Rule::TimesNewRoman;

    let (game, mut solver) = test_setup(rule.clone(), "mmhellofooX-VIII");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_digit_font_size() {
    let rule = Rule::DigitFontSize;

    let (game, mut solver) = test_setup(rule.clone(), "0123456789abc");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_letter_font_size() {
    let rule = Rule::LetterFontSize;

    let (game, mut solver) = test_setup(rule.clone(), "aAaBbbCcccc");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}

#[test]
fn rule_time() {
    let rule = Rule::Time;

    let (game, mut solver) = test_setup(rule.clone(), "foo");
    assert!(!rule.validate(solver.password.raw_password(), &game.state));
    solver.solve_rule_and_commit(&rule, &game.state);
    assert!(rule.validate(solver.password.raw_password(), &game.state));
}
