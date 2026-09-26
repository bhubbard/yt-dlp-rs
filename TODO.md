# TODO: `yt-dlp-rs` 🦀📺

A high-performance, modular, memory-safe Rust port and headless engine inspired by [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp).

---

## 🎯 Mission & Goals

- **Blazing Fast Startup & Low Footprint**: Cold startup time under 10ms (vs Python's 200–500ms startup overhead) and minimal memory footprint suitable for edge runtimes, microservices, and high-throughput pipelines.
- **Modular Architecture**: Separate the URL resolver, extractor registry, JS interpreter sandbox, streaming transport, and post-processor into decoupled crates.
- **Robust Extractor Engine**: Declarative extractor DSL with embedded JavaScript execution (QuickJS/Boa) for YouTube's `n-sig` and cipher deciphering.
- **Multi-protocol Streamer**: Native concurrent chunked HTTP, HLS (`m3u8`), and MPEG-DASH (`mpd`) downloader with pause, resume, and rate limiting.

---

## 🏗️ Crates Architecture Plan

- [ ] `yt-dlp-core`: Core traits (`Extractor`, `Downloader`, `PostProcessor`), media info structs (`Format`, `MediaMetadata`, `Chapter`), and config parser.
- [ ] `yt-dlp-extractors`: Catalog of site extractors with regex routing and automated test vectors.
- [ ] `yt-dlp-crypto`: Signature deciphering, `n-token` transforms, and light JS engine integration (via `boa_engine` or `rquickjs`).
- [ ] `yt-dlp-stream`: Concurrent multi-segmented HTTP, HLS/m3u8, and DASH chunk engine.
- [ ] `yt-dlp-postprocess`: Metadata injector (MP4 atoms, ID3), thumbnail embedder, and FFmpeg remuxing bridge.
- [ ] `yt-dlp-cli`: Fully compatible CLI interface adhering to `yt-dlp` arguments.

---

## 📋 Implementation Checklist

### Phase 1: Core Framework & Extractor Abstractions
- [ ] Define `MediaMetadata`, `MediaFormat`, `Thumbnail`, `Subtitle` data models with `serde`.
- [ ] Implement `Extractor` trait:
  ```rust
  #[async_trait]
  pub trait Extractor: Send + Sync {
      fn id(&self) -> &'static str;
      fn suitable(&self, url: &str) -> bool;
      async fn extract(&self, url: &str, client: &HttpClient) -> Result<MediaMetadata, ExtractionError>;
  }
  ```
- [ ] Create `ExtractorRegistry` with regex trie matching for $O(1)$ URL dispatch.
- [ ] Implement HTTP client wrapper with cookie jar support, browser header impersonation, and proxy rotation.

### Phase 2: JavaScript Runtime Sandbox (Cipher / N-Sig Solvers)
- [ ] Integrate lightweight JS runtime (`rquickjs` or `boa_engine`) to execute YouTube player decipher routines.
- [ ] Implement regex extraction for player cache JS URLs and unscrambling algorithms.
- [ ] Add caching for parsed cipher algorithms to avoid re-evaluating player JS on repeated downloads.

### Phase 3: High-Priority Extractors
- [ ] **YouTube (`youtube.com`, `youtu.be`)**:
  - [ ] Web client, Android client, and iOS client API requests.
  - [ ] Adaptive formats (DASH/separate audio-video streams).
  - [ ] Captions / subtitles parsing (auto-generated & manual VTT/SRT).
  - [ ] Playlist and channel pagination.
- [ ] **TikTok (`tiktok.com`)**:
  - [ ] Direct MP4 extraction without watermark.
- [ ] **Twitter / X (`twitter.com`, `x.com`)**:
  - [ ] GraphQL API syndication and m3u8 playlist resolver.
- [ ] **Instagram (`instagram.com`)**:
  - [ ] Post, reel, and story JSON parsing.
- [ ] **Vimeo (`vimeo.com`)**:
  - [ ] Config JSON and DASH master manifest parser.
- [ ] **Generic Extractors**:
  - [ ] Direct HTML5 `<video>` / `<audio>` scrapers.
  - [ ] Generic HLS (`.m3u8`) and DASH (`.mpd`) resolvers.

### Phase 4: Download Engine & Streaming
- [ ] Multi-threaded concurrent chunk downloader for HTTP direct streams.
- [ ] Native HLS fragment downloader with parallel chunk fetching and AES-128 decryption.
- [ ] Native DASH segment timeline builder and fragment assembler.
- [ ] Bandwidth throttling / rate limiting (`governor`).
- [ ] Resumable downloads (`Range` headers and `.part` file preservation).
- [ ] Progress bars and terminal formatting with `indicatif`.

### Phase 5: Post-Processing & Remuxing
- [ ] Audio/video stream muxing into MP4/MKV via native container writer or FFmpeg subprocess.
- [ ] Subtitle embedding (soft-sub and hard-sub conversion).
- [ ] Metadata tagger:
  - [ ] MP4 metadata atoms via `mp4ameta`.
  - [ ] ID3v2 tags for MP3 audio via `id3`.
- [ ] Atomic file renaming with custom template format strings (e.g. `%(title)s-%(id)s.%(ext)s`).

### Phase 6: CLI & Compatibility
- [ ] Build `clap`-based CLI matching essential `yt-dlp` flags (`-f`, `-o`, `--write-sub`, `--embed-thumbnail`, `--extract-audio`).
- [ ] JSON output mode (`-j`, `--dump-json`) matching exact `yt-dlp` JSON schema for drop-in tooling compatibility.
- [ ] Configuration file support (`yt-dlp.conf`).

### Phase 7: Benchmarks & Parity Tests
- [ ] Cold start benchmark vs Python `yt-dlp`.
- [ ] Memory consumption comparison on 1,000-item playlist dumps.
- [ ] Automated regression test suite with mock HTTP responses for top extractors.
