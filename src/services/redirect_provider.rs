use anyhow::Result;
use async_trait::async_trait;

use crate::models::{QrItem, TargetType};
use crate::storage::HistoryStore;

#[derive(Debug, Clone)]
pub enum RedirectResolution {
    ActiveUrl {
        target_url: String,
    },
    ActivePhoto {
        file_path: String,
        filename: String,
    },
    Expired {
        original_url: String,
        expired_at: String,
    },
    NotFound,
}

/// Abstract trait for dynamic QR code redirect resolution.
#[async_trait]
pub trait RedirectProvider: Send + Sync {
    #[allow(dead_code)]
    async fn register_dynamic_url(&self, item: &QrItem, base_host: &str) -> Result<String>;

    async fn resolve_short_code(&self, short_code: &str) -> RedirectResolution;
}

/// Local redirect provider that shares the same HistoryStore as the TUI app.
/// Because HistoryStore now uses Arc<Mutex> internally, cloning it gives
/// a handle to the *same* underlying data — no stale copies.
#[derive(Clone)]
pub struct LocalRedirectProvider {
    store: HistoryStore,
}

impl LocalRedirectProvider {
    pub fn new(store: HistoryStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl RedirectProvider for LocalRedirectProvider {
    async fn register_dynamic_url(&self, item: &QrItem, base_host: &str) -> Result<String> {
        let dynamic_url = format!("{}/r/{}", base_host.trim_end_matches('/'), item.short_code);
        Ok(dynamic_url)
    }

    async fn resolve_short_code(&self, short_code: &str) -> RedirectResolution {
        if let Some(item) = self.store.find_by_short_code(short_code) {
            if item.is_expired() {
                let exp_str = item
                    .expiration_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                RedirectResolution::Expired {
                    original_url: item.original_url.clone(),
                    expired_at: exp_str,
                }
            } else {
                match &item.target_type {
                    TargetType::Url(url) => RedirectResolution::ActiveUrl {
                        target_url: url.clone(),
                    },
                    TargetType::Photo {
                        file_path,
                        filename,
                    } => RedirectResolution::ActivePhoto {
                        file_path: file_path.clone(),
                        filename: filename.clone(),
                    },
                }
            }
        } else {
            RedirectResolution::NotFound
        }
    }
}
