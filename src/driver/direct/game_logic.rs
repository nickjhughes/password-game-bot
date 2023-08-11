use crate::password::{Change, MutablePassword};
use rand::{prelude::*, seq::SliceRandom};
use unicode_segmentation::UnicodeSegmentation;

/// Start a fire in the password by replacing a random grapheme with "🔥".
pub fn start_fire(password: &mut MutablePassword) {
    // Choose a random grapheme index at least 5 characters away from Paul ("🥚")
    let graphemes = password.as_str().graphemes(true).collect::<Vec<_>>();
    let valid_indices = if let Some(egg_index) = graphemes.iter().position(|g| *g == "🥚") {
        let before_egg = 0..egg_index.saturating_sub(5);
        let after_egg = (egg_index + 6).min(password.len() - 1)..password.len();
        before_egg.chain(after_egg).collect::<Vec<usize>>()
    } else {
        (0..graphemes.len()).collect::<Vec<usize>>()
    };
    let mut rng = thread_rng();
    let index = valid_indices.choose(&mut rng).unwrap();
    password.queue_change(Change::Replace {
        index: *index,
        new_grapheme: "🔥".into(),
        ignore_protection: true,
    });
    password.commit_changes();
}

/// Spread the fire. Each contiguous section of 🔥 should grow by one in both directions.
#[allow(dead_code)]
pub fn spread_fire(password: &mut MutablePassword) {
    let graphemes = password.as_str().graphemes(true).collect::<Vec<_>>();
    let mut changes = Vec::new();
    for i in 0..password.len() {
        if graphemes[i] == "🔥" {
            continue;
        }
        if (i > 0 && graphemes[i - 1] == "🔥")
            || (i < graphemes.len() - 1 && graphemes[i + 1] == "🔥")
        {
            changes.push(Change::Replace {
                index: i,
                new_grapheme: "🔥".into(),
                ignore_protection: true,
            });
        }
    }
    for change in changes {
        password.queue_change(change);
    }
    password.commit_changes();
}

// Hatch Paul, turning "🥚" into "🐔".
pub fn hatch_egg(password: &mut MutablePassword) {
    for (index, grapheme) in password.as_str().graphemes(true).enumerate() {
        if grapheme == "🥚" {
            password.queue_change(crate::password::Change::Replace {
                index,
                new_grapheme: "🐔".into(),
                ignore_protection: true,
            });
            password.commit_changes();
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{spread_fire, start_fire};
    use crate::password::MutablePassword;
    use std::collections::HashSet;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn starting_fire() {
        let mut password = MutablePassword::from_str("hello");
        start_fire(&mut password);
        assert!(password.as_str().contains("🔥"));
        assert_eq!(password.as_str().matches("🔥").count(), 1);

        // Cover all indices
        let mut indices = HashSet::new();
        while indices.len() < 5 {
            let mut password = MutablePassword::from_str("hello");
            start_fire(&mut password);
            assert!(password.as_str().contains("🔥"));
            for (index, grapheme) in password.as_str().graphemes(true).enumerate() {
                if grapheme == "🔥" {
                    indices.insert(index);
                    break;
                }
            }
        }

        // Don't place fire within 5 indices of Paul ("🥚")
        let mut indices = HashSet::new();
        while indices.len() < 6 {
            let mut password = MutablePassword::from_str("avoid the🥚egg foo");
            start_fire(&mut password);
            assert!(password.as_str().contains("🔥"));
            for (index, grapheme) in password.as_str().graphemes(true).enumerate() {
                if grapheme == "🔥" {
                    indices.insert(index);
                    break;
                }
            }
        }
        let mut indices = indices.drain().collect::<Vec<usize>>();
        indices.sort();
        assert_eq!(indices, vec![0, 1, 2, 3, 15, 16]);
    }

    #[test]
    fn spreading_fire() {
        let mut password = MutablePassword::from_str("he🔥lo");
        spread_fire(&mut password);
        assert_eq!(password.as_str(), "h🔥🔥🔥o");

        let mut password = MutablePassword::from_str("🔥hello🔥");
        spread_fire(&mut password);
        assert_eq!(password.as_str(), "🔥🔥ell🔥🔥");
    }
}
