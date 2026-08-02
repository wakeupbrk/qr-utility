use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use std::fs;
use std::sync::Arc;
use tokio::sync::watch;

use crate::services::redirect_provider::{
    LocalRedirectProvider, RedirectProvider, RedirectResolution,
};

/// Shared state passed to every axum handler.
#[derive(Clone)]
struct ServerState {
    provider: LocalRedirectProvider,
}

pub struct RedirectServer {
    provider: LocalRedirectProvider,
}

impl RedirectServer {
    pub fn new(provider: LocalRedirectProvider) -> Self {
        Self { provider }
    }

    /// Try binding to initial_port, or fallback to sequential ports up to +10.
    pub async fn bind_listener(initial_port: u16) -> Result<(tokio::net::TcpListener, u16)> {
        for offset in 0..10 {
            let p = initial_port + offset;
            let addr = format!("0.0.0.0:{}", p);
            if let Ok(listener) = tokio::net::TcpListener::bind(&addr).await {
                return Ok((listener, p));
            }
        }
        anyhow::bail!(
            "Could not bind to any port in range {}-{}",
            initial_port,
            initial_port + 9
        );
    }

    /// Run background axum HTTP server using an already-bound TcpListener.
    pub async fn run_with_listener(
        self,
        listener: tokio::net::TcpListener,
        mut shutdown_rx: watch::Receiver<bool>,
    ) -> Result<()> {
        let state = Arc::new(ServerState {
            provider: self.provider,
        });

        let app = Router::new()
            .route("/", get(status_handler))
            .route("/status", get(status_handler))
            .route("/r/{code}", get(redirect_handler))
            .route("/r/{code}/raw", get(raw_handler))
            .with_state(state);

        // Run axum with graceful shutdown tied to our watch channel
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                loop {
                    if shutdown_rx.changed().await.is_err() {
                        break;
                    }
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            })
            .await?;

        Ok(())
    }
}

// ── Handlers ─────────────────────────────────────────────────────────

