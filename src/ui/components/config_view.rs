use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::config::AppConfig;
use crate::ui::theme::ThemeStyles;

#[derive(Default)]
pub struct ConfigViewState {
    pub selected_idx: usize,
}

pub struct ConfigViewWidget;

impl ConfigViewWidget {
    pub fn draw(
        frame: &mut Frame,
        area: Rect,
        config: &AppConfig,
        state: &ConfigViewState,
        styles: &ThemeStyles,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Options list
                Constraint::Length(3), // Footer hints
            ])
            .margin(1)
            .split(area);

        let fields = vec![
            ("Default Output Folder", config.default_output_dir.clone()),
            (
                "Default Image Size (px)",
                format!("{} px", config.default_qr_size),
            ),
            (
                "Default Error Correction",
                config.default_ecc_level.label().to_string(),
            ),
            (
                "Preferred Format",
                config.preferred_format.label().to_string(),
            ),
            ("Active Color Theme", config.theme.name().to_string()),
            (
                "Quiet Zone Margin",
                if config.default_quiet_zone {
                    "Enabled".to_string()
                } else {
                    "Disabled".to_string()
                },
            ),
            (
                "Transparent Background",
                if config.transparent_bg {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
            ),
            (
                "Redirect Server Port",
                config.dynamic_server_port.to_string(),
            ),
        ];

        let items: Vec<ListItem> = fields
            .into_iter()
            .enumerate()
            .map(|(idx, (label, val))| {
                let is_sel = idx == state.selected_idx;
                let prefix = if is_sel { "▶ " } else { "  " };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{}{:<28}", prefix, label),
                        if is_sel { styles.accent } else { styles.text },
                    ),
                    Span::styled(
                        val,
                        if is_sel {
                            styles.button_active
                        } else {
                            styles.secondary
                        },
                    ),
                ]);

                if is_sel {
                    ListItem::new(line).style(styles.button_active)
                } else {
                    ListItem::new(line)
                }
            })
            .collect();

        let list_block = Block::default()
            .title(" Configuration Settings (~/.config/qr-utility/config.toml) ")
            .borders(Borders::ALL)
            .border_style(styles.border_focus);

        let list = List::new(items).block(list_block);
        frame.render_widget(list, chunks[0]);

        let footer = Paragraph::new(Line::from(Span::styled(
            " [↑/↓] Navigate settings │ [T] Cycle Theme │ [Enter] Toggle Option │ Changes saved automatically ",
            styles.accent,
        )))
        .block(Block::default().borders(Borders::TOP).border_style(styles.border))
        .alignment(Alignment::Center);

        frame.render_widget(footer, chunks[1]);
    }
}
