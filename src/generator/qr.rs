use anyhow::{anyhow, Result};
use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};
use qrcode::{EcLevel as QrEcLevel, QrCode};
use std::io::Cursor;
use std::path::Path;

use crate::models::{AppTheme, EccLevel, ExportFormat};

pub struct QrGenerator;

impl QrGenerator {
    /// Convert domain EccLevel to qrcode crate's EcLevel
    fn map_ecc(level: EccLevel) -> QrEcLevel {
        match level {
            EccLevel::Low => QrEcLevel::L,
            EccLevel::Medium => QrEcLevel::M,
            EccLevel::Quartile => QrEcLevel::Q,
            EccLevel::High => QrEcLevel::H,
        }
    }

    /// Create QrCode struct from input string
    pub fn create_qr(data: &str, ecc: EccLevel) -> Result<QrCode> {
        let qr = QrCode::with_error_correction_level(data.as_bytes(), Self::map_ecc(ecc))
            .map_err(|e| anyhow!("Failed to encode data into QR code: {}", e))?;
        Ok(qr)
    }

    /// Render QR code into crisp double-density Unicode half-block string for terminal display.
    /// Half-block characters ('▀', '▄', '█', ' ') allow 2 rows per line.
    pub fn render_unicode(qr: &QrCode, border: bool) -> String {
        let width = qr.width();
        let quiet = if border { 2 } else { 0 };
        let total_size = width + quiet * 2;

        let mut grid = vec![vec![false; total_size]; total_size];

        #[allow(clippy::needless_range_loop)]
        for y in 0..width {
            for x in 0..width {
                if qr[(x, y)] == qrcode::Color::Dark {
                    grid[y + quiet][x + quiet] = true;
                }
            }
        }

        let mut lines = Vec::new();
        let mut y = 0;
        while y < total_size {
            let mut line = String::with_capacity(total_size);
            #[allow(clippy::needless_range_loop)]
            for x in 0..total_size {
                let top = grid[y][x];
                let bottom = if y + 1 < total_size {
                    grid[y + 1][x]
                } else {
                    false
                };

                match (top, bottom) {
                    (true, true) => line.push('█'),
                    (true, false) => line.push('▀'),
                    (false, true) => line.push('▄'),
                    (false, false) => line.push(' '),
                }
            }
            lines.push(line);
            y += 2;
        }

        lines.join("\n")
    }

    /// Render QR code into traditional ASCII text ('##' for dark module, '  ' for light).
    pub fn render_ascii(qr: &QrCode) -> String {
        let width = qr.width();
        let mut lines = Vec::new();

        // Top border quiet zone
        let quiet_border = "  ".repeat(width + 2);
        lines.push(quiet_border.clone());

        for y in 0..width {
            let mut line = String::from("  ");
            for x in 0..width {
                if qr[(x, y)] == qrcode::Color::Dark {
                    line.push_str("██");
                } else {
                    line.push_str("  ");
                }
            }
            line.push_str("  ");
            lines.push(line);
        }

        lines.push(quiet_border);
        lines.join("\n")
    }

    /// Render QR code to RGBA Image buffer with custom colors, size, quiet zone, transparent BG options.
    pub fn render_image(
        qr: &QrCode,
        target_size: u32,
        quiet_zone: bool,
        theme: AppTheme,
        transparent_bg: bool,
    ) -> RgbaImage {
        let modules_count = qr.width();
        let margin_modules = if quiet_zone { 4 } else { 1 };
        let total_modules = (modules_count + margin_modules * 2) as u32;

        let module_pixel_size = (target_size / total_modules).max(4);
        let img_width = total_modules * module_pixel_size;

        let (fg_r, fg_g, fg_b) = theme.qr_fg_rgb();
        let (bg_r, bg_g, bg_b) = theme.qr_bg_rgb();

        let fg_color = Rgba([fg_r, fg_g, fg_b, 255]);
        let bg_color = if transparent_bg {
            Rgba([0, 0, 0, 0])
        } else {
            Rgba([bg_r, bg_g, bg_b, 255])
        };

        let mut img: RgbaImage = ImageBuffer::from_pixel(img_width, img_width, bg_color);

        for y in 0..modules_count {
            for x in 0..modules_count {
                if qr[(x, y)] == qrcode::Color::Dark {
                    let px_start = ((x + margin_modules) as u32) * module_pixel_size;
                    let py_start = ((y + margin_modules) as u32) * module_pixel_size;

                    for dy in 0..module_pixel_size {
                        for dx in 0..module_pixel_size {
                            let px = px_start + dx;
                            let py = py_start + dy;
                            if px < img_width && py < img_width {
                                img.put_pixel(px, py, fg_color);
                            }
                        }
                    }
                }
            }
        }

        img
    }

