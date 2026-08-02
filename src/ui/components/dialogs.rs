use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::ui::theme::ThemeStyles;

#[derive(Clone, Debug)]
pub enum ModalDialog {
    ConfirmOverwrite { target_path: String },
    HelpOverlay,
}

pub struct DialogWidget;

impl DialogWidget {
    pub fn draw_modal(frame: &mut Frame, area: Rect, dialog: &ModalDialog, styles: &ThemeStyles) {
        match dialog {
            ModalDialog::ConfirmOverwrite { target_path } => {
                let modal_area = Self::centered_rect(60, 25, area);
                frame.render_widget(Clear, modal_area);

                let block = Block::default()
                    .title(" ⚠️ File Already Exists ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(styles.error);

                let p = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(Span::styled("A file already exists at:", styles.text)),
                    Line::from(Span::styled(target_path, styles.primary)),
                    Line::from(""),
                    Line::from(Span::styled("Overwrite existing file?", styles.accent)),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(" [Y] / [Enter] ", styles.button_active),
                        Span::raw(" Yes, overwrite   "),
                        Span::styled(" [N] / [Esc] ", styles.button),
                        Span::raw(" Cancel"),
                    ]),
                ])
                .block(block)
                .alignment(Alignment::Center);

                frame.render_widget(p, modal_area);
            }
            ModalDialog::HelpOverlay => {
                let modal_area = Self::centered_rect(70, 75, area);
                frame.render_widget(Clear, modal_area);

                let block = Block::default()
                    .title(" ❓ QR Utility - Quick Help & Keyboard Shortcuts ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(styles.border_focus);

                let p = Paragraph::new(vec![
                    Line::from(Span::styled("Navigation & Global Keys:", styles.accent)),
                    Line::from(
                        " 1 - 5      Switch active tab (Wizard, History, Batch, Server, Settings)",
                    ),
                    Line::from(" Tab / Shift+Tab  Cycle through tabs"),
                    Line::from(" T          Cycle visual themes (Cyberpunk, Monokai, Ocean, etc.)"),
                    Line::from(" ? / H      Toggle this help modal"),
                    Line::from(" Q / Esc    Quit application"),
                    Line::from(""),
                    Line::from(Span::styled("Wizard Shortcuts:", styles.accent)),
                    Line::from(" Enter      Advance to next step"),
                    Line::from(" Ctrl+V     Paste URL from clipboard"),
                    Line::from(" 1 - 8      Select expiration preset"),
                    Line::from(
                        " E          Cycle error correction level (Low, Medium, Quartile, High)",
                    ),
                    Line::from(" F          Cycle export format (PNG, SVG, JPEG, ASCII, Unicode)"),
                    Line::from(" D          Toggle dynamic redirect mode"),
                    Line::from(" S          Save QR code to disk"),
                    Line::from(" C          Copy destination URL to clipboard"),
                    Line::from(" I          Copy QR image bytes to clipboard"),
                    Line::from(" O          Open saved file in OS viewer"),
                    Line::from(" R          Reveal saved file in macOS Finder"),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press [Esc] or [?] to close help.",
                        styles.text_muted,
                    )),
                ])
                .block(block)
                .wrap(Wrap { trim: true });

                frame.render_widget(p, modal_area);
            }
        }
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
                ratatui::layout::Constraint::Percentage(percent_y),
                ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
                ratatui::layout::Constraint::Percentage(percent_x),
                ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
