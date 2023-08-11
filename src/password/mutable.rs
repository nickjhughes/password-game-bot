use super::{Change, Password, ProtectedPassword};

/// A password which can have `Change`s applied to it.
#[derive(Debug, Default)]
pub struct MutablePassword {
    /// The password with associated notion of protected graphemes which
    /// can't be removed.
    password: ProtectedPassword,
    /// The current set of queued changes to the password.
    changes: Vec<Change>,
}

impl MutablePassword {
    /// Wrap the given protected password into a mutable password.
    #[allow(dead_code)]
    pub fn new(password: ProtectedPassword) -> Self {
        MutablePassword {
            password,
            changes: Vec::new(),
        }
    }

    /// Construct a new password from the given string.
    #[cfg(test)]
    pub fn from_str(string: &str) -> Self {
        MutablePassword {
            password: ProtectedPassword::from_str(string),
            changes: Vec::new(),
        }
    }

    /// The underlying `Password`.
    pub fn raw_password(&self) -> &Password {
        self.password.raw_password()
    }

    /// The underlying `Password` mutably.
    pub fn raw_password_mut(&mut self) -> &mut Password {
        self.password.raw_password_mut()
    }

    /// Get the protected graphemes.
    pub fn protected_graphemes(&self) -> &[bool] {
        self.password.protected_graphemes()
    }

    /// The length of the password in terms of grapheme clusters.
    pub fn len(&self) -> usize {
        self.password.len()
    }

    /// The password as a string slice.
    pub fn as_str(&self) -> &str {
        self.password.as_str()
    }

    /// The number of queued changes.
    #[allow(dead_code)]
    pub fn queue_len(&self) -> usize {
        self.changes.len()
    }

    /// Get the current changes.
    #[allow(dead_code)]
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    /// Queue the given change to the password. Panics if the given change is invalid
    /// (e.g., if an index is invalid, or a protected grapheme would be modified/removed).
    pub fn queue_change(&mut self, change: Change) {
        let is_valid = match &change {
            Change::Append { .. } => {
                // Appends are always valid
                true
            }
            Change::Prepend { .. } => {
                // Prepends are always valid
                true
            }
            Change::Insert { .. } => {
                // Inserts are always valid
                // Note that inserting between two protected graphemes probably
                // shouldn't be allowed, but we currently don't know if they're
                // part of the same protected "block" or not. So for now, rely
                // on the caller knowing what they're doing.
                true
            }
            Change::Remove {
                index,
                ignore_protection,
            } => {
                // Valid as long as the grapheme isn't protected
                *ignore_protection || !self.password.protected_graphemes()[*index]
            }
            Change::Replace {
                index,
                ignore_protection,
                ..
            } => {
                // Valid as long as the grapheme isn't protected
                *ignore_protection || !self.password.protected_graphemes()[*index]
            }
            Change::Format { index, .. } => {
                // Only invalid if the index is invalid (formatting is not protected)
                *index < self.password.len()
            }
        };
        if !is_valid {
            panic!("invalid change: {:?}", change);
        }

        self.changes.push(change);
    }

    /// Sort changes such that they can be committed.
    fn sort_changes_for_commit(&mut self) {
        // Default sort is correct, other than that removals need to be reversed
        self.changes.sort();
        let first_removal = self
            .changes
            .iter()
            .position(|c| matches!(c, Change::Remove { .. }));
        if let Some(first_removal) = first_removal {
            let (_, right) = self.changes.split_at_mut(first_removal);
            right.reverse();
        }
    }

    /// Commit the current set of queued changes. Will perform operations in the
    /// following order:
    ///  - format
    ///  - append
    ///  - replace
    ///  - remove
    /// Additionally, removals will be performed starting at the end of the string
    /// and working backwards.
    pub fn commit_changes(&mut self) {
        self.sort_changes_for_commit();
        for change in self.changes.drain(..) {
            self.password.apply_change(&change);
        }
    }

    /// Raw insert into the password.

    /// Protect the given grapheme.
    #[cfg(test)]
    pub fn protect(&mut self, index: usize) {
        self.password.protect(index);
    }
}

#[cfg(test)]
mod tests {
    use super::{MutablePassword, ProtectedPassword};
    use crate::password::{change::Change, Password};

    #[test]
    #[should_panic]
    fn remove_protected() {
        let mut password = MutablePassword::new(ProtectedPassword::new(Password::from_str("foo")));
        password.password.protect(0);
        password.queue_change(Change::Remove {
            index: 0,
            ignore_protection: false,
        });
    }

    #[test]
    #[should_panic]
    fn replace_protected() {
        let mut password = MutablePassword::new(ProtectedPassword::new(Password::from_str("foo")));
        password.password.protect(0);
        password.queue_change(Change::Replace {
            index: 0,
            new_grapheme: "b".into(),
            ignore_protection: false,
        });
    }

    #[test]
    fn multiple_remove() {
        // Changes in order
        let mut password = MutablePassword::new(ProtectedPassword::new(Password::from_str("abc")));
        password.changes.push(Change::Remove {
            index: 0,
            ignore_protection: false,
        });
        password.changes.push(Change::Remove {
            index: 1,
            ignore_protection: false,
        });
        password.commit_changes();
        assert_eq!(password.as_str(), "c");

        // Changes in reverse order
        let mut password = MutablePassword::new(ProtectedPassword::new(Password::from_str("abc")));
        password.changes.push(Change::Remove {
            index: 2,
            ignore_protection: false,
        });
        password.changes.push(Change::Remove {
            index: 0,
            ignore_protection: false,
        });
        password.commit_changes();
        assert_eq!(password.as_str(), "b");
    }
}
