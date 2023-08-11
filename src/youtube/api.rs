use iso8601_duration::Duration;
use log::info;
use reqwest::StatusCode;
use serde::Deserialize;
use std::fs;

use crate::{is_id_perfect, Video, VideoDuration};

impl std::fmt::Display for VideoDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoDuration::Any => write!(f, "any"),
            VideoDuration::Long => write!(f, "long"),
            VideoDuration::Medium => write!(f, "medium"),
            VideoDuration::Short => write!(f, "short"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    next_page_token: Option<String>,
    items: Option<Vec<SearchItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchItem {
    id: Id,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Id {
    video_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideosResult {
    items: Option<Vec<VideosItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideosItem {
    id: String,
    content_details: Option<ContentDetails>,
    status: Option<Status>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentDetails {
    duration: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    embeddable: bool,
}

pub fn get_api_key() -> String {
    let contents =
        fs::read_to_string("src/youtube/api_key.txt").expect("failed to read api key file");
    contents.trim_end().to_string()
}

/// Search for videos in the given duration range.
pub fn search(
    api_key: &str,
    duration: VideoDuration,
    page_token: &Option<String>,
    query: &str,
) -> (Vec<String>, Option<String>) {
    let page_token_param = if let Some(page_token) = page_token {
        info!("Searching for {}, page token {}", query, page_token);
        format!("&pageToken={}", page_token)
    } else {
        info!("Searching for {}", query);
        "".into()
    };
    let url = format!("https://youtube.googleapis.com/youtube/v3/search?q={}&part=snippet&maxResults=50&type=video&videoDuration={}&key={}{}", query, duration, api_key, page_token_param);
    let resp = reqwest::blocking::get(url).unwrap();
    if resp.status() == StatusCode::FORBIDDEN {
        panic!("Out of quota :(");
    }
    let body = resp.text().unwrap();
    let results: SearchResult = serde_json::from_str(&body).unwrap();
    if results.items.is_none() {
        return (Vec::new(), results.next_page_token);
    }
    (
        results
            .items
            .unwrap()
            .iter()
            .filter(|v| is_id_perfect(&v.id.video_id))
            .map(|v| v.id.video_id.clone())
            .collect::<Vec<String>>(),
        results.next_page_token,
    )
}

/// Get the duration of each video in seconds.
pub fn get_video_durations(api_key: &str, video_ids: &[String]) -> Vec<Video> {
    if video_ids.is_empty() {
        return Vec::new();
    }
    let ids_str = video_ids
        .iter()
        .map(|id| format!("id={}", id))
        .collect::<Vec<String>>()
        .join("&");
    let url = format!(
        "https://youtube.googleapis.com/youtube/v3/videos?part=contentDetails&{}&key={}",
        ids_str, api_key
    );
    let resp = reqwest::blocking::get(url).unwrap();
    if resp.status() == StatusCode::FORBIDDEN {
        panic!("Out of quota :(");
    }
    let body = resp.text().unwrap();
    let results: VideosResult = serde_json::from_str(&body).unwrap();
    results
        .items
        .unwrap()
        .iter()
        .map(|item| {
            let duration = item
                .content_details
                .as_ref()
                .unwrap()
                .duration
                .parse::<Duration>()
                .unwrap()
                .num_seconds()
                .unwrap() as u32;
            Video {
                id: item.id.clone(),
                duration,
            }
        })
        .collect::<Vec<Video>>()
}

/// Check if the given videos can be embedded.
pub fn get_embeddable(api_key: &str, video_ids: &[String]) -> Vec<bool> {
    if video_ids.is_empty() {
        return Vec::new();
    }
    let ids_str = video_ids
        .iter()
        .map(|id| format!("id={}", id))
        .collect::<Vec<String>>()
        .join("&");
    let url = format!(
        "https://youtube.googleapis.com/youtube/v3/videos?part=status&{}&key={}",
        ids_str, api_key
    );
    let resp = reqwest::blocking::get(url).unwrap();
    if resp.status() == StatusCode::FORBIDDEN {
        panic!("Out of quota :(");
    }
    let body = resp.text().unwrap();
    let results: VideosResult = serde_json::from_str(&body).unwrap();
    results
        .items
        .unwrap()
        .iter()
        .map(|item| item.status.as_ref().unwrap().embeddable)
        .collect::<Vec<bool>>()
}
