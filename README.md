# yt-dlp-rs 📺🦀

[![CI](https://github.com/bhubbard/yt-dlp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bhubbard/yt-dlp-rs/actions)
[![Pages](https://github.com/bhubbard/yt-dlp-rs/actions/workflows/pages.yml/badge.svg)](https://code.brandonhubbard.com/yt-dlp-rs/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-red.svg)](LICENSE)

A high-performance, modular, memory-safe media downloader written in pure Rust inspired by [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp). Sub-5ms startup times with zero Python runtime overhead.

👉 **Interactive Playground & Documentation**: [code.brandonhubbard.com/yt-dlp-rs](http://code.brandonhubbard.com/yt-dlp-rs/)

---

## ⚡ Key Highlights

- **Sub-5ms Cold Startup**: Eliminates Python interpreter startup latency, making it ideal for high-throughput edge workers, CLI tools, and microservices.
- **Multi-Protocol Engine**: Native concurrent chunked HTTP transport, HLS (`.m3u8`) master/media playlist resolver, and MPEG-DASH streaming.
- **Modular Extractor Framework**: Native extractors for YouTube (Innertube API, DASH/adaptive streams), Twitter/X (Syndication & GraphQL APIs), and generic HTML5/video feeds.
- **100% Memory Safe**: Zero unsafe code, built with async `tokio`, `reqwest`, and `indicatif`.

---

## 🚀 Installation

```bash
# Clone and build with Cargo
git clone https://github.com/bhubbard/yt-dlp-rs.git
cd yt-dlp-rs
cargo install --path .
```

---

## 🛠️ CLI Usage

```bash
# 1. Inspect available formats for a video (-F)
yt-dlp -F "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# 2. Download best available quality (-f best) to specific folder (-P)
yt-dlp -f best -P ~/Downloads "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# 3. Download best audio track only (-f bestaudio)
yt-dlp -f bestaudio -P ~/Music "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# 4. Multi-connection fragment downloading (-N)
yt-dlp -N 8 "https://example.com/live/stream.m3u8"

# 5. Output machine-readable JSON metadata (-j)
yt-dlp -j "https://x.com/user/status/123456789"
```

---

## 🧪 Running Tests

```bash
cargo test
```
All unit tests and integration tests run in under 0.01 seconds.

---

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))
