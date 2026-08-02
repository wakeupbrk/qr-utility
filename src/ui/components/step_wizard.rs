use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::{AppTheme, EccLevel, ExpirationOption, ExportFormat, QrItem, TimeUnit};
use crate::ui::theme::ThemeStyles;
use crate::utils::UrlValidator;
use qrcode::QrCode;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Step1Input,
    Step2Expiration,
    Step2CustomInput,
    Step3Preview,
    Step4SaveOption,
    Step4CustomPathInput,
}

pub struct StepWizardState {
    pub step: WizardStep,
    pub target_mode_idx: usize, // 0 = URL, 1 = Photo File
    pub url_input: String,
    pub photo_path_input: String,
    pub input_validation_error: Option<String>,
    pub selected_expiration_idx: usize,
    pub custom_expiration_value: String,
    pub custom_expiration_unit_idx: usize,
    pub ecc_level: EccLevel,
    pub export_format: ExportFormat,
    pub use_dynamic_qr: bool,
    pub custom_filename: String,
    pub generated_qr: Option<QrCode>,
    pub generated_item: Option<QrItem>,
    pub qr_unicode_preview: String,
    pub quiet_zone: bool,
    pub transparent_bg: bool,
    #[allow(dead_code)]
    pub target_size: u32,
}

impl Default for StepWizardState {
    fn default() -> Self {
        Self {
            step: WizardStep::Step1Input,
            target_mode_idx: 0,
            url_input: String::new(),
            photo_path_input: String::new(),
            input_validation_error: None,
            selected_expiration_idx: 2, // 30 minutes default
            custom_expiration_value: "45".to_string(),
            custom_expiration_unit_idx: 0,
            ecc_level: EccLevel::Medium,
            export_format: ExportFormat::Png,
            use_dynamic_qr: true,
            custom_filename: String::new(),
            generated_qr: None,
            generated_item: None,
            qr_unicode_preview: String::new(),
            quiet_zone: true,
            transparent_bg: false,
            target_size: 512,
        }
    }
}

pub struct StepWizardWidget;

