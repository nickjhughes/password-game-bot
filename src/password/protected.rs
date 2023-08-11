use unicode_segmentation::UnicodeSegmentation;

use super::{Change, Password};

/// A password combined with the notion of protected graphemes.
#[derive(Debug, Default)]
pub struct ProtectedPassword {
    /// The password.
    password: Password,
    /// The grapheme clusters in the password which mustn't be modified.
    /// The length of this Vec corresponds to `password.len()`.
    protected_graphemes: Vec<bool>,
}

impl ProtectedPassword {
    /// Add protection to the given password.
    #[cfg(test)]
    pub fn new(password: Password) -> Self {
        let protected_graphemes = vec![false; password.len()];
        ProtectedPassword {
            password,
            protected_graphemes,
        }
    }

    /// Construct a new password from the given string.
    #[cfg(test)]
    pub fn from_str(string: &str) -> Self {
        ProtectedPassword {
            password: Password::from_str(string),
            protected_graphemes: vec![false; string.graphemes(true).count()],
        }
    }

    /// The underlying `Password`.
    pub fn raw_password(&self) -> &Password {
        &self.password
    }

    /// The underlying `Password` mutably.
    pub fn raw_password_mut(&mut self) -> &mut Password {
        &mut self.password
    }

    /// The length of the password in terms of grapheme clusters.
    pub fn len(&self) -> usize {
        self.password.len()
    }

    /// The password as a string slice.
    pub fn as_str(&self) -> &str {
        self.password.as_str()
    }

    /// Get the protected graphemes.
    pub fn protected_graphemes(&self) -> &[bool] {
        &self.protected_graphemes
    }

    /// Get the protected graphemes as a bitstring.
    /// e.g., if password is "hello" with "ll" protected, then this returns "00110".
    /// The results will be of length `password.len()`.
    #[cfg(test)]
    pub fn protected_chars_bitstring(&self) -> String {
        self.protected_graphemes
            .iter()
            .map(|b| if *b { '1' } else { '0' })
            .collect::<String>()
    }

    /// Protect the given grapheme.
    #[cfg(test)]
    pub fn protect(&mut self, index: usize) {
        self.protected_graphemes[index] = true;
    }

