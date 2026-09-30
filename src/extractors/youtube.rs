use crate::error::{Result, YtDlpError};
use crate::extractors::Extractor;
use crate::types::{Format, MediaMetadata, Protocol, Thumbnail};
use crate::utils::match_group;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use serde_json::Value;

#[derive(Default, Debug, Clone, Copy)]
pub struct YouTubeExtractor;

impl YouTubeExtractor {
    pub fn extract_video_id(url: &str) -> Option<String> {
        match_group(
            url,
            r"(?:v=|\/embed\/|\/watch\?v=|youtu\.be\/|\/shorts\/)([a-zA-Z0-9_-]{11})",
        )
    }
}

#[async_trait]
impl Extractor for YouTubeExtractor {
    fn id(&self) -> &'static str {
        "youtube"
    }

    fn suitable(&self, url: &str) -> bool {
        url.contains("youtube.com") || url.contains("youtu.be")
    }

    async fn extract(&self, url: &str, client: &Client) -> Result<MediaMetadata> {
        let video_id = Self::extract_video_id(url)
            .ok_or_else(|| YtDlpError::Extraction("Could not extract YouTube video ID".to_string()))?;

        let watch_url = format!(
            "https://www.youtube.com/watch?v={}&bpctr=9999999999&has_verified=1",
            video_id
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            ),
        );

        let resp = client.get(&watch_url).headers(headers).send().await?;
        let html = resp.text().await?;

        // Extract ytInitialPlayerResponse
        let player_json = match_group(&html, r#"ytInitialPlayerResponse\s*=\s*(\{.+?\});"#)
            .ok_or_else(|| YtDlpError::Extraction("Failed to locate ytInitialPlayerResponse".to_string()))?;

        let data: Value = serde_json::from_str(&player_json)?;
        let details = &data["videoDetails"];

        let title = details["title"].as_str().unwrap_or("YouTube Video").to_string();
        let uploader = details["author"].as_str().map(|s| s.to_string());
        let description = details["shortDescription"].as_str().map(|s| s.to_string());
        let duration = details["lengthSeconds"].as_str().and_then(|s| s.parse::<f64>().ok());

        let mut meta = MediaMetadata::new(&video_id, title, url, "youtube");
        meta.uploader = uploader;
        meta.description = description;
        meta.duration = duration;

        // Thumbnails
        if let Some(thumbs) = details["thumbnail"]["thumbnails"].as_array() {
            for t in thumbs {
                if let Some(t_url) = t["url"].as_str() {
                    meta.thumbnails.push(Thumbnail {
                        url: t_url.to_string(),
                        id: None,
                    });
                }
            }
        }

        // Formats
        let streaming_data = &data["streamingData"];
        let mut formats = Vec::new();

        if let Some(std_formats) = streaming_data["formats"].as_array() {
            for f in std_formats {
                if let Some(f_url) = f["url"].as_str() {
                    let itag = f["itag"].to_string();
                    let mime = f["mimeType"].as_str().unwrap_or("");
                    let ext = if mime.contains("webm") { "webm" } else { "mp4" };

                    let mut fmt = Format::new(&itag, ext, f_url);
                    fmt.width = f["width"].as_u64().map(|w| w as u32);
                    fmt.height = f["height"].as_u64().map(|h| h as u32);
                    fmt.fps = f["fps"].as_f64();
                    fmt.filesize = f["contentLength"].as_str().and_then(|s| s.parse().ok());
                    fmt.tbr = f["bitrate"].as_f64().map(|b| b / 1000.0);
                    formats.push(fmt);
                }
            }
        }

        if let Some(adaptive) = streaming_data["adaptiveFormats"].as_array() {
            for f in adaptive {
                if let Some(f_url) = f["url"].as_str() {
                    let itag = f["itag"].to_string();
                    let mime = f["mimeType"].as_str().unwrap_or("");
                    let is_audio = mime.contains("audio");
                    let ext = if is_audio {
                        if mime.contains("webm") { "opus" } else { "m4a" }
                    } else if mime.contains("webm") {
                        "webm"
                    } else {
                        "mp4"
                    };

                    let mut fmt = Format::new(&itag, ext, f_url);
                    fmt.width = f["width"].as_u64().map(|w| w as u32);
                    fmt.height = f["height"].as_u64().map(|h| h as u32);
                    fmt.fps = f["fps"].as_f64();
                    fmt.filesize = f["contentLength"].as_str().and_then(|s| s.parse().ok());
                    fmt.tbr = f["bitrate"].as_f64().map(|b| b / 1000.0);
                    if is_audio {
                        fmt.vcodec = Some("none".to_string());
                        fmt.acodec = Some(ext.to_string());
                    }
                    formats.push(fmt);
                }
            }
        }

        // HLS live / DASH manifest
        if let Some(hls_url) = streaming_data["hlsManifestUrl"].as_str() {
            let mut fmt = Format::new("hls", "mp4", hls_url);
            fmt.protocol = Protocol::M3u8;
            formats.push(fmt);
        }

        if formats.is_empty() {
            return Err(YtDlpError::Extraction(format!(
                "No downloadable stream URLs found for video {}",
                video_id
            )));
        }

        meta.formats = formats;
        Ok(meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id() {
        assert_eq!(
            YouTubeExtractor::extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YouTubeExtractor::extract_video_id("https://youtu.be/dQw4w9WgXcQ?si=123"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            YouTubeExtractor::extract_video_id("https://www.youtube.com/shorts/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }
}
