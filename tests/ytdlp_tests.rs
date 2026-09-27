use yt_dlp_rs::{
    format_bytes, format_duration, parse_master_m3u8, parse_media_m3u8, Format, MediaMetadata,
    Protocol,
};

#[test]
fn test_format_selection_heuristics() {
    let mut meta = MediaMetadata::new("id_123", "Demo Video", "https://youtube.com/watch?v=123", "youtube");

    let mut f_1080 = Format::new("137", "mp4", "https://cdn/1080.mp4");
    f_1080.width = Some(1920);
    f_1080.height = Some(1080);
    f_1080.filesize = Some(50_000_000);

    let mut f_720 = Format::new("22", "mp4", "https://cdn/720.mp4");
    f_720.width = Some(1280);
    f_720.height = Some(720);
    f_720.filesize = Some(25_000_000);

    let mut f_audio = Format::new("140", "m4a", "https://cdn/audio.m4a");
    f_audio.vcodec = Some("none".to_string());
    f_audio.acodec = Some("m4a".to_string());
    f_audio.filesize = Some(5_000_000);

    meta.formats.push(f_720);
    meta.formats.push(f_1080);
    meta.formats.push(f_audio);

    // Test "best" selects 1080
    let best = meta.select_format(Some("best")).unwrap();
    assert_eq!(best.format_id, "137");

    // Test "worst" selects 720 (among video formats)
    let worst = meta.select_format(Some("worst")).unwrap();
    assert_eq!(worst.format_id, "22");

    // Test "bestaudio" selects audio track
    let audio = meta.select_format(Some("bestaudio")).unwrap();
    assert_eq!(audio.format_id, "140");

    // Test explicit format ID
    let explicit = meta.select_format(Some("22")).unwrap();
    assert_eq!(explicit.format_id, "22");
}

#[test]
fn test_metadata_json_serialization() {
    let mut meta = MediaMetadata::new("rickroll", "Never Gonna Give You Up", "https://youtu.be/dQw4w9WgXcQ", "youtube");
    meta.uploader = Some("Rick Astley".to_string());
    meta.duration = Some(212.0);

    let mut fmt = Format::new("1080", "mp4", "https://cdn/stream.mp4");
    fmt.protocol = Protocol::Http;
    meta.formats.push(fmt);

    let json = serde_json::to_string(&meta).unwrap();
    assert!(json.contains("rickroll"));
    assert!(json.contains("Never Gonna Give You Up"));
    assert!(json.contains("Rick Astley"));

    let deserialized: MediaMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "rickroll");
    assert_eq!(deserialized.duration, Some(212.0));
    assert_eq!(deserialized.formats.len(), 1);
}

#[test]
fn test_hls_stream_resolutions() {
    let master = r#"#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=640x360
360p.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=3000000,RESOLUTION=1920x1080
1080p.m3u8
"#;
    let variants = parse_master_m3u8(master, "https://live.video/master.m3u8").unwrap();
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[1].bandwidth, 3000000);
    assert_eq!(variants[1].url, "https://live.video/1080p.m3u8");

    let media = r#"#EXTM3U
#EXTINF:4.0,
seg1.ts
#EXTINF:4.0,
seg2.ts
"#;
    let segs = parse_media_m3u8(media, "https://live.video/1080p.m3u8").unwrap();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].url, "https://live.video/seg1.ts");
}

#[test]
fn test_format_helpers() {
    assert_eq!(format_bytes(5_242_880), "5.00MiB");
    assert_eq!(format_duration(125.0), "02:05");
}
