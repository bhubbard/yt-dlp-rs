use clap::Parser;
use std::path::PathBuf;
use yt_dlp_rs::{DownloadOptions, Downloader, ExtractorRegistry, Result};

#[derive(Parser, Debug)]
#[command(name = "yt-dlp")]
#[command(author = "Brandon Hubbard <brandon@brandonhubbard.com>")]
#[command(version = "0.0.1")]
#[command(about = "A high-performance, modular, memory-safe media downloader written in pure Rust", long_about = None)]
struct Cli {
    /// URLs to download or inspect
    #[arg(required = true)]
    urls: Vec<String>,

    /// List available formats of each provided URL
    #[arg(short = 'F', long = "list-formats")]
    list_formats: bool,

    /// Video format code, see "FORMAT SELECTION" for more details (e.g. "best", "bestaudio", or itag)
    #[arg(short = 'f', long = "format")]
    format: Option<String>,

    /// Simulate, quiet but print JSON information
    #[arg(short = 'j', long = "dump-json")]
    dump_json: bool,

    /// Output filename template
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Output destination folder
    #[arg(short = 'P', long = "paths", default_value = ".")]
    paths: PathBuf,

    /// Number of concurrent download workers
    #[arg(short = 'N', long = "concurrent-fragments", default_value = "4")]
    concurrent_fragments: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let registry = ExtractorRegistry::new();
    let client = reqwest::Client::new();

    let download_opts = DownloadOptions {
        output_dir: cli.paths,
        output_name: cli.output,
        format_selector: cli.format,
        list_formats: cli.list_formats,
        dump_json: cli.dump_json,
        threads: cli.concurrent_fragments,
    };

    let downloader = Downloader::new(download_opts);

    for url in &cli.urls {
        match registry.extract(url, &client).await {
            Ok(meta) => {
                if let Err(e) = downloader.download(&meta).await {
                    eprintln!("[error] Download failed: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[error] Extraction failed for {}: {}", url, e);
            }
        }
    }

    Ok(())
}
