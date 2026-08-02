use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::theme::ThemeStyles;

pub struct RedirectViewWidget;

impl RedirectViewWidget {
    pub fn draw(
        frame: &mut Frame,
        area: Rect,
        server_running: bool,
        port: u16,
        host_url: &str,
        styles: &ThemeStyles,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Server status header
                Constraint::Min(8),    // Architecture explanation
                Constraint::Length(3), // Footer hints
            ])
            .margin(1)
            .split(area);

        // Server Status Panel
        let status_block = Block::default()
            .title(" Dynamic Redirect Backend Service ")
            .borders(Borders::ALL)
            .border_style(if server_running {
                styles.border_focus
            } else {
                styles.border
            });

        let status_line = if server_running {
            Line::from(vec![
                Span::styled(" STATUS: ", styles.accent),
                Span::styled("● RUNNING ", styles.success),
                Span::styled(format!("(Listening on {})", host_url), styles.primary),
            ])
        } else {
            Line::from(vec![
                Span::styled(" STATUS: ", styles.accent),
                Span::styled("○ STOPPED ", styles.error),
                Span::styled(
                    "(Press [Space] to start background server)",
                    styles.text_muted,
                ),
            ])
        };

        let endpoint_example = format!("{}/r/{{short_code}}", host_url);

        let status_p = Paragraph::new(vec![
            status_line,
            Line::from(""),
            Line::from(vec![
                Span::raw(" Endpoint Format: "),
                Span::styled(endpoint_example, styles.secondary),
            ]),
            Line::from(vec![
                Span::raw(" Server Port: "),
                Span::styled(port.to_string(), styles.accent),
                Span::raw(" │ Automatic expiration enforcement & photo sharing enabled"),
            ]),
        ])
        .block(status_block);
        frame.render_widget(status_p, chunks[0]);

        // Architecture & Info Panel
        let arch_block = Block::default()
            .title(" Dynamic QR Architecture ")
            .borders(Borders::ALL)
            .border_style(styles.border);

        let arch_info = Paragraph::new(vec![
            Line::from(Span::styled(
                "How Expiring Dynamic QR Codes Work:",
                styles.accent,
            )),
            Line::from(""),
            Line::from(
                " 1. Static QR Codes: Destination URL is permanently hardcoded into the barcode.",
            ),
            Line::from(
                "    Static QR codes can NEVER expire or be updated once printed.",
            ),
            Line::from(""),
            Line::from(
                " 2. Dynamic QR Codes: Barcode encodes a proxy link pointing to this application's server.",
            ),
            Line::from(""),
            Line::from(
                " 3. When scanned by a phone camera on your local network:",
            ),
            Line::from(vec![
                Span::styled("    • Active Link: ", styles.success),
                Span::raw("Redirects immediately (302) or serves shared photo viewer page"),
            ]),
            Line::from(vec![
                Span::styled("    • Expired Link: ", styles.error),
                Span::raw("Returns 410 Gone page notice when expiration time elapses"),
            ]),
            Line::from(""),
            Line::from(Span::styled("Automatic Server Lifespan:", styles.accent)),
            Line::from(
                " The background server automatically manages item lifespans and prunes expired links.",
            ),
        ])
        .block(arch_block)
        .wrap(Wrap { trim: true });

        frame.render_widget(arch_info, chunks[1]);

        // Footer
        let footer = Paragraph::new(Line::from(Span::styled(
            " [Space] Toggle Redirect Server │ Server runs asynchronously in background ",
            styles.accent,
        )))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(styles.border),
        )
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }
}
