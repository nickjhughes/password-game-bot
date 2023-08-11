use headless_chrome::browser::tab::ModifierKey;

use super::{super::Driver, WebDriver};
use crate::{password::Change, solver::Solver};

#[test]
#[ignore]
fn get_password() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "hello".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "hello");

    driver
        .update_password(&mut vec![Change::Append {
            string: "🏋️‍♂️".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "hello🏋️‍♂️");
}

#[test]
#[ignore]
fn update_password_append() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "01234".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "01234");
}

#[test]
#[ignore]
fn update_password_multiple_appends() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![
            Change::Append {
                string: "a".into(),
                protected: false,
            },
            Change::Append {
                string: "b".into(),
                protected: false,
            },
        ])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "ab");
}

#[test]
#[ignore]
fn update_password_insert() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "for".into(),
            protected: false,
        }])
        .unwrap();
    driver
        .update_password(&mut vec![Change::Insert {
            index: 2,
            string: "oba".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "foobar");
}

#[test]
#[ignore]
fn update_password_replace() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "01234".into(),
            protected: false,
        }])
        .unwrap();
    driver
        .update_password(&mut vec![Change::Replace {
            index: 2,
            new_grapheme: "t".into(),
            ignore_protection: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "01t34");
}

#[test]
#[ignore]
fn update_password_remove() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "01234".into(),
            protected: false,
        }])
        .unwrap();
    driver
        .update_password(&mut vec![Change::Remove {
            index: 3,
            ignore_protection: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "0124");
}

#[test]
#[ignore]
fn update_password_multiple_removals() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "01234".into(),
            protected: false,
        }])
        .unwrap();
    driver
        .update_password(&mut vec![
            Change::Remove {
                index: 1,
                ignore_protection: false,
            },
            Change::Remove {
                index: 0,
                ignore_protection: false,
            },
        ])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "234");
}

#[test]
#[ignore]
fn update_password_remove_emoji() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "🔥".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "🔥");
    driver
        .update_password(&mut vec![Change::Remove {
            index: 0,
            ignore_protection: false,
        }])
        .unwrap();
    assert!(driver.get_password().unwrap().is_empty());
}

#[test]
#[ignore]
fn update_password_remove_zwj_emoji() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "👨‍👩‍👧‍👧foo".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "👨‍👩‍👧‍👧foo");
    driver
        .update_password(&mut vec![Change::Remove {
            index: 0,
            ignore_protection: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "foo");
}
#[test]
#[ignore]
fn cursor_movement_zwj_emoji() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "👨‍👩‍👧‍👧foo".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "👨‍👩‍👧‍👧foo");

    driver.cursor_to(0).unwrap();

    driver
        .update_password(&mut vec![Change::Append {
            string: "bar".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "👨‍👩‍👧‍👧foobar");
}

#[test]
#[ignore]
fn key_press_with_modifiers() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "hello".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "hello");
    driver.cursor_to(0).unwrap();

    driver
        .tab
        .press_key_with_modifiers("ArrowRight", Some(&[ModifierKey::Shift]))
        .unwrap();
    driver.tab.press_key("ArrowRight").unwrap();

    driver
        .tab
        .press_key_with_modifiers("ArrowRight", Some(&[ModifierKey::Shift]))
        .unwrap();
    driver
        .tab
        .press_key_with_modifiers("ArrowRight", Some(&[ModifierKey::Shift]))
        .unwrap();
    driver.tab.press_key("a").unwrap();

    assert_eq!(driver.get_password().unwrap(), "halo");
}

#[test]
#[ignore]
fn delete_password() {
    let solver = Solver::default();
    let mut driver = WebDriver::new(solver).unwrap();
    assert!(driver.get_password().unwrap().is_empty());

    driver
        .update_password(&mut vec![Change::Append {
            string: "🥚ello".into(),
            protected: false,
        }])
        .unwrap();
    assert_eq!(driver.get_password().unwrap(), "🥚ello");

    driver.tab.press_key("a").unwrap();

    driver.delete_and_retype_passsword().unwrap();
    assert_eq!(driver.get_password().unwrap(), "🥚ello");
}
