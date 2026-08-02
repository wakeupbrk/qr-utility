use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::ExportFormat;

pub struct FileOps;

impl FileOps {
    /// Format timestamp default filename: qr_YYYY-MM-DD_HH-MM-SS.ext
    pub fn default_filename(format: ExportFormat) -> String {
        let now = Utc::now();
        format!(
            "qr_{}.{}",
            now.format("%Y-%m-%d_%H-%M-%S"),
            format.extension()
        )
    }

    /// Check if target file exists.
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Robust path sanitizer for terminal drag-and-drop file paths.
    /// Handles single quotes, double quotes, file:// URLs, escaped spaces (\ ), and tilde (~).
    pub fn sanitize_path(input: &str) -> PathBuf {
        let mut s = input.trim().to_string();

        // 1. Strip file:// scheme if present from browser/finder drags
        if let Some(stripped) = s.strip_prefix("file://") {
            s = stripped.to_string();
        }

        // 2. Decode percent-encoded spaces (%20)
        s = s.replace("%20", " ");

        // 3. Trim single or double quotes
        s = s.trim_matches('\'').trim_matches('"').trim().to_string();

        // 4. Replace terminal escaped spaces (\ )
        s = s.replace(r"\ ", " ");

        // 5. Expand ~ home directory
        if s.starts_with("~/") || s == "~" {
            if let Ok(home) = std::env::var("HOME") {
                s = s.replacen('~', &home, 1);
            }
        }

        PathBuf::from(s)
    }

    /// Open file in system default application (Preview, Image Viewer, Browser).
    pub fn open_file(path: &Path) -> Result<()> {
        open::that(path)
            .with_context(|| format!("Failed to open file in system default app: {:?}", path))
    }

    /// Reveal file in macOS Finder or file manager.
    pub fn reveal_in_finder(path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let path_str = path.to_string_lossy();
            std::process::Command::new("open")
                .args(["-R", &path_str])
                .spawn()
                .with_context(|| format!("Failed to reveal file in Finder: {:?}", path))?;
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let parent = path.parent().unwrap_or(path);
            open::that(parent).with_context(|| format!("Failed to open directory: {:?}", parent))
        }
    }

    /// Ensure output directory exists and is writable.
    pub fn ensure_dir_exists(dir_path: &Path) -> Result<()> {
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)
                .with_context(|| format!("Failed to create output directory: {:?}", dir_path))?;
        }
        Ok(())
    }
}
