use unicode_segmentation::UnicodeSegmentation;

pub use change::{Change, FormatChange};
pub use format::Format;
pub use mutable::MutablePassword;
pub use protected::ProtectedPassword;

mod change;
pub mod format;
pub mod helpers;
mod mutable;
mod protected;

/// A password with formatting. Conceptualised as a sequence of grapheme clusters.
#[derive(Debug, Default, Clone)]
pub struct Password {
    /// The current password.
    password: String,
    /// Formatting of each grapheme.
    /// The length of this Vec corresponds to `password.graphemes().count()`.
    formatting: Vec<Format>,
}

impl Password {
    /// Construct a new password from the given string. Assumes default formatting.
    #[cfg(test)]
    pub fn from_str(string: &str) -> Self {
        Password {
            password: string.to_owned(),
            formatting: vec![Format::default(); string.graphemes(true).count()],
        }
    }

    /// The length of the password in terms of grapheme clusters.
    pub fn len(&self) -> usize {
        self.password.graphemes(true).count()
    }

    /// The password as a string slice.
    pub fn as_str(&self) -> &str {
        self.password.as_str()
    }

    /// The formatting of each grapheme.
    pub fn formatting(&self) -> &[Format] {
        &self.formatting
    }

    /// Append a string to the password. Assumes default formatting.
    pub fn append(&mut self, string: &str) {
        self.password.push_str(string);
        for _ in 0..string.graphemes(true).count() {
            self.formatting.push(Format::default());
        }

        debug_assert_eq!(self.len(), self.formatting.len());
    }

    /// Prepend a string to the password. Assumes default formatting.
    pub fn prepend(&mut self, string: &str) {
        self.password.insert_str(0, string);
        for _ in 0..string.graphemes(true).count() {
            self.formatting.insert(0, Format::default());
        }

        debug_assert_eq!(self.len(), self.formatting.len());
    }

    /// Insert a string at the given index. Assumes default formatting.
    pub fn insert(&mut self, index: usize, string: &str) {
        if index == 0 {
            self.prepend(string);
            return;
        }
        if index == self.len() {
            self.append(string);
            return;
        }

        let byte_index = self.password.grapheme_indices(true).nth(index).unwrap().0;
        self.password.insert_str(byte_index, string);
        for _ in 0..string.graphemes(true).count() {
            self.formatting.insert(index, Format::default());
        }

        debug_assert_eq!(self.len(), self.formatting.len());
    }

    /// Remove the grapheme cluster at `index` from the password.
    pub fn remove(&mut self, index: usize) {
        self.formatting.remove(index);

        let grapheme_indices = self.password.grapheme_indices(true).collect::<Vec<_>>();
        let (byte_offset, grapheme) = grapheme_indices[index];
        let (left, right) = self.password.split_at(byte_offset);

        let mut new_password = left.to_string();
        new_password.push_str(&right[grapheme.len()..]);
        self.password = new_password;

        debug_assert_eq!(self.len(), self.formatting.len());
    }

    /// Replace the grapheme cluster at `index` with the one given. Formatting will stay the same.
    pub fn replace(&mut self, index: usize, replacement: &str) {
        let grapheme_indices = self.password.grapheme_indices(true).collect::<Vec<_>>();
        let (byte_offset, grapheme) = grapheme_indices[index];
        let (left, right) = self.password.split_at(byte_offset);

        let mut new_password = left.to_string();
        new_password.push_str(replacement);
        new_password.push_str(&right[grapheme.len()..]);
        self.password = new_password;

        debug_assert_eq!(self.len(), self.formatting.len());
    }

    /// Format the grapheme cluster at `index`.
    pub fn format(&mut self, index: usize, format_change: &FormatChange) {
        self.formatting[index].change(format_change);

        debug_assert_eq!(self.len(), self.formatting.len());
    }
}

#[cfg(test)]
mod tests {
    use super::{Format, FormatChange, Password};

    #[test]
    fn append() {
        let mut password = Password::from_str("foo");
        password.append("bar");
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.formatting(), vec![Format::default(); 6]);

        let mut password = Password::from_str("foo");
        for i in 0..3 {
            password.format(i, &FormatChange::BoldOn);
        }
        password.append("bar");
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(
            password.formatting(),
            vec![
                Format::bold(),
                Format::bold(),
                Format::bold(),
                Format::default(),
                Format::default(),
                Format::default()
            ]
        );
    }

    #[test]
    fn prepend() {
        let mut password = Password::from_str("bar");
        password.prepend("foo");
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.formatting(), vec![Format::default(); 6]);

        let mut password = Password::from_str("bar");
        for i in 0..3 {
            password.format(i, &FormatChange::BoldOn);
        }
        password.prepend("foo");
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(
            password.formatting(),
            vec![
                Format::default(),
                Format::default(),
                Format::default(),
                Format::bold(),
                Format::bold(),
                Format::bold(),
            ]
        );
    }

    #[test]
    fn insert() {
        let mut password = Password::from_str("for");
        password.insert(2, "oba");
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.formatting(), vec![Format::default(); 6]);

        // At start
        let mut password = Password::from_str("bar");
        password.insert(0, "foo");
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.formatting(), vec![Format::default(); 6]);

        // At end
        let mut password = Password::from_str("foo");
        password.insert(3, "bar");
        assert_eq!(password.as_str(), "foobar");
        assert_eq!(password.formatting(), vec![Format::default(); 6]);

        // With unicode in the string
        let mut password = Password::from_str("foo🏋️‍♂️r");
        password.insert(4, "ba");
        assert_eq!(password.as_str(), "foo🏋️‍♂️bar");
        assert_eq!(password.formatting(), vec![Format::default(); 7]);
    }

    #[test]
    fn remove() {
        let mut password = Password::from_str("foo");
        password.remove(1);
        assert_eq!(password.as_str(), "fo");
        assert_eq!(password.formatting(), vec![Format::default(); 2]);

        let mut password = Password::from_str("foo");
        password.formatting[1] = Format::bold();
        password.remove(0);
        assert_eq!(password.as_str(), "oo");
        assert_eq!(
            password.formatting(),
            vec![Format::bold(), Format::default()]
        );

        // With unicode in the string
        let mut password = Password::from_str("🏋️‍♂️a");
        password.remove(1);
        assert_eq!(password.as_str(), "🏋️‍♂️");
        assert_eq!(password.formatting(), vec![Format::default()]);
    }

    #[test]
    fn replace() {
        let mut password = Password::from_str("foo");
        password.replace(0, "b");
        assert_eq!(password.as_str(), "boo");
        assert_eq!(password.formatting, vec![Format::default(); 3]);

        let mut password = Password::from_str("foo");
        password.formatting[0] = Format::bold();
        password.replace(0, "b");
        assert_eq!(password.as_str(), "boo");
        assert_eq!(
            password.formatting(),
            vec![Format::bold(), Format::default(), Format::default()]
        );

        // With unicode in the string
        let mut password = Password::from_str("🏋️‍♂️a");
        password.replace(1, "b");
        assert_eq!(password.as_str(), "🏋️‍♂️b");
        assert_eq!(password.formatting(), vec![Format::default(); 2]);
    }

    #[test]
    fn format() {
        let mut password = Password::from_str("foo");
        password.format(1, &FormatChange::BoldOn);
        assert_eq!(password.as_str(), "foo");
        assert_eq!(
            password.formatting(),
            vec![Format::default(), Format::bold(), Format::default()]
        );
    }
}
