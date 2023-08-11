use ego_tree::iter::Edge;
use lazy_regex::regex;
use lightningcss::{
    properties::{font, Property, PropertyId},
    stylesheet::ParserOptions,
    traits::Parse,
    values::{length, percentage},
};
use scraper::{Html, Node, Selector};
use svg::parser::Event;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    game::rule::Color,
    password::{format, Format},
};

/// Parse formatting from raw HTML.
pub fn parse_formatting(html: &str) -> Vec<Format> {
    let fragment = Html::parse_fragment(html);
    let p = fragment
        .select(&Selector::parse("p").unwrap())
        .next()
        .unwrap();
    // let password = p.text().collect::<Vec<_>>().join("");

    let mut current_format = Format::default();
    let mut formatting = Vec::new();
    for edge in p.traverse() {
        match edge {
            Edge::Open(node) => match node.value() {
                Node::Element(e) => match e.name() {
                    "span" => {
                        if let Some(style) = e.attr("style") {
                            for part in style.split(';') {
                                if part.trim().is_empty() {
                                    continue;
                                }
                                let (property_id_str, property_str) =
                                    part.split_once(':').unwrap_or_else(|| {
                                        panic!("style property should contain a `:`: {:?}", part)
                                    });
                                let property_id =
                                    PropertyId::parse_string(property_id_str).unwrap();
                                let property = Property::parse_string(
                                    property_id,
                                    property_str,
                                    ParserOptions::default(),
                                )
                                .unwrap();
                                match property {
                                    Property::FontFamily(font_families) => {
                                        match font_families.first().unwrap() {
                                            font::FontFamily::Generic(generic) => match generic {
                                                font::GenericFontFamily::Monospace => {
                                                    current_format.font_family =
                                                        format::FontFamily::Monospace;
                                                }
                                                f => panic!("unexpected font {:?}", f),
                                            },
                                            font::FontFamily::FamilyName(name) => {
                                                match name.to_string().as_str() {
                                                    "Comic Sans" => {
                                                        current_format.font_family =
                                                            format::FontFamily::ComicSans;
                                                    }
                                                    "Wingdings" => {
                                                        current_format.font_family =
                                                            format::FontFamily::Wingdings;
                                                    }
                                                    "Times New Roman" => {
                                                        current_format.font_family =
                                                            format::FontFamily::TimesNewRoman;
                                                    }
                                                    f => panic!("unexpected font {:?}", f),
                                                }
                                            }
                                        }
                                    }
                                    Property::FontSize(font_size) => match font_size {
                                        font::FontSize::Length(l) => match l {
                                            percentage::DimensionPercentage::Dimension(d) => {
                                                match d {
                                                    length::LengthValue::Px(px) => {
                                                        match format::FontSize::try_from(px as u32)
                                                        {
                                                            Ok(s) => current_format.font_size = s,
                                                            Err(_) => {
                                                                panic!("invalid font size {:?}", px)
                                                            }
                                                        }
                                                    }
                                                    d => panic!("unexpected font size {:?}", d),
                                                }
                                            }
                                            l => panic!("unexpected font size {:?}", l),
                                        },
                                        s => panic!("unexpected font size {:?}", s),
                                    },
                                    p => {
                                        panic!("unexpected css property {:?}", p)
                                    }
                                }
                            }
                        }
                    }
                    "strong" => {
                        current_format.bold = true;
                    }
                    "em" => {
                        current_format.italic = true;
                    }
                    "p" => {}
                    e => {
                        panic!("unexpected element {:?}", e);
                    }
                },
                Node::Text(t) => {
                    for g in t.graphemes(true) {
                        if g != "🐛" {
                            formatting.push(current_format.clone());
                        }
                    }
                }
                n => {
                    panic!("unexpected node {:?}", n)
                }
            },
            Edge::Close(node) => match node.value() {
                Node::Element(e) => match e.name() {
                    "span" => {
                        if let Some(style) = e.attr("style") {
                            for part in style.split(';') {
                                if part.trim().is_empty() {
                                    continue;
                                }
                                let (property_id_str, property_str) = part.split_once(':').unwrap();
                                let property_id =
                                    PropertyId::parse_string(property_id_str).unwrap();
                                let property = Property::parse_string(
                                    property_id,
                                    property_str,
                                    ParserOptions::default(),
                                )
                                .unwrap();
                                match property {
                                    Property::FontFamily(_) => {
                                        current_format.font_family = format::FontFamily::default();
                                    }
                                    Property::FontSize(_) => {
                                        current_format.font_size = format::FontSize::default();
                                    }
                                    p => {
                                        panic!("unexpected css property {:?}", p)
                                    }
                                }
                            }
                        }
                    }
                    "strong" => {
                        current_format.bold = false;
                    }
                    "em" => {
                        current_format.italic = false;
                    }
                    "p" => {}
                    e => {
                        panic!("unexpected element {:?}", e);
                    }
                },
                Node::Text(_) => {}
                n => {
                    panic!("unexpected node {:?}", n)
                }
            },
        }
    }
    formatting
}

