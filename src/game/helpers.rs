use cached::proc_macro::cached;
use chrono::prelude::*;
use iso8601_duration::Duration;
use isocountry::CountryCode;
use ordered_float::NotNan;
use pleco::{bots::JamboreeSearcher, tools::Searcher, BitMove, Board};
use reverse_geocoder::{Locations, ReverseGeocoder};
use scraper::{Html, Selector};
use suncalc::{moon_illumination, Timestamp};

use super::rule::MoonPhase;

/// Get today's Wordle answer from neal.fun API for the given date.
#[cached]
pub fn get_wordle_answer(date: NaiveDate) -> String {
    let url = format!(
        "https://neal.fun/api/password-game/wordle?date={}",
        date.format("%Y-%m-%d")
    );
    let body = reqwest::blocking::get(url).unwrap().text().unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
    json["answer"].to_string().trim_matches('"').to_owned()
}

/// Get the phase of the moon on the given date.
#[cached]
pub fn get_moon_phase(datetime: DateTime<Local>) -> MoonPhase {
    let datetime = datetime
        .with_timezone(&chrono_tz::US::Eastern)
        .with_hour(0)
        .unwrap();
    let today = datetime.timestamp_millis();
    let tomorrow = today + 24 * 60 * 60 * 1000;
    let phase_today = moon_illumination(Timestamp(today)).phase;
    let phase_tomorrow = moon_illumination(Timestamp(tomorrow)).phase;

    if phase_today <= 0.25 && phase_tomorrow >= 0.25 {
        MoonPhase::FirstQuarter
    } else if phase_today <= 0.5 && phase_tomorrow >= 0.5 {
        MoonPhase::Full
    } else if phase_today <= 0.75 && phase_tomorrow >= 0.75 {
        MoonPhase::LastQuarter
    } else if phase_today >= phase_tomorrow {
        MoonPhase::New
    } else if phase_today <= 0.25 {
        MoonPhase::WaxingCrescent
    } else if phase_today <= 0.5 {
        MoonPhase::WaxingGibbous
    } else if phase_today <= 0.75 {
        MoonPhase::WaningGibbous
    } else {
        MoonPhase::WaningCrescent
    }
}

/// Check if a number is prime.
#[cached]
pub fn is_prime(n: usize) -> bool {
    if n <= 1 {
        return false;
    }
    let limit = (n as f64).sqrt() as usize;
    for i in 2..=limit {
        if n % i == 0 {
            return false;
        }
    }
    true
}

/// Convert a pleco::BitMove into standard algebraic notation (SAN).
/// Note that this function only supports a subset of SAN, enough to cover all the
/// solution moves to puzzles in the password game.
fn bitmove_to_san(mut board: Board, bit_move: BitMove) -> String {
    let dest_square = bit_move.get_dest().to_string();
    let piece = board
        .piece_at_sq(bit_move.get_src())
        .to_string()
        .to_ascii_uppercase();
    let capture = if bit_move.is_capture() { "x" } else { "" };
    board.apply_move(bit_move);
    let check = if board.in_check() { "+" } else { "" };
    format!(
        "{}{}{}{}",
        if piece == "P" { "" } else { &piece },
        capture,
        dest_square,
        check
    )
}

/// Get the optimal move in algebraic notation for the given position.
#[cached]
pub fn get_optimal_move(fen: String) -> String {
    let board = Board::from_fen(&fen).expect("failed to parse FEN");
    let optimal_move = JamboreeSearcher::best_move(board.clone(), 4);
    bitmove_to_san(board, optimal_move)
}

/// Locate the country of the given lat/long coordinate pair.
#[cached]
pub fn get_country_from_coordinates(lat: NotNan<f64>, long: NotNan<f64>) -> String {
    let locations = Locations::from_memory();
    let geocoder = ReverseGeocoder::new(&locations);
    let search_result = geocoder
        .search((lat.into_inner(), long.into_inner()))
        .expect("failed to search coordinates");
    let country_code = &search_result.record.cc;
    let country = CountryCode::for_alpha2(country_code).expect("failed to match country code");
    let country_name = country.name().to_ascii_lowercase();
    match country_name.as_str() {
        "russian federation" => "russia".into(),
        "venezuela (bolivarian republic of)" => "venezuela".into(),
        "iran (islamic republic of)" => "iran".into(),
        "holy see" => "italy".into(),
        _ => country_name,
    }
}

/// Get the duration of the given YouTube video in seconds.
#[cached]
pub fn get_youtube_duration(id: String) -> u32 {
    let url = format!("https://www.youtube.com/watch?v={}", id);
    let body = reqwest::blocking::get(&url).unwrap().text().unwrap();
    let document = Html::parse_document(&body);
    let selector = Selector::parse("meta").unwrap();
    for element in document.select(&selector) {
        if let Some(itemprop) = element.value().attr("itemprop") {
            if itemprop == "duration" {
                let duration_str = element.value().attr("content").unwrap();
                let duration = duration_str
                    .parse::<Duration>()
                    .unwrap()
                    .num_seconds()
                    .unwrap() as u32;
                return duration;
            }
        }
    }
    panic!("failed to get youtube video duration");
}

#[cfg(test)]
mod tests {
    use super::{get_optimal_move, get_youtube_duration};

    #[test]
    fn chess_puzzles() {
        let fen = "r1b2k1r/ppp1bppp/8/1B1Q4/5q2/2P5/PPP2PPP/R3R1K1 w - - 0 1";
        assert_eq!(get_optimal_move(fen.to_owned()), "Qd8+");

        let fen = "r2qrb2/p1pn1Qp1/1p4Nk/4PR2/3n4/7N/P5PP/R6K w - - 0 1";
        assert_eq!(get_optimal_move(fen.to_owned()), "Ne7");
    }

    #[test]
    #[ignore]
    fn youtube_duration() {
        assert_eq!(get_youtube_duration("Hc6J5rlKhIc".into()), 15);
    }
}
