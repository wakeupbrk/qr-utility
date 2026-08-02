use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::models::QrItem;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HistoryData {
    items: Vec<QrItem>,
}

/// Thread-safe history store backed by a shared Arc<Mutex<...>>.
/// All clones share the same underlying data so the redirect server
/// always sees items added by the TUI.
#[derive(Debug, Clone)]
pub struct HistoryStore {
    inner: Arc<Mutex<HistoryData>>,
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HistoryData::default())),
        }
    }
}

impl HistoryStore {
    pub fn history_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(home).join(".config").join("qr-utility");
            let _ = fs::create_dir_all(&dir);
            dir.join("history.json")
        } else {
            PathBuf::from("history.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::history_path();
        let data = if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                serde_json::from_str::<HistoryData>(&content).unwrap_or_default()
            } else {
                HistoryData::default()
            }
        } else {
            HistoryData::default()
        };
        Self {
            inner: Arc::new(Mutex::new(data)),
        }
    }

    fn save_inner(data: &HistoryData) -> Result<()> {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(data)
            .with_context(|| "Failed to serialize history store to JSON")?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write history file to {:?}", path))?;
        Ok(())
    }

    pub fn items(&self) -> Vec<QrItem> {
        let guard = self.inner.lock().unwrap();
        guard.items.clone()
    }

    pub fn add(&self, item: QrItem) -> Result<()> {
        let mut guard = self.inner.lock().unwrap();
        // Remove existing item if same ID or short code
        guard
            .items
            .retain(|i| i.id != item.id && i.short_code != item.short_code);
        guard.items.insert(0, item);
        // Keep top 200 items in history
        if guard.items.len() > 200 {
            guard.items.truncate(200);
        }
        Self::save_inner(&guard)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let mut guard = self.inner.lock().unwrap();
        guard.items.retain(|i| i.id != id);
        Self::save_inner(&guard)
    }

    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        let mut guard = self.inner.lock().unwrap();
        let mut new_state = false;
        if let Some(item) = guard.items.iter_mut().find(|i| i.id == id) {
            item.is_favorite = !item.is_favorite;
            new_state = item.is_favorite;
        }
        Self::save_inner(&guard)?;
        Ok(new_state)
    }

    pub fn find_by_short_code(&self, short_code: &str) -> Option<QrItem> {
        let guard = self.inner.lock().unwrap();
        guard
            .items
            .iter()
            .find(|i| i.short_code == short_code)
            .cloned()
    }

    pub fn search(&self, query: &str) -> Vec<QrItem> {
        let q = query.trim().to_lowercase();
        let guard = self.inner.lock().unwrap();
        if q.is_empty() {
            return guard.items.clone();
        }
        guard
            .items
            .iter()
            .filter(|i| {
                i.original_url.to_lowercase().contains(&q)
                    || i.short_code.to_lowercase().contains(&q)
                    || i.title.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }
}
