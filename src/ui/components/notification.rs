use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
    pub duration: Duration,
}

impl ToastNotification {
    pub fn new(message: String, level: NotificationLevel) -> Self {
        Self {
            message,
            level,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

pub struct NotificationWidget;

impl NotificationWidget {
    pub fn draw(frame: &mut Frame, toast: &ToastNotification, area: Rect) {
        let (border_color, title, icon) = match toast.level {
            NotificationLevel::Success => (Color::Green, " SUCCESS ", "✓ "),
            NotificationLevel::Error => (Color::Red, " ERROR ", "✗ "),
            NotificationLevel::Warning => (Color::Yellow, " WARNING ", "⚠ "),
            NotificationLevel::Info => (Color::Cyan, " INFO ", "ℹ "),
        };

        // Render toast in top-right or centered overlay
        let toast_width = (toast.message.len() + 10).min(area.width as usize - 4) as u16;
        let toast_height = 3;

        let toast_area = Rect {
            x: area.width.saturating_sub(toast_width + 2),
            y: 1,
            width: toast_width,
            height: toast_height,
        };

        frame.render_widget(Clear, toast_area);

        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            );

        let content = Paragraph::new(Line::from(vec![
            Span::styled(
                icon,
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&toast.message),
        ]))
        .block(block);

        frame.render_widget(content, toast_area);
    }
}
