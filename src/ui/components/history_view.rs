use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::storage::HistoryStore;
use crate::ui::theme::ThemeStyles;

#[derive(Default)]
pub struct HistoryViewState {
    pub selected_idx: usize,
    pub search_query: String,
    pub is_searching: bool,
}

pub struct HistoryViewWidget;

impl HistoryViewWidget {
    pub fn draw(
        frame: &mut Frame,
        area: Rect,
        store: &HistoryStore,
        state: &HistoryViewState,
        styles: &ThemeStyles,
    ) {
        let filtered_items = store.search(&state.search_query);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(10),   // List & Detail splits
                Constraint::Length(3), // Footer hints
            ])
            .split(area);

        // Search Bar
        let search_block = Block::default()
            .title(" Search History (/ to focus) ")
            .borders(Borders::ALL)
            .border_style(if state.is_searching {
                styles.border_focus
            } else {
                styles.border
            });

        let search_text = if state.search_query.is_empty() && !state.is_searching {
            Span::styled("Type to filter by URL or short code...", styles.text_muted)
        } else {
            Span::styled(&state.search_query, styles.primary)
        };

        let search_p = Paragraph::new(Line::from(vec![search_text])).block(search_block);
        frame.render_widget(search_p, chunks[0]);

        // List + Details layout
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(chunks[1]);

        let items: Vec<ListItem> = filtered_items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = idx == state.selected_idx;
                let fav_icon = if item.is_favorite { "★ " } else { "☆ " };
                let exp_status = item.remaining_time_str();

                let line = Line::from(vec![
                    Span::styled(
                        fav_icon,
                        if item.is_favorite {
                            styles.accent
                        } else {
                            styles.text_muted
                        },
                    ),
                    Span::styled(
                        crate::utils::UrlValidator::truncate(&item.original_url, 30),
                        if is_selected {
                            styles.button_active
                        } else {
                            styles.text
                        },
                    ),
                    Span::styled(format!(" [{}]", exp_status), styles.text_muted),
                ]);

                if is_selected {
                    ListItem::new(line).style(styles.button_active)
                } else {
                    ListItem::new(line)
                }
            })
            .collect();

        let list_block = Block::default()
            .title(format!(" QR History ({}) ", filtered_items.len()))
            .borders(Borders::ALL)
            .border_style(styles.border);

        let list = List::new(items).block(list_block);
        frame.render_widget(list, body_chunks[0]);

        // Details Panel
        let detail_block = Block::default()
            .title(" Item Details ")
            .borders(Borders::ALL)
            .border_style(styles.border);

        if let Some(selected_item) = filtered_items.get(state.selected_idx) {
            let details = vec![
                Line::from(vec![
                    Span::styled("Original URL: ", styles.accent),
                    Span::raw(&selected_item.original_url),
                ]),
                Line::from(vec![
                    Span::styled("Short Code: ", styles.accent),
                    Span::raw(&selected_item.short_code),
                ]),
                Line::from(vec![
                    Span::styled("Dynamic URL: ", styles.accent),
                    Span::raw(
                        selected_item
                            .dynamic_url
                            .as_deref()
                            .unwrap_or("None (Static)"),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Expiration: ", styles.accent),
                    Span::styled(
                        selected_item.remaining_time_str(),
                        if selected_item.is_expired() {
                            styles.error
                        } else {
                            styles.success
                        },
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Created At: ", styles.accent),
                    Span::raw(
                        selected_item
                            .created_at
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string(),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Saved Path: ", styles.accent),
                    Span::raw(
                        selected_item
                            .last_saved_path
                            .as_deref()
                            .unwrap_or("Not saved"),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled("Actions:", styles.accent)),
                Line::from(
                    " [F] Toggle Favorite  │ [C] Copy Link  │ [I] Copy Image  │ [Del] Delete",
                ),
            ];

            let p = Paragraph::new(details)
                .block(detail_block)
                .wrap(Wrap { trim: true });
            frame.render_widget(p, body_chunks[1]);
        } else {
            let p = Paragraph::new("No items found matching filter.")
                .block(detail_block)
                .style(styles.text_muted);
            frame.render_widget(p, body_chunks[1]);
        }

        // Footer
        let footer_p = Paragraph::new(Line::from(Span::styled(
            " [↑/↓] Navigate list │ [/] Search │ [F] Favorite │ [C] Copy URL │ [I] Copy Image │ [Del] Remove ",
            styles.accent,
        )))
        .block(Block::default().borders(Borders::TOP).border_style(styles.border))
        .alignment(Alignment::Center);

        frame.render_widget(footer_p, chunks[2]);
    }
}
