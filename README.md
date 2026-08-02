# QR Utility (`qru`)

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-performance terminal utility written in Rust for generating dynamic, expiring QR codes and sharing local files.

---

## Quick Install

To install `qru` directly onto any machine:

```bash
cargo install --git https://github.com/wakeupbrk/qr-utility
```

Once installed, launch the application from any terminal:

```bash
qru
```

---

## Features

- **Interactive 4-Step TUI**:
  - **Input**: Enter web URLs or drag-and-drop local image files (`Up`/`Down` keys to switch target mode).
  - **Expiration**: Select preset lifespan (5m, 10m, 30m, 1h, 1d, 7d, Never) or custom durations.
  - **Preview & Styling**: Live double-density Unicode preview (`▀`, `▄`, `█`), Error Correction customization, background transparency, and format toggles.
  - **Export**: Save high-resolution PNG, SVG, JPEG, ASCII, or Unicode output.
- **Dynamic Backend Server**:
  - Built-in asynchronous HTTP server powered by Tokio and Axum.
  - Automatic local Wi-Fi IP detection and port binding with fallback.
  - Serves HTTP 302 redirects for active URLs and styled photo viewers for image files.
  - Automatic expiration enforcement (HTTP 410 Gone) and automated server shutdown when link lifespans end.
- **Visual Themes**: Cyberpunk, Monokai, Ocean, Sunset, Matrix, and Slate Dark.
- **CSV Batch Generation**: Bulk-generate QR codes from CSV lists.
- **Headless CLI Subcommands**: Full scriptability for headless environments.

---

## Usage

### Interactive Mode

```bash
qru
```

| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Cycle navigation tabs |
| `Up` / `Down` | Toggle input mode (URL vs Photo) in Step 1 |
| `1` - `5` | Switch active tab |
| `T` | Cycle themes |
| `?` / `H` | Help overlay |
| `Ctrl+V` | Paste clipboard content |
| `S` / `Enter` | Save output file |
| `C` | Copy URL to clipboard |
| `I` | Copy PNG image bytes to clipboard |
| `Ctrl+C` | Quit |

---

### Headless CLI Mode

```bash
# Generate dynamic PNG QR code expiring in 1 hour
qru generate --url "https://example.com" --expire "1h" --output "my_qr.png"

# Process CSV batch
qru batch --csv links.csv --output-dir ./qrcodes
```

---

## License

MIT License. See `LICENSE` for details.