/// Extract chess FEN from chess puzzle SVG.
pub fn extract_fen_from_svg(svg_contents: &str, turn: char) -> String {
    let mut in_pre = false;
    let mut pre = None;
    for event in svg::read(svg_contents).unwrap() {
        match event {
            Event::Tag(path, tag_type, _) => {
                if path == "pre" {
                    match tag_type {
                        svg::node::element::tag::Type::Start => in_pre = true,
                        svg::node::element::tag::Type::End => break,
                        _ => {}
                    }
                }
            }
            Event::Text(text) => {
                if in_pre {
                    pre = Some(text);
                }
            }
            _ => {}
        }
    }
    let pre = pre.unwrap();

    let mut fen = String::new();
    for rank in pre.lines() {
        let mut spaces = 0;
        let files = rank.split_ascii_whitespace();
        for file in files {
            let piece = file.chars().next().unwrap();
            if piece.is_ascii_lowercase() || piece.is_ascii_uppercase() {
                // piece
                if spaces > 0 {
                    fen.push_str(&spaces.to_string());
                }
                spaces = 0;

                fen.push(piece);
            } else {
                // empty square
                spaces += 1;
            }
        }
        if spaces > 0 {
            fen.push_str(&spaces.to_string());
        }
        if fen.chars().filter(|c| *c == '/').count() < 7 {
            fen.push('/');
        }
    }

    fen.push(' ');
    fen.push(turn);
    fen.push_str(" - - 0 1");

    fen
}

/// Get RGB color from CSS style.
pub fn extract_color_from_css_style(style: &str) -> Color {
    let re = regex!(r"rgb\((\d+),\s*(\d+),\s*(\d+)\)");
    let captures = re.captures(style).unwrap();
    Color {
        r: captures.get(1).unwrap().as_str().parse::<u8>().unwrap(),
        g: captures.get(2).unwrap().as_str().parse::<u8>().unwrap(),
        b: captures.get(3).unwrap().as_str().parse::<u8>().unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_fen_from_svg, parse_formatting};
    use crate::password::Format;

    #[test]
    fn formatting() {
        let html = "<div contenteditable=\"true\" translate=\"no\" class=\"ProseMirror ProseMirror-focused\" tabindex=\"0\"><p><span style=\"font-family: Monospace; font-size: 28px\">🥚b<strong>a</strong>n<strong>ua</strong>g🏋\u{fe0f}\u{200d}♂\u{fe0f}c<strong>a</strong></span></p></div>";
        let formatting = parse_formatting(html);
        assert_eq!(
            formatting,
            vec![
                Format::default(),
                Format::default(),
                Format::bold(),
                Format::default(),
                Format::bold(),
                Format::bold(),
                Format::default(),
                Format::default(),
                Format::default(),
                Format::bold(),
            ]
        );
    }

    #[test]
    fn extract_fen() {
        let svg_contents = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.2" baseProfile="tiny" viewBox="0 0 390 390"><desc><pre>r . b . . k . r
            p p p . b p p p
            . . . . . . . .
            . B . Q . . . .
            . . . . . q . .
            . . P . . . . .
            P P P . . P P P
            R . . . R . K .</pre></desc></svg>"#;
        assert_eq!(
            extract_fen_from_svg(svg_contents, 'w'),
            "r1b2k1r/ppp1bppp/8/1B1Q4/5q2/2P5/PPP2PPP/R3R1K1 w - - 0 1"
        );
    }
}
