use crate::error::{Result, YtDlpError};
use crate::hls::{parse_master_m3u8, parse_media_m3u8};
use crate::types::{MediaMetadata, Protocol};
use crate::utils::{format_bytes, sanitize_filename};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, RANGE, USER_AGENT};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub output_dir: PathBuf,
    pub output_name: Option<String>,
    pub format_selector: Option<String>,
    pub list_formats: bool,
    pub dump_json: bool,
    pub threads: usize,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            output_name: None,
            format_selector: None,
            list_formats: false,
            dump_json: false,
            threads: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Downloader {
    options: DownloadOptions,
    client: Client,
}

impl Downloader {
    pub fn new(options: DownloadOptions) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { options, client }
    }

    /// Prints format table matching `yt-dlp -F`.
    pub fn print_formats(&self, meta: &MediaMetadata) {
        println!("[info] Available formats for {}:", meta.id);
        println!(
            "{:<10} {:<6} {:<12} {:<10} {:<10} {}",
            "ID", "EXT", "RESOLUTION", "FPS", "FILESIZE", "PROTO"
        );
        println!("{}", "-".repeat(60));

        for f in &meta.formats {
            let fps_str = f.fps.map(|fps| format!("{:.0}fps", fps)).unwrap_or_else(|| "-".to_string());
            let size_str = f.filesize.map(format_bytes).unwrap_or_else(|| "-".to_string());
            let proto_str = match f.protocol {
                Protocol::Http => "http",
                Protocol::M3u8 => "m3u8",
                Protocol::M3u8Native => "m3u8_native",
                Protocol::Dash => "dash",
            };

            println!(
                "{:<10} {:<6} {:<12} {:<10} {:<10} {}",
                f.format_id,
                f.ext,
                f.resolution(),
                fps_str,
                size_str,
                proto_str
            );
        }
    }

    /// Downloads the selected format.
    pub async fn download(&self, meta: &MediaMetadata) -> Result<PathBuf> {
        if self.options.dump_json {
            let json = serde_json::to_string_pretty(meta)?;
            println!("{}", json);
            return Ok(self.options.output_dir.clone());
        }

        if self.options.list_formats {
            self.print_formats(meta);
            return Ok(self.options.output_dir.clone());
        }

        let format = meta
            .select_format(self.options.format_selector.as_deref())
            .ok_or_else(|| YtDlpError::FormatNotFound {
                requested: self.options.format_selector.clone().unwrap_or_else(|| "best".to_string()),
                available: meta.formats.iter().map(|f| f.format_id.as_str()).collect::<Vec<_>>().join(", "),
            })?;

        tokio::fs::create_dir_all(&self.options.output_dir).await?;

        let base_name = self.options.output_name.clone().unwrap_or_else(|| {
            sanitize_filename(&format!("{} [{}]", meta.title, meta.id))
        });
        let target_path = self.options.output_dir.join(format!("{}.{}", base_name, format.ext));

        println!(
            "[download] Destination: {}",
            target_path.display()
        );

        match format.protocol {
            Protocol::M3u8 | Protocol::M3u8Native => {
                self.download_hls(&format.url, &target_path).await?;
            }
            Protocol::Http | Protocol::Dash => {
                self.download_http(&format.url, &target_path, format.filesize.unwrap_or(0)).await?;
            }
        }

        println!("[download] 100% of {} in finished", target_path.display());
        Ok(target_path)
    }

    async fn download_http(&self, url: &str, dest: &Path, expected_size: u64) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            ),
        );

        let existing_len = if dest.exists() {
            tokio::fs::metadata(dest).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if existing_len > 0 {
            headers.insert(RANGE, HeaderValue::from_str(&format!("bytes={}-", existing_len)).unwrap());
        }

        let resp = self.client.get(url).headers(headers).send().await?;
        let total_size = if existing_len > 0 {
            resp.content_length().map(|l| l + existing_len).unwrap_or(expected_size)
        } else {
            resp.content_length().unwrap_or(expected_size)
        };

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_position(existing_len);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dest)
            .await?;

        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Done");
        Ok(())
    }

    async fn download_hls(&self, m3u8_url: &str, dest: &Path) -> Result<()> {
        println!("[hls] Fetching HLS playlist: {}", m3u8_url);
        let resp = self.client.get(m3u8_url).send().await?;
        let content = resp.text().await?;

        let segments = if content.contains("#EXT-X-STREAM-INF:") {
            let variants = parse_master_m3u8(&content, m3u8_url)?;
            let best_variant = variants
                .iter()
                .max_by_key(|v| v.bandwidth)
                .ok_or_else(|| YtDlpError::HlsParse("No stream variants in master playlist".to_string()))?;
            let media_resp = self.client.get(&best_variant.url).send().await?;
            let media_content = media_resp.text().await?;
            parse_media_m3u8(&media_content, &best_variant.url)?
        } else {
            parse_media_m3u8(&content, m3u8_url)?
        };

        println!("[hls] Downloading {} segments...", segments.len());
        let pb = ProgressBar::new(segments.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} segments ({eta})")
                .unwrap(),
        );

        let mut out_file = File::create(dest).await?;

        for seg in segments {
            let seg_bytes = self.client.get(&seg.url).send().await?.bytes().await?;
            out_file.write_all(&seg_bytes).await?;
            pb.inc(1);
        }

        pb.finish_with_message("HLS stream joined successfully");
        Ok(())
    }
}
