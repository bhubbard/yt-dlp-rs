use crate::error::{Result, YtDlpError};
use crate::extractors::Extractor;
use crate::types::{Format, MediaMetadata, Protocol};
use crate::utils::match_group;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use serde_json::Value;

#[derive(Default)]
pub struct TwitterExtractor;

#[async_trait]
impl Extractor for TwitterExtractor {
    fn id(&self) -> &'static str {
        "twitter"
    }

    fn suitable(&self, url: &str) -> bool {
        url.contains("twitter.com") || url.contains("x.com")
    }

    async fn extract(&self, url: &str, client: &Client) -> Result<MediaMetadata> {
        let tweet_id = match_group(url, r"(?:status|statuses)/(\d+)")
            .ok_or_else(|| YtDlpError::Extraction("Could not extract Tweet ID".to_string()))?;

        // Query Twitter syndication endpoint
        let api_url = format!(
            "https://cdn.syndication.twimg.com/tweet-result?id={}&lang=en",
            tweet_id
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            ),
        );

        let resp = client.get(&api_url).headers(headers).send().await?;
        let data: Value = resp.json().await?;

        let text = data["text"].as_str().unwrap_or("Tweet Video").to_string();
        let uploader = data["user"]["name"].as_str().map(|s| s.to_string());

        let mut meta = MediaMetadata::new(&tweet_id, text, url, "twitter");
        meta.uploader = uploader;

        let mut formats = Vec::new();

        if let Some(media_entities) = data["mediaDetails"].as_array() {
            for m in media_entities {
                if let Some(variants) = m["video_info"]["variants"].as_array() {
                    for v in variants {
                        if let Some(v_url) = v["url"].as_str() {
                            let content_type = v["content_type"].as_str().unwrap_or("");
                            let bitrate = v["bitrate"].as_u64();

                            if content_type == "application/x-mpegURL" {
                                let mut fmt = Format::new("hls", "mp4", v_url);
                                fmt.protocol = Protocol::M3u8;
                                formats.push(fmt);
                            } else if content_type == "video/mp4" {
                                let br = bitrate.unwrap_or(0);
                                let fid = format!("http-{}", br / 1000);
                                let mut fmt = Format::new(&fid, "mp4", v_url);
                                fmt.tbr = Some(br as f64 / 1000.0);
                                formats.push(fmt);
                            }
                        }
                    }
                }
            }
        }

        if formats.is_empty() {
            return Err(YtDlpError::Extraction(format!(
                "No video streams found for Tweet {}",
                tweet_id
            )));
        }

        meta.formats = formats;
        Ok(meta)
    }
}
