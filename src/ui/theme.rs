use ratatui::style::{Color, Modifier, Style};

use crate::models::AppTheme;

pub struct ThemeStyles {
    pub primary: Style,
    pub secondary: Style,
    pub accent: Style,
    pub border: Style,
    pub border_focus: Style,
    pub text: Style,
    pub text_muted: Style,
    pub success: Style,
    pub error: Style,
    #[allow(dead_code)]
    pub warning: Style,
    #[allow(dead_code)]
    pub header: Style,
    pub tab_active: Style,
    pub tab_inactive: Style,
    pub button: Style,
    pub button_active: Style,
}

impl ThemeStyles {
    pub fn from_theme(theme: AppTheme) -> Self {
        let p_color = theme.primary();
        let s_color = theme.secondary();
        let a_color = theme.accent();
        let b_color = theme.border();

        Self {
            primary: Style::default().fg(p_color).add_modifier(Modifier::BOLD),
            secondary: Style::default().fg(s_color),
            accent: Style::default().fg(a_color).add_modifier(Modifier::BOLD),
            border: Style::default().fg(b_color),
            border_focus: Style::default().fg(p_color).add_modifier(Modifier::BOLD),
            text: Style::default().fg(Color::Reset),
            text_muted: Style::default().fg(Color::DarkGray),
            success: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            error: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            header: Style::default().fg(p_color).add_modifier(Modifier::BOLD),
            tab_active: Style::default()
                .fg(Color::Black)
                .bg(p_color)
                .add_modifier(Modifier::BOLD),
            tab_inactive: Style::default().fg(Color::Gray),
            button: Style::default().fg(Color::White).bg(b_color),
            button_active: Style::default()
                .fg(Color::Black)
                .bg(a_color)
                .add_modifier(Modifier::BOLD),
        }
    }
}
