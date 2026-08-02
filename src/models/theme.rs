use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    Cyberpunk,
    Monokai,
    Ocean,
    Sunset,
    Matrix,
    ClassicDark,
}

impl AppTheme {
    pub const ALL: &'static [AppTheme] = &[
        AppTheme::Cyberpunk,
        AppTheme::Monokai,
        AppTheme::Ocean,
        AppTheme::Sunset,
        AppTheme::Matrix,
        AppTheme::ClassicDark,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::Cyberpunk => "Cyberpunk (Neon Magenta & Cyan)",
            AppTheme::Monokai => "Monokai (Yellow & Green)",
            AppTheme::Ocean => "Ocean (Deep Blue & Teal)",
            AppTheme::Sunset => "Sunset (Orange & Purple)",
            AppTheme::Matrix => "Matrix (Emerald Green)",
            AppTheme::ClassicDark => "Classic Dark (Slate & Amber)",
        }
    }

    pub fn primary(&self) -> Color {
        match self {
            AppTheme::Cyberpunk => Color::Rgb(255, 0, 128),
            AppTheme::Monokai => Color::Rgb(229, 181, 103),
            AppTheme::Ocean => Color::Rgb(0, 180, 216),
            AppTheme::Sunset => Color::Rgb(247, 127, 0),
            AppTheme::Matrix => Color::Rgb(0, 255, 65),
            AppTheme::ClassicDark => Color::Rgb(217, 119, 6),
        }
    }

    pub fn secondary(&self) -> Color {
        match self {
            AppTheme::Cyberpunk => Color::Rgb(0, 240, 255),
            AppTheme::Monokai => Color::Rgb(166, 226, 46),
            AppTheme::Ocean => Color::Rgb(114, 9, 183),
            AppTheme::Sunset => Color::Rgb(214, 40, 40),
            AppTheme::Matrix => Color::Rgb(0, 143, 17),
            AppTheme::ClassicDark => Color::Rgb(71, 85, 105),
        }
    }

    pub fn accent(&self) -> Color {
        match self {
            AppTheme::Cyberpunk => Color::Rgb(255, 230, 0),
            AppTheme::Monokai => Color::Rgb(102, 217, 239),
            AppTheme::Ocean => Color::Rgb(72, 202, 228),
            AppTheme::Sunset => Color::Rgb(252, 191, 73),
            AppTheme::Matrix => Color::Rgb(57, 255, 20),
            AppTheme::ClassicDark => Color::Rgb(245, 158, 11),
        }
    }

    pub fn border(&self) -> Color {
        match self {
            AppTheme::Cyberpunk => Color::Rgb(70, 70, 100),
            AppTheme::Monokai => Color::Rgb(80, 80, 80),
            AppTheme::Ocean => Color::Rgb(40, 80, 120),
            AppTheme::Sunset => Color::Rgb(120, 60, 80),
            AppTheme::Matrix => Color::Rgb(0, 80, 20),
            AppTheme::ClassicDark => Color::Rgb(51, 65, 85),
        }
    }

    #[allow(dead_code)]
    pub fn background(&self) -> Color {
        Color::Reset
    }

    /// Color for QR code foreground (modules) in PNG/JPEG rendering.
    pub fn qr_fg_rgb(&self) -> (u8, u8, u8) {
        match self {
            AppTheme::Cyberpunk => (255, 0, 128),
            AppTheme::Monokai => (40, 40, 40),
            AppTheme::Ocean => (0, 119, 182),
            AppTheme::Sunset => (214, 40, 40),
            AppTheme::Matrix => (0, 200, 50),
            AppTheme::ClassicDark => (0, 0, 0),
        }
    }

    /// Color for QR code background in PNG/JPEG rendering.
    pub fn qr_bg_rgb(&self) -> (u8, u8, u8) {
        match self {
            AppTheme::Cyberpunk => (15, 15, 25),
            AppTheme::Monokai => (248, 248, 242),
            AppTheme::Ocean => (237, 246, 249),
            AppTheme::Sunset => (253, 240, 213),
            AppTheme::Matrix => (5, 20, 5),
            AppTheme::ClassicDark => (255, 255, 255),
        }
    }
}
