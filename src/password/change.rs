use derivative::Derivative;

use super::format::{FontFamily, FontSize};

/// A modification to formatting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatChange {
    BoldOn,
    ItalicOn,
    FontSize(FontSize),
    FontFamily(FontFamily),
}

/// A modification to a password.
#[derive(Debug, Clone, Derivative)]
#[derivative(
    PartialEq,
    Eq,
    PartialOrd = "feature_allow_slow_enum",
    Ord = "feature_allow_slow_enum"
)]
pub enum Change {
    /// Format a single grapheme at the given index.
    Format {
        /// The index of the grapheme to format.
        index: usize,
        /// The formatting change to apply.
        format_change: FormatChange,
    },
    /// Prepend a string to the start of the password.
    Prepend {
        /// The string to prepend.
        string: String,
        /// Whether the new grapheme clusters as a result of the change should be
        /// considered protected.
        protected: bool,
    },
    /// Append a string to the end of the password.
    Append {
        /// The string to append.
        #[derivative(PartialOrd = "ignore", Ord = "ignore")]
        string: String,
        /// Whether the new grapheme clusters as a result of the change should be
        /// considered protected.
        #[derivative(PartialOrd = "ignore", Ord = "ignore")]
        protected: bool,
    },
    /// Insert a string at the given index.
    #[allow(dead_code)]
    Insert {
        /// The index where the string should be inserted.
        index: usize,
        /// The string to insert.
        string: String,
        /// Whether the new grapheme clusters as a result of the change should be
        /// considered protected.
        protected: bool,
    },
    /// Replace a single grapheme out for another one at the given index.
    Replace {
        /// The index of the grapheme to replace.
        index: usize,
        /// The new grapheme to insert.
        new_grapheme: String,
        /// Is it okay to replace a protected grapheme?
        ignore_protection: bool,
    },
    /// Remove a single grapheme at the given index from the password.
    Remove {
        /// The index of the grapheme to remove.
        index: usize,
        /// Is it okay to remove a protected grapheme?
        ignore_protection: bool,
    },
}
