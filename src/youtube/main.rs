use log::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs};

#[allow(dead_code)]
mod api;
mod web;

const MIN_DURATION: u32 = 180;
const MAX_DURATION: u32 = 2180;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum VideoDuration {
    Any,
    /// 20:01..
    Long,
    /// 4:00..=20:00
    Medium,
    /// 0:01..=3:59
    Short,
}

impl VideoDuration {
    pub fn to_web_api_param_type(&self) -> u8 {
        0x18
    }

    pub fn to_web_api_param_value(&self) -> u8 {
        match self {
            VideoDuration::Any => 0x00,
            VideoDuration::Long => 0x02,
            VideoDuration::Medium => 0x03,
            VideoDuration::Short => 0x01,
        }
    }

    pub fn min_duration(&self) -> u32 {
        match self {
            VideoDuration::Any => MIN_DURATION,
            VideoDuration::Long => 20 * 60 + 1,
            VideoDuration::Medium => 4 * 60,
            VideoDuration::Short => MIN_DURATION,
        }
    }

    pub fn max_duration(&self) -> u32 {
        match self {
            VideoDuration::Any => MAX_DURATION,
            VideoDuration::Long => MAX_DURATION,
            VideoDuration::Medium => 20 * 60,
            VideoDuration::Short => 4 * 60 - 1,
        }
    }

    pub fn count(&self) -> usize {
        self.max_duration() as usize - self.min_duration() as usize + 1
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Video {
    id: String,
    /// Duration in seconds
    duration: u32,
}

/// Sum the single digits in the given string.
fn digit_sum(id: &str) -> u32 {
    let mut sum = 0;
    for ch in id.chars() {
        if ch.is_ascii_digit() {
            sum += ch.to_string().parse::<u32>().unwrap();
        }
    }
    sum
}

/// Count the number of non-"I" roman numeral digits in the given string.
fn roman_digit_count(id: &str) -> usize {
    id.chars()
        .filter(|ch| {
            *ch == 'V' || *ch == 'X' || *ch == 'L' || *ch == 'C' || *ch == 'D' || *ch == 'M'
        })
        .count()
}

/// Determine whether the ID is fully useful (i.e., doesn't contain roman numerals or non-zero
/// digits).
fn is_id_perfect(id: &str) -> bool {
    let mut is_valid = true;
    for ch in id.chars() {
        if ch.is_ascii_digit() && ch != '0' {
            is_valid = false;
            break;
        }
        if ch == 'V' || ch == 'X' || ch == 'L' || ch == 'C' || ch == 'D' || ch == 'M' {
            is_valid = false;
            break;
        }
    }
    is_valid
}

fn check_videos(videos: &[Video]) {
    let mut durations = HashSet::new();
    for video in videos {
        if !durations.insert(video.duration) {
            panic!("duplicate duration {:?} in videos.json", video.duration);
        }
    }
}

fn load_videos() -> Vec<Video> {
    if let Ok(contents) = fs::read_to_string("src/youtube/videos.json") {
        let videos: Vec<Video> = serde_json::from_str(&contents).unwrap();
        check_videos(&videos);
        videos
    } else {
        // File doesn't exist yet, return empty vector
        Vec::new()
    }
}

fn print_videos_summary(videos: &[Video], duration: VideoDuration) {
    let count = videos
        .iter()
        .filter(|v| v.duration >= duration.min_duration() && v.duration <= duration.max_duration())
        .count();
    let prop = count as f32 / duration.count() as f32;
    let perfect_count = videos
        .iter()
        .filter(|v| {
            v.duration >= duration.min_duration()
                && v.duration <= duration.max_duration()
                && is_id_perfect(&v.id)
        })
        .count();
    let perfect_prop = perfect_count as f32 / count as f32;
    info!(
        "Summary ({:?}): Covered {} of {} durations ({:.1}%), {} ({:.1}%) of which are perfect",
        duration,
        count,
        duration.count(),
        prop * 100.0,
        perfect_count,
        perfect_prop * 100.0
    );
}

fn save_videos(videos: &[Video], duration: VideoDuration) {
    let f = fs::File::create("src/youtube/videos.json").expect("failed to open videos.json");
    serde_json::to_writer(f, videos).expect("failed to write to videos.json");
    print_videos_summary(videos, duration);
}

fn update_videos(videos: &mut Vec<Video>, new_videos: &[Video]) {
    let mut new_count = 0;
    let mut update_count = 0;
    for new_video in new_videos {
        if new_video.duration < MIN_DURATION || new_video.duration > MAX_DURATION {
            continue;
        }
        if videos.iter().any(|v| v.id == new_video.id) {
            // Duplicate ID
            continue;
        }
        if videos.iter().any(|v| {
            if v.duration == new_video.duration {
                // Duplicate duration
                // Only include if fewer non-"I"" roman numeral digits & non-zero digit sum
                if digit_sum(&new_video.id) <= digit_sum(&v.id)
                    && roman_digit_count(&new_video.id) <= roman_digit_count(&v.id)
                {
                    // Duplicate duration with a better ID
                    false
                } else {
                    // Duplicate duration, but not a better ID
                    true
                }
            } else {
                // New duration
                false
            }
        }) {
            continue;
        }
        // Remove any videos with the same duration, incase we're replacing with a better ID
        if videos.iter().any(|v| v.duration == new_video.duration) {
            update_count += 1;
        } else {
            new_count += 1;
        }
        videos.retain(|v| v.duration != new_video.duration);
        videos.push(new_video.clone());
    }
    info!("{} new durations, {} better IDs", new_count, update_count);
    check_videos(videos);
}

#[allow(dead_code)]
fn use_api(duration: VideoDuration) {
    let mut nouns = fs::read_to_string("src/youtube/top-1000-nouns.txt")
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_owned())
        .collect::<Vec<String>>();
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    nouns.shuffle(&mut thread_rng());
    let mut nouns_iter = nouns.iter();

    let api_key = api::get_api_key();
    let mut page_token = None;
    let mut query = nouns_iter.next().unwrap();
    let mut videos = load_videos();
    info!("Loaded {} videos from file", videos.len());

    while videos.len() < 60 {
        let (results_ids, next_page_token) =
            api::search(&api_key, duration.clone(), &page_token, query);
        if !results_ids.is_empty() {
            let new_videos = api::get_video_durations(&api_key, &results_ids);
            update_videos(&mut videos, &new_videos);
            save_videos(&videos, duration.clone());
            info!("Saved {} videos to file", videos.len());
        }
        if next_page_token.is_some() {
            page_token = next_page_token;
        } else {
            // No more pages, change query
            query = nouns_iter.next().expect("out of nouns");
            page_token = None;
        }
    }
}

fn use_web_api(duration: VideoDuration) {
    let mut nouns = fs::read_to_string("src/youtube/top-1000-nouns.txt")
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_owned())
        .collect::<Vec<String>>();
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    nouns.shuffle(&mut thread_rng());
    let mut nouns_iter = nouns.iter();

