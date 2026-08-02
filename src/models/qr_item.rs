use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EccLevel {
    Low,
    Medium,
    Quartile,
    High,
}

impl EccLevel {
    pub fn label(&self) -> &'static str {
        match self {
            EccLevel::Low => "Low (~7% recovery)",
            EccLevel::Medium => "Medium (~15% recovery)",
            EccLevel::Quartile => "Quartile (~25% recovery)",
            EccLevel::High => "High (~30% recovery)",
        }
    }

    #[allow(dead_code)]
    pub fn short_code(&self) -> &'static str {
        match self {
            EccLevel::Low => "L",
            EccLevel::Medium => "M",
            EccLevel::Quartile => "Q",
            EccLevel::High => "H",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Png,
    Svg,
    Jpeg,
    Ascii,
    Unicode,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Svg => "svg",
            ExportFormat::Jpeg => "jpg",
            ExportFormat::Ascii => "txt",
            ExportFormat::Unicode => "txt",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Png => "PNG Image",
            ExportFormat::Svg => "SVG Vector",
            ExportFormat::Jpeg => "JPEG Image",
            ExportFormat::Ascii => "ASCII Text",
            ExportFormat::Unicode => "Unicode Text",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
    Url(String),
    Photo { file_path: String, filename: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrItem {
    pub id: String,
    pub original_url: String,
    pub target_type: TargetType,
    pub short_code: String,
    pub dynamic_url: Option<String>,
    pub expiration_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_favorite: bool,
    pub last_saved_path: Option<String>,
    pub ecc_level: EccLevel,
    pub export_format: ExportFormat,
    pub title: Option<String>,
}

impl QrItem {
    pub fn new(
        original_url: String,
        target_type: TargetType,
        short_code: String,
        dynamic_url: Option<String>,
        expiration_time: Option<DateTime<Utc>>,
        ecc_level: EccLevel,
        export_format: ExportFormat,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            original_url,
            target_type,
            short_code,
            dynamic_url,
            expiration_time,
            created_at: Utc::now(),
            is_favorite: false,
            last_saved_path: None,
            ecc_level,
            export_format,
            title: None,
        }
    }

    pub fn encoded_url(&self) -> &str {
        if let Some(ref d_url) = self.dynamic_url {
            d_url
        } else {
            &self.original_url
        }
    }

    pub fn is_expired(&self) -> bool {
        crate::models::expiration::ExpirationOption::is_expired(self.expiration_time)
    }

    pub fn remaining_time_str(&self) -> String {
        crate::models::expiration::ExpirationOption::format_remaining(self.expiration_time)
    }
}
