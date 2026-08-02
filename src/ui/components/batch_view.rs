use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use crate::ui::theme::ThemeStyles;

pub struct BatchViewState {
    pub csv_path_input: String,
    pub is_processing: bool,
    pub current_progress: usize,
    pub total_count: usize,
    pub log_messages: Vec<String>,
}

impl Default for BatchViewState {
    fn default() -> Self {
        Self {
            csv_path_input: "links.csv".to_string(),
            is_processing: false,
            current_progress: 0,
            total_count: 0,
            log_messages: Vec::new(),
        }
    }
}

pub struct BatchViewWidget;

impl BatchViewWidget {
    pub fn draw(frame: &mut Frame, area: Rect, state: &BatchViewState, styles: &ThemeStyles) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // CSV input
                Constraint::Length(3), // Progress bar
                Constraint::Min(6),    // Activity log
                Constraint::Length(3), // Footer hints
            ])
            .margin(1)
            .split(area);

        // CSV Path Input
        let csv_block = Block::default()
            .title(" CSV Batch Generation ")
            .borders(Borders::ALL)
            .border_style(styles.border_focus);

        let csv_p = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("CSV File Path: "),
                Span::styled(&state.csv_path_input, styles.primary),
            ]),
            Line::from(Span::styled(
                "CSV Format: url,label (e.g. https://google.com,google_qr)",
                styles.text_muted,
            )),
        ])
        .block(csv_block);
        frame.render_widget(csv_p, chunks[0]);

        // Progress Gauge
        let ratio = if state.total_count > 0 {
            (state.current_progress as f64 / state.total_count as f64).min(1.0)
        } else {
            0.0
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(format!(
                        " Progress ({}/{}) ",
                        state.current_progress, state.total_count
                    ))
                    .borders(Borders::ALL)
                    .border_style(styles.border),
            )
            .gauge_style(styles.tab_active)
            .ratio(ratio);
        frame.render_widget(gauge, chunks[1]);

        // Activity Log
        let log_lines: Vec<Line> = state
            .log_messages
            .iter()
            .rev()
            .take(15)
            .map(|msg| Line::from(Span::raw(msg)))
            .collect();

        let log_p = Paragraph::new(log_lines)
            .block(
                Block::default()
                    .title(" Processing Log ")
                    .borders(Borders::ALL)
                    .border_style(styles.border),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(log_p, chunks[2]);

        // Footer
        let footer = Paragraph::new(Line::from(Span::styled(
            " [Enter] Start Batch Generation │ [Esc] Clear Log │ Output saved to configured directory ",
            styles.accent,
        )))
        .block(Block::default().borders(Borders::TOP).border_style(styles.border))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }
}
