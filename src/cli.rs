use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::AppConfig;
use crate::generator::{BatchGenerator, QrGenerator};
use crate::models::{EccLevel, ExpirationOption, ExportFormat, QrItem, TargetType};
use crate::services::*;
use crate::storage::HistoryStore;
use crate::utils::{FileOps, UrlValidator};

#[derive(Parser, Debug)]
#[command(
    name = "qru",
    author = "amar",
    version = "0.1.0",
    about = "Modern terminal utility for generating dynamic QR codes with expiration and rich TUI",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a single QR code directly from CLI flags
    Generate {
        /// Target URL (e.g. https://example.com)
        #[arg(short, long)]
        url: String,

        /// Expiration period (5m, 10m, 30m, 1h, 1d, 7d, never)
        #[arg(short = 'x', long, default_value = "30m")]
        expire: String,

        /// Output file path (e.g. qr.png)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Export format (png, svg, jpeg, ascii, unicode)
        #[arg(short, long, default_value = "png")]
        format: String,

        /// Error Correction Level (L, M, Q, H)
        #[arg(short = 'e', long, default_value = "M")]
        ecc: String,

        /// Disable dynamic redirect proxy URL
        #[arg(long, default_value_t = false)]
        static_only: bool,
    },

    /// Batch generate QR codes from a CSV file
    Batch {
        /// Path to input CSV file containing links
        #[arg(short, long)]
        csv: PathBuf,

        /// Output directory for generated files
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },

    /// Run dynamic redirect HTTP proxy server standalone
    Server {
        /// Server port number
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },

    /// Print history of generated QR codes
    History,
}

impl Cli {
    pub async fn execute_command(command: Commands) -> Result<()> {
        let config = AppConfig::load();
        let history = HistoryStore::load();

        match command {
            Commands::Generate {
                url,
                expire,
                output,
                format,
                ecc,
                static_only,
            } => {
                let valid_url = UrlValidator::validate(&url)
                    .map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;

                let exp_opt = ExpirationOption::from_str_lenient(&expire)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                let ecc_level = match ecc.to_uppercase().as_str() {
                    "L" => EccLevel::Low,
                    "M" => EccLevel::Medium,
                    "Q" => EccLevel::Quartile,
                    "H" => EccLevel::High,
                    _ => EccLevel::Medium,
                };

                let export_fmt = match format.to_lowercase().as_str() {
                    "svg" => ExportFormat::Svg,
                    "jpg" | "jpeg" => ExportFormat::Jpeg,
                    "ascii" => ExportFormat::Ascii,
                    "unicode" | "txt" => ExportFormat::Unicode,
                    _ => ExportFormat::Png,
                };

                let short_code = uuid::Uuid::new_v4().to_string()[..8].to_string();
                let encoded_target = if !static_only {
                    format!("{}/r/{}", config.dynamic_server_host, short_code)
                } else {
                    valid_url.clone()
                };

                let qr = QrGenerator::create_qr(&encoded_target, ecc_level)?;

                if export_fmt == ExportFormat::Ascii || export_fmt == ExportFormat::Unicode {
                    println!("\n{}", QrGenerator::render_unicode(&qr, true));
                }

                let out_path = output.unwrap_or_else(|| {
                    let filename = FileOps::default_filename(export_fmt);
                    config.get_resolved_output_dir().join(filename)
                });

                QrGenerator::save_to_file(
                    &qr,
                    &out_path,
                    export_fmt,
                    config.default_qr_size,
                    true,
                    config.theme,
                    false,
                )?;

                let item = QrItem::new(
                    valid_url.clone(),
                    TargetType::Url(valid_url.clone()),
                    short_code,
                    if !static_only {
                        Some(encoded_target)
                    } else {
                        None
                    },
                    exp_opt.to_timestamp(),
                    ecc_level,
                    export_fmt,
                );
                let _ = history.add(item);

                println!("✓ Successfully generated QR code!");
                println!("  Target URL: {}", valid_url);
                println!("  Expiration: {}", exp_opt.label());
                println!("  Saved File: {:?}", out_path);
            }
            Commands::Batch { csv, output_dir } => {
                let out = output_dir.unwrap_or_else(|| config.get_resolved_output_dir());
                println!("Starting CSV batch generation from {:?}", csv);

                let res = BatchGenerator::process_csv(
                    &csv,
                    &out,
                    config.preferred_format,
                    config.default_ecc_level,
                    config.theme,
                    config.default_qr_size,
                    |curr, total, url| {
                        println!("[{}/{}] Processing {}", curr, total, url);
                    },
                )?;

                println!(
                    "\n✓ Batch Complete! Succeeded: {}, Failed: {}",
                    res.succeeded, res.failed
                );
            }
            Commands::Server { port } => {
                let provider = LocalRedirectProvider::new(history);
                let (listener, bound_port) = RedirectServer::bind_listener(port).await?;
                println!("⚡ Server running on port {}", bound_port);
                let server = RedirectServer::new(provider);
                let (_tx, rx) = tokio::sync::watch::channel(false);
                server.run_with_listener(listener, rx).await?;
            }
            Commands::History => {
                println!("=== QR Code Generation History ===");
                for (idx, item) in history.items().iter().enumerate() {
                    let fav = if item.is_favorite { "★" } else { " " };
                    println!(
                        "{}. {} URL: {} | Exp: {} | Code: {}",
                        idx + 1,
                        fav,
                        item.original_url,
                        item.remaining_time_str(),
                        item.short_code
                    );
                }
            }
        }
        Ok(())
    }
}