    /// Apply the given change to the password. Panics if it's not valid.
    pub fn apply_change(&mut self, change: &Change) {
        match change {
            Change::Format {
                index,
                format_change,
            } => {
                self.password.format(*index, format_change);
            }
            Change::Append { string, protected } => {
                self.password.append(string);
                for _ in 0..string.graphemes(true).count() {
                    self.protected_graphemes.push(*protected);
                }

                debug_assert_eq!(self.password.len(), self.protected_graphemes.len());
            }
            Change::Prepend { string, protected } => {
                self.password.prepend(string);
                for _ in 0..string.graphemes(true).count() {
                    self.protected_graphemes.insert(0, *protected);
                }

                debug_assert_eq!(self.password.len(), self.protected_graphemes.len());
            }
            Change::Insert {
                index,
                string,
                protected,
            } => {
                self.password.insert(*index, string);
                for _ in 0..string.graphemes(true).count() {
                    self.protected_graphemes.insert(*index, *protected);
                }

                debug_assert_eq!(self.password.len(), self.protected_graphemes.len());
            }
            Change::Remove {
                index,
                ignore_protection,
            } => {
                assert!(*ignore_protection || !self.protected_graphemes[*index]);

                self.password.remove(*index);
                self.protected_graphemes.remove(*index);

                debug_assert_eq!(self.password.len(), self.protected_graphemes.len());
            }
            Change::Replace {
                index,
                new_grapheme,
                ignore_protection,
            } => {
                assert!(*ignore_protection || !self.protected_graphemes[*index]);

                self.password.replace(*index, new_grapheme);

                debug_assert_eq!(self.password.len(), self.protected_graphemes.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Change, Password, ProtectedPassword};

    #[test]
    fn protected_bitstring() {
        // ASCII
        let password = ProtectedPassword {
            password: Password::from_str("hello"),
            protected_graphemes: vec![false, false, true, true, false],
        };
        let bitstring = password.protected_chars_bitstring();
        assert_eq!(bitstring, "00110");

        // Unicode
        let password = ProtectedPassword {
            password: Password::from_str("🏋️‍♂️1"),
            protected_graphemes: vec![true, false],
        };
        let bitstring = password.protected_chars_bitstring();
        assert_eq!(bitstring, "10");
    }

    #[test]
    fn append() {
        // Unprotected
        let mut password = ProtectedPassword::from_str("foo");
        password.apply_change(&Change::Append {
            string: "bar".into(),
            protected: false,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.protected_graphemes(), vec![false; 6]);

        // Protected
        let mut password = ProtectedPassword::from_str("foo");
        password.apply_change(&Change::Append {
            string: "bar".into(),
            protected: true,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(
            password.protected_graphemes(),
            vec![false, false, false, true, true, true]
        );
    }

    #[test]
    fn prepend() {
        // Unprotected
        let mut password = ProtectedPassword::from_str("bar");
        password.apply_change(&Change::Prepend {
            string: "foo".into(),
            protected: false,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.protected_graphemes(), vec![false; 6]);

        // Protected
        let mut password = ProtectedPassword::from_str("bar");
        password.apply_change(&Change::Prepend {
            string: "foo".into(),
            protected: true,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(
            password.protected_graphemes(),
            vec![true, true, true, false, false, false]
        );
    }

    #[test]
    fn insert() {
        // Unprotected
        let mut password = ProtectedPassword::from_str("for");
        password.apply_change(&Change::Insert {
            index: 2,
            string: "oba".into(),
            protected: false,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.protected_graphemes(), vec![false; 6]);

        // Protected
        let mut password = ProtectedPassword::from_str("for");
        password.apply_change(&Change::Insert {
            index: 2,
            string: "oba".into(),
            protected: true,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(
            password.protected_graphemes(),
            vec![false, false, true, true, true, false]
        );

        // At start
        let mut password = ProtectedPassword::from_str("bar");
        password.apply_change(&Change::Insert {
            index: 0,
            string: "foo".into(),
            protected: false,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.protected_graphemes(), vec![false; 6]);

        // At end
        let mut password = ProtectedPassword::from_str("foo");
        password.apply_change(&Change::Insert {
            index: 3,
            string: "bar".into(),
            protected: false,
        });
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.protected_graphemes(), vec![false; 6]);
    }

    #[test]
    fn remove() {
        let mut password = ProtectedPassword::from_str("foo");
        password.apply_change(&Change::Remove {
            index: 1,
            ignore_protection: false,
        });
        assert_eq!(password.as_str(), "fo");
        assert_eq!(password.protected_graphemes(), vec![false, false]);

        let mut password = ProtectedPassword::from_str("foo");
        password.protected_graphemes[1] = true;
        password.apply_change(&Change::Remove {
            index: 0,
            ignore_protection: false,
        });
        assert_eq!(password.as_str(), "oo");
        assert_eq!(password.protected_graphemes(), vec![true, false]);

        // With unicode in the string
        let mut password = ProtectedPassword::from_str("🏋️‍♂️a");
        password.apply_change(&Change::Remove {
            index: 1,
            ignore_protection: false,
        });
        assert_eq!(password.as_str(), "🏋️‍♂️");
        assert_eq!(password.protected_graphemes(), vec![false]);
    }

    #[test]
    fn replace() {
        let mut password = ProtectedPassword::from_str("foo");
        password.apply_change(&Change::Replace {
            index: 0,
            new_grapheme: "b".into(),
            ignore_protection: false,
        });
        assert_eq!(password.as_str(), "boo");
        assert_eq!(password.protected_graphemes(), vec![false, false, false]);

        let mut password = ProtectedPassword::new(Password::from_str("foo"));
        password.protected_graphemes[1] = true;
        password.apply_change(&Change::Replace {
            index: 0,
            new_grapheme: "b".into(),
            ignore_protection: false,
        });
        assert_eq!(password.as_str(), "boo");
        assert_eq!(password.protected_graphemes(), vec![false, true, false]);

        // With unicode in the string
        let mut password = ProtectedPassword::new(Password::from_str("🏋️‍♂️a"));
        password.apply_change(&Change::Replace {
            index: 1,
            new_grapheme: "b".into(),
            ignore_protection: false,
        });
        assert_eq!(password.as_str(), "🏋️‍♂️b");
        assert_eq!(password.protected_graphemes(), vec![false, false]);
    }

    #[test]
    #[should_panic]
    fn remove_protected_direct() {
        let mut password = ProtectedPassword::from_str("foo");
        password.protect(0);
        password.apply_change(&Change::Remove {
            index: 0,
            ignore_protection: false,
        });
    }

    #[test]
    #[should_panic]
    fn replace_protected_direct() {
        let mut password = ProtectedPassword::from_str("foo");
        password.protect(0);
        password.apply_change(&Change::Replace {
            index: 0,
            new_grapheme: "b".into(),
            ignore_protection: false,
        });
    }
}
