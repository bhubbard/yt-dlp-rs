use crate::error::{Result, YtDlpError};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct HlsStreamVariant {
    pub bandwidth: u64,
    pub resolution: Option<(u32, u32)>,
    pub codecs: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HlsSegment {
    pub duration: f64,
    pub url: String,
}

/// Parses an HLS Master Playlist containing variants.
pub fn parse_master_m3u8(content: &str, base_url: &str) -> Result<Vec<HlsStreamVariant>> {
    let base = Url::parse(base_url)?;
    let mut variants = Vec::new();
    let mut current_bw = 0;
    let mut current_res = None;
    let mut current_codecs = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("#EXT-X-STREAM-INF:") {
            let attrs = &line["#EXT-X-STREAM-INF:".len()..];
            for attr in attrs.split(',') {
                let kv: Vec<&str> = attr.splitn(2, '=').collect();
                if kv.len() == 2 {
                    let key = kv[0].trim();
                    let val = kv[1].trim().trim_matches('"');
                    match key {
                        "BANDWIDTH" => current_bw = val.parse().unwrap_or(0),
                        "RESOLUTION" => {
                            let dims: Vec<&str> = val.split('x').collect();
                            if dims.len() == 2 {
                                if let (Ok(w), Ok(h)) = (dims[0].parse(), dims[1].parse()) {
                                    current_res = Some((w, h));
                                }
                            }
                        }
                        "CODECS" => current_codecs = Some(val.to_string()),
                        _ => {}
                    }
                }
            }
        } else if !line.starts_with('#') && !line.is_empty() {
            let resolved = base.join(line)?.to_string();
            variants.push(HlsStreamVariant {
                bandwidth: current_bw,
                resolution: current_res.take(),
                codecs: current_codecs.take(),
                url: resolved,
            });
            current_bw = 0;
        }
    }

    Ok(variants)
}

/// Parses an HLS Media Playlist containing audio/video segments.
pub fn parse_media_m3u8(content: &str, base_url: &str) -> Result<Vec<HlsSegment>> {
    let base = Url::parse(base_url)?;
    let mut segments = Vec::new();
    let mut current_duration = 0.0;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("#EXTINF:") {
            let dur_str = &line["#EXTINF:".len()..].split(',').next().unwrap_or("0");
            current_duration = dur_str.parse().unwrap_or(0.0);
        } else if !line.starts_with('#') && !line.is_empty() {
            let resolved = base.join(line)?.to_string();
            segments.push(HlsSegment {
                duration: current_duration,
                url: resolved,
            });
            current_duration = 0.0;
        }
    }

    if segments.is_empty() {
        return Err(YtDlpError::HlsParse("No segments found in m3u8 playlist".to_string()));
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_master_m3u8() {
        let m3u8 = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-STREAM-INF:BANDWIDTH=1280000,RESOLUTION=1280x720,CODECS="avc1.64001f,mp4a.40.2"
720p.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2560000,RESOLUTION=1920x1080,CODECS="avc1.640028,mp4a.40.2"
1080p.m3u8
"#;
        let variants = parse_master_m3u8(m3u8, "https://cdn.example.com/live/master.m3u8").unwrap();
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].bandwidth, 1280000);
        assert_eq!(variants[0].resolution, Some((1280, 720)));
        assert_eq!(variants[0].url, "https://cdn.example.com/live/720p.m3u8");
        assert_eq!(variants[1].resolution, Some((1920, 1080)));
    }

    #[test]
    fn test_parse_media_m3u8() {
        let m3u8 = r#"#EXTM3U
#EXT-X-TARGETDURATION:10
#EXTINF:9.009,
segment1.ts
#EXTINF:8.5,
segment2.ts
#EXT-X-ENDLIST
"#;
        let segs = parse_media_m3u8(m3u8, "https://cdn.example.com/hls/playlist.m3u8").unwrap();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].duration, 9.009);
        assert_eq!(segs[0].url, "https://cdn.example.com/hls/segment1.ts");
        assert_eq!(segs[1].duration, 8.5);
    }
}
