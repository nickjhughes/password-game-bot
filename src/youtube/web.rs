use base64::{engine::general_purpose, Engine as _};
use log::warn;
use serde::{Deserialize, Serialize};

const WEB_API_URL: &str =
    "https://www.youtube.com/youtubei/v1/search?key=AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

#[derive(Debug, Serialize)]
struct Request {
    context: Context,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation: Option<String>,
}

#[derive(Debug, Serialize)]
struct Context {
    client: Client,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Client {
    client_name: String,
    client_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    contents: Option<Contents>,
    on_response_received_commands: Option<Vec<OnResponeReceivedCommand>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OnResponeReceivedCommand {
    append_continuation_items_action: Option<AppendContinuationItemsAction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppendContinuationItemsAction {
    continuation_items: Vec<Content>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contents {
    two_column_search_results_renderer: TwoColumnSearchResultsRenderer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TwoColumnSearchResultsRenderer {
    primary_contents: PrimaryContents,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrimaryContents {
    section_list_renderer: SectionListRenderer,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SectionListRenderer {
    contents: Vec<Content>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Content {
    ItemSectionRenderer {
        contents: Vec<ItemContent>,
    },
    #[serde(rename_all = "camelCase")]
    ContinuationItemRenderer {
        continuation_endpoint: ContinuationEndpoint,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContinuationEndpoint {
    continuation_command: ContinuationCommand,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContinuationCommand {
    token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
enum ItemContent {
    MovieRenderer {},
    AdSlotRenderer {},
    SearchPyvRenderer {},
    ReelShelfRenderer {},
    ShelfRenderer {},
    MessageRenderer {},
    #[serde(rename_all = "camelCase")]
    VideoRenderer {
        video_id: String,
        length_text: Option<LengthText>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LengthText {
    simple_text: String,
}

use crate::{Video, VideoDuration};

fn parse_length_text(text: &str) -> u32 {
    let mut parts = text.split(':');
    parts.next().unwrap().parse::<u32>().unwrap() * 60
        + parts.next().unwrap().parse::<u32>().unwrap()
}

/// Search for videos in the given duration range.
pub fn search(
    duration: VideoDuration,
    continuation_token: &Option<String>,
    query: &str,
) -> (Vec<Video>, Option<String>) {
    let body = if let Some(continuation_token) = continuation_token {
        Request {
            context: Context {
                client: Client {
                    client_name: "WEB".into(),
                    client_version: "2.20201211.09.00".into(),
                },
            },
            query: None,
            params: None,
            continuation: Some(continuation_token.to_owned()),
        }
    } else {
        let param_bytes = vec![
            0x12,
            0x04,
            0x10, // result type
            0x01, // video
            duration.to_web_api_param_type(),
            duration.to_web_api_param_value(),
        ];
        let params: String = general_purpose::STANDARD.encode(param_bytes);
        Request {
            context: Context {
                client: Client {
                    client_name: "WEB".into(),
                    client_version: "2.20201211.09.00".into(),
                },
            },
            query: Some(query.to_owned()),
            params: Some(urlencoding::encode(&params).to_string()),
            continuation: None,
        }
    };
    let body_string = serde_json::to_string(&body).unwrap();

    let client = reqwest::blocking::Client::new();
    let resp = client.post(WEB_API_URL).body(body_string).send().unwrap();
    let data = resp.text().unwrap();

    let resp: Response = serde_json::from_str(&data).unwrap();

    let mut continuation_token = None;
    let mut videos = Vec::new();
    let items: &[Content] = if resp.contents.is_some() {
        resp.contents
            .as_ref()
            .unwrap()
            .two_column_search_results_renderer
            .primary_contents
            .section_list_renderer
            .contents
            .as_ref()
    } else if resp.on_response_received_commands.is_some() {
        resp.on_response_received_commands.as_ref().unwrap()[0]
            .append_continuation_items_action
            .as_ref()
            .unwrap()
            .continuation_items
            .as_ref()
    } else {
        warn!("No contents or continuation...");
        return (Vec::new(), None);
    };
    for item in items {
        match item {
            Content::ItemSectionRenderer { contents } => {
                for item in contents {
                    if let ItemContent::VideoRenderer {
                        video_id,
                        length_text: Some(length_text),
                    } = item
                    {
                        let duration = parse_length_text(&length_text.simple_text);
                        videos.push(Video {
                            id: video_id.to_owned(),
                            duration,
                        });
                    }
                }
            }
            Content::ContinuationItemRenderer {
                continuation_endpoint,
            } => {
                continuation_token = Some(continuation_endpoint.continuation_command.token.clone());
            }
        }
    }

    (videos, continuation_token)
}
