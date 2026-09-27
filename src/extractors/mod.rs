pub mod generic;
pub mod twitter;
pub mod youtube;

use crate::error::Result;
use crate::types::MediaMetadata;
use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;

#[async_trait]
pub trait Extractor: Send + Sync {
    fn id(&self) -> &'static str;
    fn suitable(&self, url: &str) -> bool;
    async fn extract(&self, url: &str, client: &Client) -> Result<MediaMetadata>;
}

pub struct ExtractorRegistry {
    extractors: Vec<Arc<dyn Extractor>>,
    fallback: Arc<dyn Extractor>,
}

impl Default for ExtractorRegistry {
    fn default() -> Self {
        let mut reg = Self {
            extractors: Vec::new(),
            fallback: Arc::new(generic::GenericExtractor::default()),
        };
        reg.register(Arc::new(youtube::YouTubeExtractor::default()));
        reg.register(Arc::new(twitter::TwitterExtractor::default()));
        reg
    }
}

impl ExtractorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, extractor: Arc<dyn Extractor>) {
        self.extractors.push(extractor);
    }

    pub async fn extract(&self, url: &str, client: &Client) -> Result<MediaMetadata> {
        for ext in &self.extractors {
            if ext.suitable(url) {
                return ext.extract(url, client).await;
            }
        }
        self.fallback.extract(url, client).await
    }
}
