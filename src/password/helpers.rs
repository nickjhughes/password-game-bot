use lazy_regex::regex;
use numerals::roman::Roman;
use unicode_segmentation::UnicodeSegmentation;

/// Get all element symbols in a string, along with their grapheme index.
/// Two-letter symbols will be preferenced over single-letter symbols, if they are overlapping.
/// (e.g., "Fe" will result in "Fe", not "F")
pub fn get_elements(string: &str) -> Vec<(&periodic_table::Element, usize)> {
    let grapheme_indices = string.grapheme_indices(true).collect::<Vec<_>>();

    let mut elements = Vec::new();
    for element in periodic_table::periodic_table() {
        for (element_byte_index, _) in string.match_indices(element.symbol) {
            let grapheme_index = grapheme_indices
                .iter()
                .enumerate()
                .find_map(|(grapheme_index, (byte_index, _))| {
                    if *byte_index == element_byte_index {
                        Some(grapheme_index)
                    } else {
                        None
                    }
                })
                .unwrap();
            elements.push((*element, grapheme_index));
        }
    }

    // Remove overlapping results (e.g., "Fe" resulting in both "F" and "Fe")
    elements.sort_by(|a, b| a.0.symbol.len().cmp(&b.0.symbol.len()).reverse());
    let mut indices = Vec::new();
    let mut duplicates = Vec::new();
    for (j, (_, i)) in elements.iter().enumerate() {
        if indices.contains(i) {
            duplicates.push(j);
        } else {
            indices.push(*i);
        }
    }

    duplicates.sort();
    duplicates.reverse();
    for j in duplicates {
        elements.remove(j);
    }

    elements.sort_by(|a, b| a.1.cmp(&b.1));
    elements
}

/// Get all single digits in a string, along with their grapheme index.
pub fn get_digits(string: &str) -> Vec<(u32, usize)> {
    string
        .graphemes(true)
        .enumerate()
        .filter_map(|(i, g)| {
            if g.len() == 1 {
                let ch = g.chars().next().unwrap();
                if ch.is_ascii_digit() {
                    return Some((ch.to_digit(10).unwrap(), i));
                }
            }
            None
        })
        .collect::<Vec<(u32, usize)>>()
}

/// Get all alphabetic letters in a string (A..=Z | a..=z), along with their grapheme index.
pub fn get_letters(string: &str) -> Vec<(char, usize)> {
    string
        .graphemes(true)
        .enumerate()
        .filter_map(|(i, g)| {
            let ch = g.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                Some((ch, i))
            } else {
                None
            }
        })
        .collect::<Vec<(char, usize)>>()
}

/// Get all roman numerals in the string, converted to decimal, along with
/// their grapheme index and length.
pub fn get_roman_numerals(string: &str) -> Vec<(u64, usize, usize)> {
    let grapheme_indices = string.grapheme_indices(true).collect::<Vec<_>>();

    let re = regex!(r"M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})");
    re.captures_iter(string)
        .filter_map(|c| {
            let m = c.get(0).unwrap();
            let s = m.as_str();
            if s.is_empty() {
                return None;
            }
            let number = Roman::parse(s).unwrap().value() as u64;
            // Convert byte index to a grapheme index
            let grapheme_index = grapheme_indices
                .iter()
                .enumerate()
                .find_map(|(grapheme_index, (byte_index, _))| {
                    if *byte_index == m.start() {
                        Some(grapheme_index)
                    } else {
                        None
                    }
                })
                .unwrap();
            Some((number, grapheme_index, m.end() - m.start()))
        })
        .collect::<Vec<(u64, usize, usize)>>()
}

/// Get the ID of the first valid YouTube video URL in the given string,
/// or None if there are none. "youtube.com" URLs are preferences over
/// "youtu.be" URLs.
pub fn get_youtube_id(string: &str) -> Option<String> {
    let re1 = regex!(r"youtube\.com/watch\?v=(.{11})");
    let re2 = regex!(r"youtu\.be/(.{11})");

    if let Some(captures) = re1.captures(string) {
        Some(captures.get(1).unwrap().as_str().to_owned())
    } else if let Some(captures) = re2.captures(string) {
        Some(captures.get(1).unwrap().as_str().to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{get_digits, get_elements, get_roman_numerals, get_youtube_id};

    #[test]
    fn elements() {
        assert_eq!(
            get_elements("He")
                .iter()
                .map(|(e, i)| (e.symbol, *i))
                .collect::<Vec<_>>(),
            vec![("He", 0)]
        );
        assert_eq!(
            get_elements("FooBar")
                .iter()
                .map(|(e, i)| (e.symbol, *i))
                .collect::<Vec<_>>(),
            vec![("F", 0), ("Ba", 3)]
        );
    }

    #[test]
    fn digits() {
        assert_eq!(get_digits("foo10"), vec![(1, 3), (0, 4)]);
    }

    #[test]
    fn roman_numerals() {
        assert_eq!(get_roman_numerals("D"), vec![(500, 0, 1)]);
        assert_eq!(get_roman_numerals("😀VIIX"), vec![(7, 1, 3), (10, 4, 1)]);
        assert!(get_roman_numerals("i").is_empty());
    }

    #[test]
    fn youtube_id() {
        assert_eq!(
            get_youtube_id("youtube.com/watch?v=Hc6J5rlKhIc"),
            Some("Hc6J5rlKhIc".into())
        );
        assert_eq!(
            get_youtube_id("youtu.be/Hc6J5rlKhIc"),
            Some("Hc6J5rlKhIc".into())
        );
        assert_eq!(get_youtube_id("Hc6J5rlKhIc"), None);
    }
}
