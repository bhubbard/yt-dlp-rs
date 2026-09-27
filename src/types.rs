use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    #[default]
    Http,
    M3u8,
    M3u8Native,
    Dash,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Format {
    pub format_id: String,
    pub ext: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcodec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acodec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesize: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tbr: Option<f64>,
    #[serde(default)]
    pub protocol: Protocol,
}

impl Format {
    pub fn new(format_id: impl Into<String>, ext: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            format_id: format_id.into(),
            ext: ext.into(),
            url: url.into(),
            width: None,
            height: None,
            fps: None,
            vcodec: None,
            acodec: None,
            filesize: None,
            tbr: None,
            protocol: Protocol::Http,
        }
    }

    pub fn resolution(&self) -> String {
        match (self.width, self.height) {
            (Some(w), Some(h)) => format!("{}x{}", w, h),
            _ => "audio only".to_string(),
        }
    }

    pub fn is_video(&self) -> bool {
        self.vcodec.as_deref() != Some("none") && self.height.unwrap_or(0) > 0
    }

    pub fn is_audio_only(&self) -> bool {
        self.vcodec.as_deref() == Some("none") || (self.height.is_none() && self.acodec.is_some())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub start_time: f64,
    pub end_time: f64,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thumbnails: Vec<Thumbnail>,
    pub formats: Vec<Format>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chapters: Vec<Chapter>,
    pub webpage_url: String,
    pub extractor: String,
}

impl MediaMetadata {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        webpage_url: impl Into<String>,
        extractor: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            uploader: None,
            duration: None,
            thumbnails: Vec::new(),
            formats: Vec::new(),
            chapters: Vec::new(),
            webpage_url: webpage_url.into(),
            extractor: extractor.into(),
        }
    }

    /// Selects a format based on yt-dlp format selector (e.g. "best", "worst", "bestaudio", or specific ID).
    pub fn select_format(&self, selector: Option<&str>) -> Option<&Format> {
        let sel = selector.unwrap_or("best");
        match sel {
            "best" => self
                .formats
                .iter()
                .filter(|f| f.is_video())
                .max_by_key(|f| (f.height.unwrap_or(0), f.filesize.unwrap_or(0))),
            "worst" => self
                .formats
                .iter()
                .filter(|f| f.is_video())
                .min_by_key(|f| (f.height.unwrap_or(0), f.filesize.unwrap_or(0))),
            "bestaudio" => self
                .formats
                .iter()
                .filter(|f| f.is_audio_only())
                .max_by_key(|f| f.filesize.unwrap_or(0)),
            id => self.formats.iter().find(|f| f.format_id == id),
        }
        .or_else(|| self.formats.first())
    }
}
