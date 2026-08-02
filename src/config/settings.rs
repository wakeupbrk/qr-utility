use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::process::Command;

use crate::models::{AppTheme, EccLevel, ExportFormat};

/// Intelligent LAN IP detector for macOS / Linux / Windows.
/// Skips loopback (lo0 / 127.0.0.1) and VPN / point-to-point interfaces (utun, tun, wg, ppp, etc.)
/// to ensure mobile devices on the local Wi-Fi / LAN can scan and reach the server.
pub fn detect_local_lan_ip() -> String {
    // 1. Try parsing system network interfaces (ifconfig on macOS/Linux)
    if let Ok(output) = Command::new("ifconfig").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut current_iface_skip = false;

            for line in text.lines() {
                if !line.starts_with('\t') && !line.starts_with(' ') {
                    let iface_name = line.split(':').next().unwrap_or("");
                    let flags = line;

                    // Exclude loopback (lo), point-to-point (utun, tun, wg, ppp), and POINTOPOINT flags
                    current_iface_skip = iface_name.starts_with("lo")
                        || iface_name.starts_with("utun")
                        || iface_name.starts_with("tun")
                        || iface_name.starts_with("wg")
                        || iface_name.starts_with("ppp")
                        || flags.contains("POINTOPOINT");
                } else if !current_iface_skip {
                    let trimmed = line.trim();
                    if trimmed.starts_with("inet ") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let ip = parts[1];
                            if ip != "127.0.0.1" && !ip.starts_with("169.254.") {
                                return ip.to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Fallback: UDP connect trick
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if !ip.starts_with("127.") && !ip.starts_with("10.3.") {
                    return ip;
                }
            }
        }
    }

    "127.0.0.1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_output_dir: String,
    pub default_qr_size: u32,
    pub default_ecc_level: EccLevel,
    pub preferred_format: ExportFormat,
    pub theme: AppTheme,
    pub default_quiet_zone: bool,
    pub transparent_bg: bool,
    pub dynamic_server_enabled: bool,
    pub dynamic_server_port: u16,
    pub dynamic_server_host: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let default_dir = format!("{}/qrcodes", home);
        let lan_ip = detect_local_lan_ip();

        Self {
            default_output_dir: default_dir,
            default_qr_size: 512,
            default_ecc_level: EccLevel::Medium,
            preferred_format: ExportFormat::Png,
            theme: AppTheme::Cyberpunk,
            default_quiet_zone: true,
            transparent_bg: false,
            dynamic_server_enabled: true,
            dynamic_server_port: 8080,
            dynamic_server_host: format!("http://{}:8080", lan_ip),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let config_dir = PathBuf::from(home).join(".config").join("qr-utility");
            let _ = fs::create_dir_all(&config_dir);
            config_dir.join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = toml::from_str::<AppConfig>(&content) {
                    let lan_ip = detect_local_lan_ip();
                    // Always ensure dynamic_server_host points to valid LAN IP
                    config.dynamic_server_host =
                        format!("http://{}:{}", lan_ip, config.dynamic_server_port);
                    let _ = config.save();
                    return config;
                }
            }
        }
        let config = Self::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize app configuration to TOML")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write configuration to {:?}", path))?;
        Ok(())
    }

    pub fn get_resolved_output_dir(&self) -> PathBuf {
        let path = PathBuf::from(&self.default_output_dir);
        let _ = fs::create_dir_all(&path);
        path
    }
}
