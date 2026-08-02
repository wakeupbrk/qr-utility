use anyhow::{anyhow, Result};
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;

pub struct SystemClipboard;

impl SystemClipboard {
    /// Copy text string to system clipboard.
    pub fn copy_text(text: &str) -> Result<()> {
        let mut clipboard =
            Clipboard::new().map_err(|e| anyhow!("Failed to access system clipboard: {}", e))?;
        clipboard
            .set_text(text)
            .map_err(|e| anyhow!("Failed to set clipboard text: {}", e))?;
        Ok(())
    }

    /// Copy RGBA image to system clipboard.
    pub fn copy_image(img_rgba: &image::RgbaImage) -> Result<()> {
        let mut clipboard =
            Clipboard::new().map_err(|e| anyhow!("Failed to access system clipboard: {}", e))?;

        let width = img_rgba.width() as usize;
        let height = img_rgba.height() as usize;
        let raw_pixels = img_rgba.as_raw();

        let img_data = ImageData {
            width,
            height,
            bytes: Cow::Borrowed(raw_pixels),
        };

        clipboard
            .set_image(img_data)
            .map_err(|e| anyhow!("Failed to copy image to clipboard: {}", e))?;
        Ok(())
    }
}
