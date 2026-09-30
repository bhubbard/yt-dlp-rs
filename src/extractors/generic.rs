use crate::error::{Result, YtDlpError};
use crate::extractors::Extractor;
use crate::types::{Format, MediaMetadata, Protocol};
use crate::utils::match_group;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;

#[derive(Default, Debug, Clone, Copy)]
pub struct GenericExtractor;

#[async_trait]
impl Extractor for GenericExtractor {
    fn id(&self) -> &'static str {
        "generic"
    }

    fn suitable(&self, _url: &str) -> bool {
        true
    }

    async fn extract(&self, url: &str, client: &Client) -> Result<MediaMetadata> {
        let clean_url = url.trim();

        // 1. Direct stream link check (.m3u8, .mp4, .webm)
        if clean_url.ends_with(".m3u8") || clean_url.contains(".m3u8?") {
            let mut meta = MediaMetadata::new("stream", "HLS Stream", clean_url, "generic");
            let mut fmt = Format::new("hls", "mp4", clean_url);
            fmt.protocol = Protocol::M3u8;
            meta.formats.push(fmt);
            return Ok(meta);
        }

        if clean_url.ends_with(".mp4") || clean_url.ends_with(".webm") || clean_url.ends_with(".mkv") {
            let ext = clean_url.rsplit('.').next().unwrap_or("mp4");
            let mut meta = MediaMetadata::new("direct", "Direct Media Stream", clean_url, "generic");
            meta.formats.push(Format::new("direct", ext, clean_url));
            return Ok(meta);
        }

        // 2. Fetch page HTML and look for video tags / og:video
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            ),
        );

        let resp = client.get(clean_url).headers(headers).send().await?;
        let html = resp.text().await?;

        let title = match_group(&html, r#"<title>(.*?)</title>"#).unwrap_or_else(|| "Video".to_string());
        let mut meta = MediaMetadata::new("generic", title, clean_url, "generic");

        // Try og:video
        if let Some(og_url) = match_group(&html, r#"<meta\s+property="og:video"\s+content="([^"]+)""#) {
            let is_hls = og_url.contains(".m3u8");
            let mut fmt = Format::new("og-video", "mp4", og_url);
            if is_hls {
                fmt.protocol = Protocol::M3u8;
            }
            meta.formats.push(fmt);
            return Ok(meta);
        }

        // Try <video src="...">
        if let Some(src) = match_group(&html, r#"<video[^>]+src="([^"]+)""#) {
            let is_hls = src.contains(".m3u8");
            let mut fmt = Format::new("html5", "mp4", src);
            if is_hls {
                fmt.protocol = Protocol::M3u8;
            }
            meta.formats.push(fmt);
            return Ok(meta);
        }

        Err(YtDlpError::UnsupportedUrl(clean_url.to_string()))
    }
}
