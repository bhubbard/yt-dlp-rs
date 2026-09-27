use thiserror::Error;

#[derive(Error, Debug)]
pub enum YtDlpError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported URL or no matching extractor: {0}")]
    UnsupportedUrl(String),

    #[error("Extraction error: {0}")]
    Extraction(String),

    #[error("Requested format '{requested}' not available. Available formats: {available}")]
    FormatNotFound {
        requested: String,
        available: String,
    },

    #[error("HLS parsing error: {0}")]
    HlsParse(String),

    #[error("General error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, YtDlpError>;