    let mut continuation_token = None;
    let mut query = nouns_iter.next().unwrap();
    info!("New query: {:?}", query);
    let mut videos = load_videos();
    info!("Loaded {} videos from file", videos.len());

    let mut query_request_count = 0;
    while videos.len() < (MAX_DURATION - MIN_DURATION + 1) as usize {
        let (new_videos, next_continuation_token) =
            web::search(duration.clone(), &continuation_token, query);
        query_request_count += 1;
        update_videos(&mut videos, &new_videos);
        save_videos(&videos, duration.clone());

        if next_continuation_token.is_some() && query_request_count < 10 {
            continuation_token = next_continuation_token;
        } else {
            // No more pages, change query
            query = nouns_iter.next().expect("out of nouns");
            query_request_count = 0;
            continuation_token = None;
            info!("New query: {:?}", query);
        }
    }
}

#[allow(dead_code)]
fn delete_non_embeddable() {
    let api_key = api::get_api_key();
    let videos = load_videos();
    info!("Loaded {} videos from file", videos.len());

    let mut embeddable_videos = Vec::new();
    for chunk in videos.chunks(50) {
        let embeddable = api::get_embeddable(
            &api_key,
            &chunk
                .iter()
                .map(|v| v.id.to_owned())
                .collect::<Vec<String>>(),
        );
        for (video, is_embeddable) in chunk.iter().zip(embeddable.iter()) {
            if *is_embeddable {
                embeddable_videos.push(video.clone());
            } else {
                info!("Removing un-embeddeable video {}", video.id);
            }
        }
    }

    save_videos(&embeddable_videos, VideoDuration::Any);
}

fn main() {
    env_logger::try_init().unwrap_or(());
    use_web_api(VideoDuration::Long);
    // delete_non_embeddable();
}