async fn status_handler(State(state): State<Arc<ServerState>>) -> Html<String> {
    let _ = &state;
    let lan_ip = crate::config::settings::detect_local_lan_ip();
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>QR Utility Server</title>
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f172a; color: #f8fafc; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
.card {{ background: #1e293b; padding: 2.5rem 3rem; border-radius: 1rem; box-shadow: 0 20px 25px -5px rgba(0,0,0,0.5); text-align: center; border: 1px solid #334155; max-width: 500px; }}
h1 {{ color: #38bdf8; margin-bottom: 0.5rem; font-size: 1.5rem; }}
p {{ color: #94a3b8; margin: 0.5rem 0; }}
.badge {{ background: #10b981; color: #022c22; font-weight: bold; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.85rem; }}
.ip {{ font-family: monospace; color: #f59e0b; background: #0f172a; padding: 0.25rem 0.5rem; border-radius: 0.25rem; }}
</style>
</head>
<body>
<div class="card">
  <h1>⚡ QR Utility Server</h1>
  <p>Status: <span class="badge">ACTIVE</span></p>
  <p>LAN: <span class="ip">http://{}:8080</span></p>
  <p style="color:#64748b;font-size:0.85rem;">Serving dynamic QR redirects & shared photos.</p>
</div>
</body>
</html>"#,
        lan_ip
    ))
}

async fn redirect_handler(
    State(state): State<Arc<ServerState>>,
    Path(code): Path<String>,
) -> Response {
    let resolution = state.provider.resolve_short_code(&code).await;
    match resolution {
        RedirectResolution::ActiveUrl { target_url } => {
            Redirect::temporary(&target_url).into_response()
        }
        RedirectResolution::ActivePhoto {
            file_path,
            filename,
        } => serve_photo_page(&code, &file_path, &filename),
        RedirectResolution::Expired {
            original_url,
            expired_at,
        } => serve_expired_page(&original_url, &expired_at),
        RedirectResolution::NotFound => {
            (StatusCode::NOT_FOUND, Html("Not Found".to_string())).into_response()
        }
    }
}

async fn raw_handler(State(state): State<Arc<ServerState>>, Path(code): Path<String>) -> Response {
    let resolution = state.provider.resolve_short_code(&code).await;
    match resolution {
        RedirectResolution::ActivePhoto {
            file_path,
            filename,
        } => {
            let sanitized = crate::utils::FileOps::sanitize_path(&file_path);
            if !sanitized.exists() {
                return (StatusCode::NOT_FOUND, "File not found").into_response();
            }
            match fs::read(&sanitized) {
                Ok(bytes) => {
                    let mime = guess_mime(&filename);
                    (
                        StatusCode::OK,
                        [
                            (header::CONTENT_TYPE, mime.to_string()),
                            (header::CACHE_CONTROL, "public, max-age=86400".to_string()),
                        ],
                        bytes,
                    )
                        .into_response()
                }
                Err(_) => (StatusCode::NOT_FOUND, "Could not read file").into_response(),
            }
        }
        _ => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

// ── Helper renderers ─────────────────────────────────────────────────

fn serve_photo_page(short_code: &str, _file_path: &str, filename: &str) -> Response {
    let raw_url = format!("/r/{}/raw", short_code);
    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Shared Photo</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ background: #090d16; color: #f8fafc; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; padding: 1.5rem; }}
.container {{ background: #131c2e; border: 1px solid #1e2d4a; border-radius: 1.25rem; padding: 1.5rem; max-width: 640px; width: 100%; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.7); text-align: center; }}
.header {{ margin-bottom: 1.25rem; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #1e2d4a; padding-bottom: 0.75rem; }}
.title {{ font-size: 1.1rem; font-weight: 600; color: #38bdf8; }}
.badge {{ background: #0284c7; color: #f0f9ff; font-size: 0.75rem; font-weight: bold; padding: 0.2rem 0.6rem; border-radius: 9999px; }}
.img-wrapper {{ border-radius: 0.75rem; overflow: hidden; background: #000; max-height: 70vh; margin-bottom: 1.25rem; border: 1px solid #334155; }}
img {{ max-width: 100%; max-height: 70vh; object-fit: contain; display: block; margin: 0 auto; }}
.actions {{ display: flex; gap: 0.75rem; justify-content: center; flex-wrap: wrap; }}
.btn {{ background: #0284c7; color: white; border: none; padding: 0.6rem 1.25rem; border-radius: 0.5rem; font-size: 0.9rem; font-weight: 600; text-decoration: none; cursor: pointer; transition: background 0.2s; }}
.btn:hover {{ background: #0369a1; }}
.footer {{ margin-top: 1rem; color: #64748b; font-size: 0.8rem; }}
</style>
</head>
<body>
<div class="container">
  <div class="header">
    <div class="title">📷 Shared Photo</div>
    <div class="badge">QR UTILITY</div>
  </div>
  <div class="img-wrapper">
    <img src="{raw}" alt="{name}" />
  </div>
  <div class="actions">
    <a href="{raw}" download="{name}" class="btn">⬇ Download</a>
    <a href="{raw}" target="_blank" class="btn" style="background:#334155;">🔍 Full Size</a>
  </div>
  <div class="footer">{name}</div>
</div>
</body>
</html>"#,
        raw = raw_url,
        name = filename,
    );
    Html(body).into_response()
}

fn serve_expired_page(original_url: &str, expired_at: &str) -> Response {
    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>QR Expired</title>
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
body {{ font-family: system-ui, -apple-system, sans-serif; background: #0f172a; color: #f8fafc; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }}
.card {{ background: #1e293b; padding: 3rem; border-radius: 1rem; border: 1px solid #ef4444; text-align: center; max-width: 480px; }}
h1 {{ color: #ef4444; font-size: 1.8rem; margin-bottom: 1rem; }}
p {{ color: #94a3b8; font-size: 1rem; line-height: 1.5; }}
.url {{ background: #0f172a; padding: 0.5rem; border-radius: 0.5rem; font-family: monospace; color: #64748b; word-break: break-all; margin: 1rem 0; }}
</style>
</head>
<body>
<div class="card">
  <h1>⏳ QR Code Expired</h1>
  <p>This link is no longer active.</p>
  <div class="url">{}</div>
  <p><small>Expired: {}</small></p>
</div>
</body>
</html>"#,
        original_url, expired_at
    );
    (StatusCode::GONE, Html(body)).into_response()
}

fn guess_mime(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else {
        "application/octet-stream"
    }
}