impl StepWizardWidget {
    pub fn draw(
        frame: &mut Frame,
        area: Rect,
        state: &StepWizardState,
        styles: &ThemeStyles,
        theme: AppTheme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Wizard progress indicator header
                Constraint::Min(10),   // Main step content
                Constraint::Length(3), // Footer key hints
            ])
            .split(area);

        Self::draw_step_header(frame, chunks[0], state.step, styles);

        match state.step {
            WizardStep::Step1Input => Self::draw_step1_input(frame, chunks[1], state, styles),
            WizardStep::Step2Expiration => {
                Self::draw_step2_expiration(frame, chunks[1], state, styles)
            }
            WizardStep::Step2CustomInput => {
                Self::draw_step2_custom(frame, chunks[1], state, styles)
            }
            WizardStep::Step3Preview => {
                Self::draw_step3_preview(frame, chunks[1], state, styles, theme)
            }
            WizardStep::Step4SaveOption | WizardStep::Step4CustomPathInput => {
                Self::draw_step4_save(frame, chunks[1], state, styles)
            }
        }

        Self::draw_wizard_footer(frame, chunks[2], state.step, styles);
    }

    fn draw_step_header(
        frame: &mut Frame,
        area: Rect,
        current_step: WizardStep,
        styles: &ThemeStyles,
    ) {
        let step1_style = if current_step == WizardStep::Step1Input {
            styles.tab_active
        } else {
            styles.tab_inactive
        };
        let step2_style = if matches!(
            current_step,
            WizardStep::Step2Expiration | WizardStep::Step2CustomInput
        ) {
            styles.tab_active
        } else {
            styles.tab_inactive
        };
        let step3_style = if current_step == WizardStep::Step3Preview {
            styles.tab_active
        } else {
            styles.tab_inactive
        };
        let step4_style = if matches!(
            current_step,
            WizardStep::Step4SaveOption | WizardStep::Step4CustomPathInput
        ) {
            styles.tab_active
        } else {
            styles.tab_inactive
        };

        let steps_line = Line::from(vec![
            Span::styled(" Step 1: Target (URL / Photo) ", step1_style),
            Span::raw(" ➔ "),
            Span::styled(" Step 2: Expiration ", step2_style),
            Span::raw(" ➔ "),
            Span::styled(" Step 3: QR Preview ", step3_style),
            Span::raw(" ➔ "),
            Span::styled(" Step 4: Save & Export ", step4_style),
        ]);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(styles.border);

        let p = Paragraph::new(steps_line)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(p, area);
    }

    fn draw_step1_input(
        frame: &mut Frame,
        area: Rect,
        state: &StepWizardState,
        styles: &ThemeStyles,
    ) {
        let block = Block::default()
            .title(" Step 1: Target Choice (URL or Photo) ")
            .borders(Borders::ALL)
            .border_style(styles.border_focus);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Mode Selector (URL vs Photo)
                Constraint::Length(2), // Prompt text
                Constraint::Length(3), // Input box
                Constraint::Length(2), // Validation feedback
                Constraint::Min(2),    // Instructions / Drag & Drop instructions
            ])
            .margin(1)
            .split(area);

        frame.render_widget(block, area);

        // Mode Selector Bar
        let mode1_style = if state.target_mode_idx == 0 {
            styles.button_active
        } else {
            styles.button
        };
        let mode2_style = if state.target_mode_idx == 1 {
            styles.button_active
        } else {
            styles.button
        };

        let mode_p = Paragraph::new(Line::from(vec![
            Span::styled(" [↑/↓] Switch Mode: ", styles.accent),
            Span::styled(" [1] Web URL Link ", mode1_style),
            Span::raw("   "),
            Span::styled(" [2] Drag & Drop Photo File ", mode2_style),
        ]));
        frame.render_widget(mode_p, inner_layout[0]);

        if state.target_mode_idx == 0 {
            // URL Mode
            let prompt =
                Paragraph::new("Paste or type destination URL (e.g. https://example.com):")
                    .style(styles.text);
            frame.render_widget(prompt, inner_layout[1]);

            let input_block = Block::default().borders(Borders::ALL).border_style(
                if state.input_validation_error.is_none() && !state.url_input.is_empty() {
                    styles.success
                } else if state.input_validation_error.is_some() {
                    styles.error
                } else {
                    styles.border
                },
            );

            let input_text = if state.url_input.is_empty() {
                Span::styled("https://example.com", styles.text_muted)
            } else {
                Span::styled(&state.url_input, styles.primary)
            };

            let input_p = Paragraph::new(Line::from(vec![input_text])).block(input_block);
            frame.render_widget(input_p, inner_layout[2]);

            let validation_p = if let Some(ref err) = state.input_validation_error {
                Paragraph::new(Line::from(vec![
                    Span::styled(" ✗ ", styles.error),
                    Span::styled(err, styles.error),
                ]))
            } else if !state.url_input.is_empty() {
                if let Ok(valid) = UrlValidator::validate(&state.url_input) {
                    Paragraph::new(Line::from(vec![
                        Span::styled(" ✓ Valid URL: ", styles.success),
                        Span::raw(valid),
                    ]))
                } else {
                    Paragraph::new("")
                }
            } else {
                Paragraph::new("")
            };
            frame.render_widget(validation_p, inner_layout[3]);

            let tips = Paragraph::new(vec![
                Line::from(Span::styled("URL Tips:", styles.accent)),
                Line::from(" • Press [Enter] to validate & proceed to Expiration choice"),
                Line::from(" • Press [Tab] or [2] to switch to Photo File Share mode"),
            ])
            .style(styles.text_muted);
            frame.render_widget(tips, inner_layout[4]);
        } else {
            // Photo File Mode
            let prompt =
                Paragraph::new("Drag & drop a photo/image file into terminal, or type/paste path:")
                    .style(styles.text);
            frame.render_widget(prompt, inner_layout[1]);

            let clean_path = state.photo_path_input.trim_matches('\'').trim_matches('"');
            let path_obj = Path::new(clean_path);
            let exists = !clean_path.is_empty() && path_obj.exists() && path_obj.is_file();

            let input_block = Block::default()
                .borders(Borders::ALL)
                .border_style(if exists {
                    styles.success
                } else if !clean_path.is_empty() {
                    styles.error
                } else {
                    styles.border
                });

            let input_text = if state.photo_path_input.is_empty() {
                Span::styled(
                    "Drop image file here... (e.g. ~/Downloads/photo.jpg)",
                    styles.text_muted,
                )
            } else {
                Span::styled(&state.photo_path_input, styles.primary)
            };

            let input_p = Paragraph::new(Line::from(vec![input_text])).block(input_block);
            frame.render_widget(input_p, inner_layout[2]);

            let validation_p = if exists {
                Paragraph::new(Line::from(vec![
                    Span::styled(" ✓ Photo Ready: ", styles.success),
                    Span::raw(format!(
                        "{:?} ({} bytes)",
                        path_obj.file_name().unwrap_or_default(),
                        path_obj.metadata().map(|m| m.len()).unwrap_or(0)
                    )),
                ]))
            } else if !clean_path.is_empty() {
                Paragraph::new(Line::from(vec![
                    Span::styled(" ✗ File Not Found: ", styles.error),
                    Span::raw(clean_path),
                ]))
            } else {
                Paragraph::new("")
            };
            frame.render_widget(validation_p, inner_layout[3]);

            let tips = Paragraph::new(vec![
                Line::from(Span::styled("📷 Photo Sharing via QR Code:", styles.accent)),
                Line::from(" • Simply drag any PNG, JPEG, GIF, WEBP photo directly into this terminal window!"),
                Line::from(" • When scanned on mobile Wi-Fi, displays a beautiful responsive photo viewing web page."),
                Line::from(" • Enforces link expiration timers (e.g., photo QR self-destructs after 10m or 1h)."),
            ])
            .style(styles.text_muted);
            frame.render_widget(tips, inner_layout[4]);
        }
    }

    fn draw_step2_expiration(
        frame: &mut Frame,
        area: Rect,
        state: &StepWizardState,
        styles: &ThemeStyles,
    ) {
        let block = Block::default()
            .title(" Step 2: Choose Expiration Period ")
            .borders(Borders::ALL)
            .border_style(styles.border_focus);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(1)
            .split(area);

        frame.render_widget(block, area);

        let options = ExpirationOption::ALL_PRESETS;
        let mut items: Vec<ListItem> = options
            .iter()
            .enumerate()
            .map(|(idx, opt)| {
                let prefix = if idx == state.selected_expiration_idx {
                    "▶ "
                } else {
                    "  "
                };
                let line = format!("{}) {}{}", idx + 1, prefix, opt.label());
                if idx == state.selected_expiration_idx {
                    ListItem::new(line).style(styles.button_active)
                } else {
                    ListItem::new(line).style(styles.text)
                }
            })
            .collect();

        // 8) Custom
        let custom_prefix = if state.selected_expiration_idx == 7 {
            "▶ "
        } else {
            "  "
        };
        let custom_line = format!(
            "8) {}Custom Expiration (minutes/hours/days...)",
            custom_prefix
        );
        if state.selected_expiration_idx == 7 {
            items.push(ListItem::new(custom_line).style(styles.button_active));
        } else {
            items.push(ListItem::new(custom_line).style(styles.text));
        }

        let list = List::new(items).block(
            Block::default()
                .title(" Expiration Presets ")
                .borders(Borders::ALL)
                .border_style(styles.border),
        );
        frame.render_widget(list, chunks[0]);

        // Right panel details
        let selected_opt = if state.selected_expiration_idx < 7 {
            options[state.selected_expiration_idx].clone()
        } else {
            ExpirationOption::Custom {
                value: state.custom_expiration_value.parse().unwrap_or(30),
                unit: TimeUnit::Minutes,
            }
        };

        let timestamp_str = selected_opt
            .to_timestamp()
            .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| "Never (Permanent)".to_string());

        let remaining_str = selected_opt
            .to_timestamp()
            .map(|t| crate::models::ExpirationOption::format_remaining(Some(t)))
            .unwrap_or_else(|| "Infinite duration".to_string());

        let details = Paragraph::new(vec![
            Line::from(Span::styled("Selected Expiration:", styles.accent)),
            Line::from(vec![
                Span::raw(" Label: "),
                Span::styled(selected_opt.label(), styles.primary),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(" Expiration Timestamp: "),
                Span::styled(timestamp_str, styles.secondary),
            ]),
            Line::from(vec![
                Span::raw(" Active Duration: "),
                Span::styled(remaining_str, styles.success),
            ]),
            Line::from(""),
            Line::from(Span::styled("Note on Dynamic QRs & Photos:", styles.accent)),
            Line::from(" Dynamic QR codes & photos encode a local server link."),
            Line::from(" Scans after expiration receive a stylish 410 Expired notice."),
        ])
        .block(
            Block::default()
                .title(" Expiration Summary ")
                .borders(Borders::ALL)
                .border_style(styles.border),
        )
        .wrap(Wrap { trim: true });

        frame.render_widget(details, chunks[1]);
    }

    fn draw_step2_custom(
        frame: &mut Frame,
        area: Rect,
        state: &StepWizardState,
        styles: &ThemeStyles,
    ) {
        let block = Block::default()
            .title(" Step 2: Custom Expiration ")
            .borders(Borders::ALL)
            .border_style(styles.border_focus);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Value input
                Constraint::Length(5), // Unit selection
                Constraint::Min(2),    // Instructions
            ])
            .margin(1)
            .split(area);

        frame.render_widget(block, area);

        let val_p = Paragraph::new(Line::from(vec![
            Span::raw("Enter amount: "),
            Span::styled(&state.custom_expiration_value, styles.primary),
        ]))
        .block(
            Block::default()
                .title(" Value ")
                .borders(Borders::ALL)
                .border_style(styles.border_focus),
        );
        frame.render_widget(val_p, layout[0]);

        let units = [
            TimeUnit::Minutes,
            TimeUnit::Hours,
            TimeUnit::Days,
            TimeUnit::Weeks,
            TimeUnit::Months,
        ];

        let unit_spans: Vec<Span> = units
            .iter()
            .enumerate()
            .map(|(idx, u)| {
                if idx == state.custom_expiration_unit_idx {
                    Span::styled(format!(" [{}] ", u), styles.button_active)
                } else {
                    Span::styled(format!("  {}  ", u), styles.text_muted)
                }
            })
            .collect();

        let unit_p = Paragraph::new(Line::from(unit_spans)).block(
            Block::default()
                .title(" Unit (Use Left/Right arrows) ")
                .borders(Borders::ALL)
                .border_style(styles.border),
        );
        frame.render_widget(unit_p, layout[1]);

        let help = Paragraph::new("Press [Enter] to confirm custom expiration and proceed.");
        frame.render_widget(help, layout[2]);
    }

    fn draw_step3_preview(
        frame: &mut Frame,
        area: Rect,
        state: &StepWizardState,
        styles: &ThemeStyles,
        theme: AppTheme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        // Left Panel: Terminal QR Code Double-Density Canvas
        let qr_block = Block::default()
            .title(" Step 3: Terminal QR Code Preview ")
            .borders(Borders::ALL)
            .border_style(styles.border_focus);

        let preview_content = Paragraph::new(state.qr_unicode_preview.as_str())
            .alignment(Alignment::Center)
            .block(qr_block)
            .style(Style::default().fg(theme.primary()));

        frame.render_widget(preview_content, chunks[0]);

        // Right Panel: Metadata & Customization
        let meta_block = Block::default()
            .title(" Configuration & Details ")
            .borders(Borders::ALL)
            .border_style(styles.border);

        let mut lines = Vec::new();

        if state.target_mode_idx == 0 {
            lines.push(Line::from(vec![
                Span::styled("✓ Target URL: ", styles.success),
                Span::styled(
                    crate::utils::UrlValidator::truncate(&state.url_input, 35),
                    styles.text,
                ),
            ]));
        } else {
            let clean = state.photo_path_input.trim_matches('\'').trim_matches('"');
            let name = Path::new(clean)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            lines.push(Line::from(vec![
                Span::styled("✓ Photo Target: ", styles.success),
                Span::styled(name.to_string(), styles.text),
            ]));
        }

        if let Some(ref item) = state.generated_item {
            lines.push(Line::from(vec![
                Span::styled("✓ Expiration: ", styles.success),
                Span::styled(item.remaining_time_str(), styles.accent),
            ]));
            lines.push(Line::from(vec![
                Span::styled("✓ Mode: ", styles.success),
                Span::styled(
                    if state.use_dynamic_qr {
                        "Dynamic Expiring (Local LAN Redirect Server)"
                    } else {
                        "Static Direct URL"
                    },
                    styles.secondary,
                ),
            ]));
            if let Some(ref d_url) = item.dynamic_url {
                lines.push(Line::from(vec![
                    Span::styled("✓ Scannable Link: ", styles.success),
                    Span::styled(d_url, styles.primary),
                ]));
            }
        }

        lines.extend(vec![
            Line::from(""),
            Line::from(Span::styled("Customization Options:", styles.accent)),
            Line::from(vec![
                Span::raw(" [E] Error Correction: "),
                Span::styled(state.ecc_level.label(), styles.primary),
            ]),
            Line::from(vec![
                Span::raw(" [F] Export Format: "),
                Span::styled(state.export_format.label(), styles.secondary),
            ]),
            Line::from(vec![
                Span::raw(" [T] Color Palette: "),
                Span::styled(theme.name(), styles.accent),
            ]),
            Line::from(vec![
                Span::raw(" [Q] Quiet Zone Margin: "),
                Span::styled(
                    if state.quiet_zone {
                        "Enabled (4 modules)"
                    } else {
                        "Disabled"
                    },
                    styles.text,
                ),
            ]),
            Line::from(vec![
                Span::raw(" [B] Transparent Background: "),
                Span::styled(if state.transparent_bg { "Yes" } else { "No" }, styles.text),
            ]),
            Line::from(""),
            Line::from(Span::styled("Shortcuts:", styles.accent)),
            Line::from(" [C] Copy Link  │ [I] Copy QR Image  │ [Enter] Proceed to Save"),
        ]);

        let p = Paragraph::new(lines)
            .block(meta_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(p, chunks[1]);
    }

    fn draw_step4_save(
        frame: &mut Frame,
        area: Rect,
        state: &StepWizardState,
        styles: &ThemeStyles,
    ) {
        let block = Block::default()
            .title(" Step 4: Save & Export Options ")
            .borders(Borders::ALL)
            .border_style(styles.border_focus);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Primary Save Prompt
                Constraint::Length(4), // Target Path Info
                Constraint::Min(6),    // Action Options
            ])
            .margin(1)
            .split(area);

        frame.render_widget(block, area);

        let default_file = if state.custom_filename.is_empty() {
            crate::utils::FileOps::default_filename(state.export_format)
        } else {
            state.custom_filename.clone()
        };

        let prompt_p = Paragraph::new(vec![
            Line::from(Span::styled("Save QR Code File?", styles.accent)),
            Line::from(vec![
                Span::raw(" Target Format: "),
                Span::styled(state.export_format.label(), styles.primary),
            ]),
        ]);
        frame.render_widget(prompt_p, chunks[0]);

        let path_block = Block::default()
            .title(" Output File ")
            .borders(Borders::ALL)
            .border_style(if state.step == WizardStep::Step4CustomPathInput {
                styles.border_focus
            } else {
                styles.border
            });

        let path_text = if state.step == WizardStep::Step4CustomPathInput {
            format!("Custom path: {}", state.custom_filename)
        } else {
            format!("Filename: {}", default_file)
        };

        let path_p = Paragraph::new(path_text).block(path_block);
        frame.render_widget(path_p, chunks[1]);

        let actions = Paragraph::new(vec![
            Line::from(Span::styled("Available Actions:", styles.accent)),
            Line::from(vec![
                Span::styled(" [S] / [Enter] ", styles.button_active),
                Span::raw(
                    " Save file to default directory (~/Desktop/projects/tool-utilities/qrcodes/)",
                ),
            ]),
            Line::from(vec![
                Span::styled(" [P] ", styles.button),
                Span::raw(" Edit custom output path/filename"),
            ]),
            Line::from(vec![
                Span::styled(" [C] ", styles.button),
                Span::raw(" Copy Link to System Clipboard"),
            ]),
            Line::from(vec![
                Span::styled(" [I] ", styles.button),
                Span::raw(" Copy Image bytes to Clipboard"),
            ]),
            Line::from(vec![
                Span::styled(" [O] ", styles.button),
                Span::raw(" Open saved file in default OS viewer"),
            ]),
            Line::from(vec![
                Span::styled(" [F] ", styles.button),
                Span::raw(" Reveal saved file in macOS Finder"),
            ]),
        ]);

        frame.render_widget(actions, chunks[2]);
    }

    fn draw_wizard_footer(frame: &mut Frame, area: Rect, step: WizardStep, styles: &ThemeStyles) {
        let text = match step {
            WizardStep::Step1Input => {
                " [1] URL Mode │ [2] Photo Mode │ [Enter] Next Step │ [Esc] Clear "
            }
            WizardStep::Step2Expiration => {
                " [1-8] Select Preset │ [↑/↓] Navigate │ [Enter] Confirm "
            }
            WizardStep::Step2CustomInput => " [0-9] Value │ [←/→] Change Unit │ [Enter] Confirm ",
            WizardStep::Step3Preview => {
                " [Enter] Save Options │ [E] ECC │ [F] Format │ [T] Theme │ [D] Dynamic "
            }
            WizardStep::Step4SaveOption | WizardStep::Step4CustomPathInput => {
                " [S/Enter] Save Image │ [P] Path │ [C] Copy Link │ [I] Copy Image │ [Esc] Back "
            }
        };

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(styles.border);

        let p = Paragraph::new(Line::from(Span::styled(text, styles.accent)))
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(p, area);
    }
}
