use strum::{EnumCount, EnumIter};

use super::FormatChange;

/// Font size options.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, EnumCount)]
pub enum FontSize {
    #[default]
    Px28,
    Px32,
    Px36,
    Px42,
    Px49,
    Px64,
    Px81,
    Px0,
    Px1,
    Px4,
    Px9,
    Px12,
    Px16,
    Px25,
}

impl TryFrom<u32> for FontSize {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FontSize::Px0),
            1 => Ok(FontSize::Px1),
            4 => Ok(FontSize::Px4),
            9 => Ok(FontSize::Px9),
            12 => Ok(FontSize::Px12),
            16 => Ok(FontSize::Px16),
            25 => Ok(FontSize::Px25),
            28 => Ok(FontSize::Px28),
            32 => Ok(FontSize::Px32),
            36 => Ok(FontSize::Px36),
            42 => Ok(FontSize::Px42),
            49 => Ok(FontSize::Px49),
            64 => Ok(FontSize::Px64),
            81 => Ok(FontSize::Px81),
            _ => Err("invalid font size"),
        }
    }
}

impl FontSize {
    pub fn index(&self) -> usize {
        match self {
            FontSize::Px0 => 0,
            FontSize::Px1 => 1,
            FontSize::Px4 => 2,
            FontSize::Px9 => 3,
            FontSize::Px12 => 4,
            FontSize::Px16 => 5,
            FontSize::Px25 => 6,
            FontSize::Px28 => 7,
            FontSize::Px32 => 8,
            FontSize::Px36 => 9,
            FontSize::Px42 => 10,
            FontSize::Px49 => 11,
            FontSize::Px64 => 12,
            FontSize::Px81 => 13,
        }
    }
}

/// Font family options.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, EnumCount)]
pub enum FontFamily {
    #[default]
    Monospace,
    #[allow(dead_code)]
    ComicSans,
    Wingdings,
    TimesNewRoman,
}

impl FontFamily {
    pub fn index(&self) -> usize {
        match self {
            FontFamily::Monospace => 0,
            FontFamily::ComicSans => 1,
            FontFamily::Wingdings => 2,
            FontFamily::TimesNewRoman => 3,
        }
    }
}

/// Formatting properties of a grapheme cluster.
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Format {
    /// Bold.
    pub bold: bool,
    /// Italic.
    pub italic: bool,
    /// Font size.
    pub font_size: FontSize,
    /// Font family.
    pub font_family: FontFamily,
}

impl std::fmt::Debug for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let font_family = match self.font_family {
            FontFamily::Monospace => "M",
            FontFamily::ComicSans => "CS",
            FontFamily::Wingdings => "W",
            FontFamily::TimesNewRoman => "TNR",
        };
        f.write_fmt(format_args!(
            "{}-{}-{:?}-{}",
            if self.bold { "B" } else { "b" },
            if self.italic { "I" } else { "i" },
            self.font_size,
            font_family
        ))
    }
}

impl Format {
    pub fn change(&mut self, change: &FormatChange) {
        match change {
            FormatChange::BoldOn => self.bold = true,
            FormatChange::ItalicOn => self.italic = true,
            FormatChange::FontSize(font_size) => self.font_size = font_size.clone(),
            FormatChange::FontFamily(font_family) => self.font_family = font_family.clone(),
        }
    }

    #[cfg(test)]
    pub fn bold() -> Self {
        Format {
            bold: true,
            ..Default::default()
        }
    }
}
