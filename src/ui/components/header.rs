use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme::ThemeStyles;

pub struct HeaderWidget;

impl HeaderWidget {
    pub fn draw(frame: &mut Frame, area: Rect, styles: &ThemeStyles, server_running: bool) {
        let status_span = if server_running {
            Span::styled(
                " ● SERVER ACTIVE ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(" ○ SERVER OFF ", Style::default().fg(Color::DarkGray))
        };

        let banner_lines = vec![Line::from(vec![
            Span::styled("  ⚡ QR UTILITY  ", styles.primary),
            Span::styled(
                "│ Dynamic & Expiring Terminal QR Generator ",
                styles.text_muted,
            ),
            status_span,
            Span::styled("│ v0.1.0 ", styles.secondary),
        ])];

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(styles.border);

        let p = Paragraph::new(banner_lines)
            .block(block)
            .alignment(Alignment::Left);

        frame.render_widget(p, area);
    }
}