    /// Save image to disk in PNG, JPEG, SVG, ASCII, or Unicode format.
    pub fn save_to_file(
        qr: &QrCode,
        dest_path: &Path,
        format: ExportFormat,
        target_size: u32,
        quiet_zone: bool,
        theme: AppTheme,
        transparent_bg: bool,
    ) -> Result<()> {
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        match format {
            ExportFormat::Png => {
                let img = Self::render_image(qr, target_size, quiet_zone, theme, transparent_bg);
                img.save_with_format(dest_path, ImageFormat::Png)?;
            }
            ExportFormat::Jpeg => {
                let img = Self::render_image(qr, target_size, quiet_zone, theme, false);
                // Convert RGBA to RGB for JPEG
                let rgb_img = image::DynamicImage::ImageRgba8(img).to_rgb8();
                rgb_img.save_with_format(dest_path, ImageFormat::Jpeg)?;
            }
            ExportFormat::Svg => {
                let svg_xml = Self::render_svg_string(qr, theme, quiet_zone);
                std::fs::write(dest_path, svg_xml)?;
            }
            ExportFormat::Ascii => {
                let ascii_str = Self::render_ascii(qr);
                std::fs::write(dest_path, ascii_str)?;
            }
            ExportFormat::Unicode => {
                let unicode_str = Self::render_unicode(qr, true);
                std::fs::write(dest_path, unicode_str)?;
            }
        }

        Ok(())
    }

    /// Render SVG string with theme colors
    pub fn render_svg_string(qr: &QrCode, theme: AppTheme, quiet_zone: bool) -> String {
        let (fg_r, fg_g, fg_b) = theme.qr_fg_rgb();
        let (bg_r, bg_g, bg_b) = theme.qr_bg_rgb();
        let fg_hex = format!("#{:02x}{:02x}{:02x}", fg_r, fg_g, fg_b);
        let bg_hex = format!("#{:02x}{:02x}{:02x}", bg_r, bg_g, bg_b);

        let width = qr.width();
        let margin = if quiet_zone { 4 } else { 1 };
        let total = width + margin * 2;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {0} {0}" width="512" height="512">
<rect width="100%" height="100%" fill="{1}"/>
<path fill="{2}" d=""#,
            total, bg_hex, fg_hex
        );

        let mut path_data = String::new();
        for y in 0..width {
            for x in 0..width {
                if qr[(x, y)] == qrcode::Color::Dark {
                    let px = x + margin;
                    let py = y + margin;
                    path_data.push_str(&format!("M{},{}h1v1h-1z ", px, py));
                }
            }
        }

        svg.push_str(&path_data);
        svg.push_str(r#""/></svg>"#);
        svg
    }

    /// Render image bytes to memory in PNG format (for clipboard copy)
    #[allow(dead_code)]
    pub fn render_png_bytes(
        qr: &QrCode,
        target_size: u32,
        quiet_zone: bool,
        theme: AppTheme,
        transparent_bg: bool,
    ) -> Result<Vec<u8>> {
        let img = Self::render_image(qr, target_size, quiet_zone, theme, transparent_bg);
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png)?;
        Ok(buffer)
    }
}
