use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::watch;

use crate::config::AppConfig;
use crate::generator::{BatchGenerator, QrGenerator};
use crate::models::*;
use crate::services::*;
use crate::storage::HistoryStore;
use crate::ui::components::*;
use crate::utils::{FileOps, UrlValidator};

pub struct App {
    pub config: AppConfig,
    pub history_store: HistoryStore,
    pub active_tab: AppTab,
    pub wizard_state: StepWizardState,
    pub history_state: HistoryViewState,
    pub batch_state: BatchViewState,
    pub config_state: ConfigViewState,
    pub active_modal: Option<ModalDialog>,
    pub current_toast: Option<ToastNotification>,
    pub server_running: bool,
    pub redirect_provider: LocalRedirectProvider,
    pub server_shutdown_tx: Option<watch::Sender<bool>>,
    pub should_quit: bool,
    pub last_saved_file: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let history_store = HistoryStore::load();
        // Pass the same HistoryStore handle — it uses Arc<Mutex> internally
        // so the redirect server always sees newly added items.
        let redirect_provider = LocalRedirectProvider::new(history_store.clone());

        let mut app = Self {
            config,
            history_store,
            active_tab: AppTab::Wizard,
            wizard_state: StepWizardState::default(),
            history_state: HistoryViewState::default(),
            batch_state: BatchViewState::default(),
            config_state: ConfigViewState::default(),
            active_modal: None,
            current_toast: None,
            server_running: false,
            redirect_provider,
            server_shutdown_tx: None,
            should_quit: false,
            last_saved_file: None,
        };

        // Auto-start redirect server on 0.0.0.0 for mobile LAN scanning
        if app.config.dynamic_server_enabled {
            app.start_server();
        }

        app
    }

    pub fn toast(&mut self, message: &str, level: NotificationLevel) {
        self.current_toast = Some(ToastNotification::new(message.to_string(), level));
    }

    pub fn start_server(&mut self) {
        if self.server_running {
            return;
        }

        let initial_port = self.config.dynamic_server_port;
        let provider = self.redirect_provider.clone();

        // Perform async listener binding
        let bind_res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { RedirectServer::bind_listener(initial_port).await })
        });

        match bind_res {
            Ok((listener, bound_port)) => {
                let (tx, rx) = watch::channel(false);
                self.server_shutdown_tx = Some(tx);
                self.config.dynamic_server_port = bound_port;
                let lan_ip = crate::config::settings::detect_local_lan_ip();
                self.config.dynamic_server_host = format!("http://{}:{}", lan_ip, bound_port);
                let _ = self.config.save();

                tokio::spawn(async move {
                    let server = RedirectServer::new(provider);
                    let _ = server.run_with_listener(listener, rx).await;
                });

                self.server_running = true;
                self.toast(
                    &format!("Server active on {}", self.config.dynamic_server_host),
                    NotificationLevel::Success,
                );
            }
            Err(e) => {
                self.server_running = false;
                self.toast(
                    &format!("Failed to start server: {}", e),
                    NotificationLevel::Error,
                );
            }
        }
    }

    pub fn stop_server(&mut self) {
        if let Some(tx) = self.server_shutdown_tx.take() {
            let _ = tx.send(true);
        }
        self.server_running = false;
        self.toast("Redirect server stopped.", NotificationLevel::Info);
    }

    pub fn toggle_server(&mut self) {
        if self.server_running {
            self.stop_server();
        } else {
            self.start_server();
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(50);
        let mut last_tick = Instant::now();

        loop {
            // Dismiss toast if expired
            if let Some(ref toast) = self.current_toast {
                if toast.is_expired() {
                    self.current_toast = None;
                }
            }

            // Periodic server auto-expiration check:
            // Auto-stop background server when all dynamic items have expired
            if self.server_running {
                let items = self.history_store.items();
                let has_dynamic_items = items.iter().any(|i| i.dynamic_url.is_some());
                let has_active_dynamic = items
                    .iter()
                    .any(|i| i.dynamic_url.is_some() && !i.is_expired());
                if has_dynamic_items && !has_active_dynamic {
                    self.stop_server();
                    self.toast(
                        "Server auto-closed: link expiration period ended.",
                        NotificationLevel::Info,
                    );
                }
            }

            terminal.draw(|frame| crate::ui::draw::draw_app(frame, self))?;

            // CRITICAL: move the blocking crossterm poll into spawn_blocking
            // so it doesn't starve the tokio runtime (which runs the HTTP server).
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            let maybe_key = tokio::task::spawn_blocking(move || {
                if crossterm::event::poll(timeout).unwrap_or(false) {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == event::KeyEventKind::Press {
                            return Some(key);
                        }
                    }
                }
                None
            })
            .await?;

            if let Some(key) = maybe_key {
                self.handle_key_event(key).await;
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) {
        // Ctrl+C always quits immediately, like any normal terminal app
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        // Modal active interceptor
        if self.active_modal.is_some() {
            self.handle_modal_key(key).await;
            return;
        }

        // Search mode interceptor in History tab
        if self.active_tab == AppTab::History && self.history_state.is_searching {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.history_state.is_searching = false;
                }
                KeyCode::Backspace => {
                    self.history_state.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.history_state.search_query.push(c);
                }
                _ => {}
            }
            return;
        }

        // ALWAYS allow Tab / Shift+Tab to switch active tab unconditionally
        match key.code {
            KeyCode::Tab => {
                self.active_tab = AppTab::from_index((self.active_tab.index() + 1) % 5);
                return;
            }
            KeyCode::BackTab => {
                self.active_tab = AppTab::from_index((self.active_tab.index() + 4) % 5);
                return;
            }
            _ => {}
        }

        // Global shortcuts when NOT focused on an active text input box
        if !matches!(
            self.wizard_state.step,
            WizardStep::Step1Input
                | WizardStep::Step2CustomInput
                | WizardStep::Step4CustomPathInput
        ) {
            match key.code {
                KeyCode::Char('1') => self.active_tab = AppTab::Wizard,
                KeyCode::Char('2') => self.active_tab = AppTab::History,
                KeyCode::Char('3') => self.active_tab = AppTab::Batch,
                KeyCode::Char('4') => self.active_tab = AppTab::Server,
                KeyCode::Char('5') => self.active_tab = AppTab::Settings,
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    let next_theme_idx = (self.config.theme as usize + 1) % AppTheme::ALL.len();
                    self.config.theme = AppTheme::ALL[next_theme_idx];
                    let _ = self.config.save();
                    self.toast(
                        &format!("Theme changed to {}", self.config.theme.name()),
                        NotificationLevel::Info,
                    );
                }
                KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => {
                    self.active_modal = Some(ModalDialog::HelpOverlay);
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.should_quit = true;
                    return;
                }
                _ => {}
            }
        }

        // Tab-specific handlers
        match self.active_tab {
            AppTab::Wizard => self.handle_wizard_key(key).await,
            AppTab::History => self.handle_history_key(key).await,
            AppTab::Batch => self.handle_batch_key(key).await,
            AppTab::Server => {
                if key.code == KeyCode::Char(' ') {
                    self.toggle_server();
                }
            }
            AppTab::Settings => self.handle_config_key(key).await,
        }
    }

    async fn handle_modal_key(&mut self, key: KeyEvent) {
        if let Some(ModalDialog::ConfirmOverwrite { target_path }) = self.active_modal.clone() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.active_modal = None;
                    self.perform_save_file(Path::new(&target_path), true).await;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.active_modal = None;
                    self.toast("Save operation cancelled.", NotificationLevel::Info);
                }
                _ => {}
            }
        } else if let Some(ModalDialog::HelpOverlay) = self.active_modal {
            if matches!(
                key.code,
                KeyCode::Esc
                    | KeyCode::Char('?')
                    | KeyCode::Char('h')
                    | KeyCode::Char('H')
                    | KeyCode::Enter
            ) {
                self.active_modal = None;
            }
        }
    }

    async fn handle_wizard_key(&mut self, key: KeyEvent) {
        match self.wizard_state.step {
            WizardStep::Step1Input => {
                // Clipboard paste (Ctrl+V or Cmd+V)
                if (key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::SUPER))
                    && key.code == KeyCode::Char('v')
                {
                    if let Ok(pasted) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                        if self.wizard_state.target_mode_idx == 0 {
                            self.wizard_state.url_input.push_str(pasted.trim());
                        } else {
                            self.wizard_state.photo_path_input.push_str(pasted.trim());
                        }
                        self.validate_step1_input();
                        self.toast("Pasted into input field.", NotificationLevel::Info);
                    }
                    return;
                }

                match key.code {
                    KeyCode::Up | KeyCode::Down => {
                        self.wizard_state.target_mode_idx = 1 - self.wizard_state.target_mode_idx;
                    }
                    KeyCode::Char(c) => {
                        if self.wizard_state.target_mode_idx == 0 {
                            self.wizard_state.url_input.push(c);
                        } else {
                            self.wizard_state.photo_path_input.push(c);
                        }
                        self.validate_step1_input();
                    }
                    KeyCode::Backspace => {
                        if self.wizard_state.target_mode_idx == 0 {
                            self.wizard_state.url_input.pop();
                        } else {
                            self.wizard_state.photo_path_input.pop();
                        }
                        self.validate_step1_input();
                    }
                    KeyCode::Enter => {
                        if self.validate_step1_input() {
                            self.wizard_state.step = WizardStep::Step2Expiration;
                        } else {
                            if self.wizard_state.target_mode_idx == 0 {
                                self.toast(
                                    "Please enter a valid HTTP/HTTPS URL.",
                                    NotificationLevel::Error,
                                );
                            } else {
                                self.toast(
                                    "Photo file not found! Drag & drop an existing image file.",
                                    NotificationLevel::Error,
                                );
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.wizard_state.url_input.clear();
                        self.wizard_state.photo_path_input.clear();
                        self.wizard_state.input_validation_error = None;
                    }
                    _ => {}
                }
            }
            WizardStep::Step2Expiration => match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let digit = c.to_digit(10).unwrap_or(0) as usize;
                    if (1..=7).contains(&digit) {
                        self.wizard_state.selected_expiration_idx = digit - 1;
                        self.generate_qr_preview().await;
                        self.wizard_state.step = WizardStep::Step3Preview;
                    } else if digit == 8 {
                        self.wizard_state.selected_expiration_idx = 7;
                        self.wizard_state.step = WizardStep::Step2CustomInput;
                    }
                }
                KeyCode::Up => {
                    self.wizard_state.selected_expiration_idx =
                        self.wizard_state.selected_expiration_idx.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.wizard_state.selected_expiration_idx =
                        (self.wizard_state.selected_expiration_idx + 1).min(7);
                }
                KeyCode::Enter => {
                    if self.wizard_state.selected_expiration_idx == 7 {
                        self.wizard_state.step = WizardStep::Step2CustomInput;
                    } else {
                        self.generate_qr_preview().await;
                        self.wizard_state.step = WizardStep::Step3Preview;
                    }
                }
                KeyCode::Esc => {
                    self.wizard_state.step = WizardStep::Step1Input;
                }
                _ => {}
            },
            WizardStep::Step2CustomInput => match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    self.wizard_state.custom_expiration_value.push(c);
                }
                KeyCode::Backspace => {
                    self.wizard_state.custom_expiration_value.pop();
                }
                KeyCode::Left => {
                    self.wizard_state.custom_expiration_unit_idx = self
                        .wizard_state
                        .custom_expiration_unit_idx
                        .saturating_sub(1);
                }
                KeyCode::Right => {
                    self.wizard_state.custom_expiration_unit_idx =
                        (self.wizard_state.custom_expiration_unit_idx + 1).min(4);
                }
                KeyCode::Enter => {
                    self.generate_qr_preview().await;
                    self.wizard_state.step = WizardStep::Step3Preview;
                }
                KeyCode::Esc => {
                    self.wizard_state.step = WizardStep::Step2Expiration;
                }
                _ => {}
            },
            WizardStep::Step3Preview => match key.code {
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    self.wizard_state.ecc_level = match self.wizard_state.ecc_level {
                        EccLevel::Low => EccLevel::Medium,
                        EccLevel::Medium => EccLevel::Quartile,
                        EccLevel::Quartile => EccLevel::High,
                        EccLevel::High => EccLevel::Low,
                    };
                    self.generate_qr_preview().await;
                    self.toast(
                        &format!("Error Correction: {}", self.wizard_state.ecc_level.label()),
                        NotificationLevel::Info,
                    );
                }
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    self.wizard_state.export_format = match self.wizard_state.export_format {
                        ExportFormat::Png => ExportFormat::Svg,
                        ExportFormat::Svg => ExportFormat::Jpeg,
                        ExportFormat::Jpeg => ExportFormat::Ascii,
                        ExportFormat::Ascii => ExportFormat::Unicode,
                        ExportFormat::Unicode => ExportFormat::Png,
                    };
                    self.toast(
                        &format!("Format: {}", self.wizard_state.export_format.label()),
                        NotificationLevel::Info,
                    );
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    self.wizard_state.use_dynamic_qr = !self.wizard_state.use_dynamic_qr;
                    self.generate_qr_preview().await;
                    self.toast(
                        &format!(
                            "Dynamic QR: {}",
                            if self.wizard_state.use_dynamic_qr {
                                "ENABLED"
                            } else {
                                "DISABLED"
                            }
                        ),
                        NotificationLevel::Info,
                    );
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.wizard_state.quiet_zone = !self.wizard_state.quiet_zone;
                    self.generate_qr_preview().await;
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    self.wizard_state.transparent_bg = !self.wizard_state.transparent_bg;
                    self.generate_qr_preview().await;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if let Some(ref item) = self.wizard_state.generated_item {
                        if SystemClipboard::copy_text(item.encoded_url()).is_ok() {
                            self.toast(
                                "Link copied to system clipboard!",
                                NotificationLevel::Success,
                            );
                        } else {
                            self.toast(
                                "Failed to copy link to clipboard.",
                                NotificationLevel::Error,
                            );
                        }
                    }
                }
                KeyCode::Char('i') | KeyCode::Char('I') => {
                    if let Some(ref qr) = self.wizard_state.generated_qr {
                        let img =
                            QrGenerator::render_image(qr, 512, true, self.config.theme, false);
                        if SystemClipboard::copy_image(&img).is_ok() {
                            self.toast(
                                "QR Image copied to system clipboard!",
                                NotificationLevel::Success,
                            );
                        } else {
                            self.toast(
                                "Failed to copy image to clipboard.",
                                NotificationLevel::Error,
                            );
                        }
                    }
                }
                KeyCode::Enter => {
                    self.wizard_state.step = WizardStep::Step4SaveOption;
                }
                KeyCode::Esc => {
                    self.wizard_state.step = WizardStep::Step2Expiration;
                }
                _ => {}
            },
            WizardStep::Step4SaveOption => match key.code {
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Enter => {
                    let dir = self.config.get_resolved_output_dir();
                    let filename = if self.wizard_state.custom_filename.is_empty() {
                        FileOps::default_filename(self.wizard_state.export_format)
                    } else {
                        self.wizard_state.custom_filename.clone()
                    };
                    let target_path = dir.join(filename);

                    if FileOps::exists(&target_path) {
                        self.active_modal = Some(ModalDialog::ConfirmOverwrite {
                            target_path: target_path.to_string_lossy().to_string(),
                        });
                    } else {
                        self.perform_save_file(&target_path, false).await;
                    }
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    self.wizard_state.step = WizardStep::Step4CustomPathInput;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if let Some(ref item) = self.wizard_state.generated_item {
                        let _ = SystemClipboard::copy_text(item.encoded_url());
                        self.toast("Link copied to clipboard!", NotificationLevel::Success);
                    }
                }
                KeyCode::Char('i') | KeyCode::Char('I') => {
                    if let Some(ref qr) = self.wizard_state.generated_qr {
                        let img =
                            QrGenerator::render_image(qr, 512, true, self.config.theme, false);
                        let _ = SystemClipboard::copy_image(&img);
                        self.toast("QR Image copied to clipboard!", NotificationLevel::Success);
                    }
                }
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    if let Some(ref path) = self.last_saved_file {
                        if FileOps::open_file(path).is_ok() {
                            self.toast(
                                "Opened QR file in default viewer.",
                                NotificationLevel::Info,
                            );
                        } else {
                            self.toast("Failed to open file.", NotificationLevel::Error);
                        }
                    } else {
                        self.toast("Please save the QR code first.", NotificationLevel::Warning);
                    }
                }
                KeyCode::Char('r')
                | KeyCode::Char('R')
                | KeyCode::Char('f')
                | KeyCode::Char('F') => {
                    if let Some(ref path) = self.last_saved_file {
                        if FileOps::reveal_in_finder(path).is_ok() {
                            self.toast("Revealed QR file in Finder.", NotificationLevel::Info);
                        } else {
                            self.toast("Failed to reveal file.", NotificationLevel::Error);
                        }
                    } else {
                        self.toast("Please save the QR code first.", NotificationLevel::Warning);
                    }
                }
                KeyCode::Esc => {
                    self.wizard_state.step = WizardStep::Step3Preview;
                }
                _ => {}
            },
            WizardStep::Step4CustomPathInput => match key.code {
                KeyCode::Char(c) => {
                    self.wizard_state.custom_filename.push(c);
                }
                KeyCode::Backspace => {
                    self.wizard_state.custom_filename.pop();
                }
                KeyCode::Enter => {
                    self.wizard_state.step = WizardStep::Step4SaveOption;
                }
                KeyCode::Esc => {
                    self.wizard_state.step = WizardStep::Step4SaveOption;
                }
                _ => {}
            },
        }
    }

    async fn handle_history_key(&mut self, key: KeyEvent) {
        let items_len = self
            .history_store
            .search(&self.history_state.search_query)
            .len();
        match key.code {
            KeyCode::Up => {
                self.history_state.selected_idx = self.history_state.selected_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                if items_len > 0 {
                    self.history_state.selected_idx =
                        (self.history_state.selected_idx + 1).min(items_len - 1);
                }
            }
            KeyCode::Char('/') => {
                self.history_state.is_searching = true;
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                let search_results = self.history_store.search(&self.history_state.search_query);
                if let Some(item) = search_results.get(self.history_state.selected_idx) {
                    let id = item.id.clone();
                    if let Ok(new_fav) = self.history_store.toggle_favorite(&id) {
                        self.toast(
                            if new_fav {
                                "Marked as Favorite!"
                            } else {
                                "Removed from Favorites."
                            },
                            NotificationLevel::Info,
                        );
                    }
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                let search_results = self.history_store.search(&self.history_state.search_query);
                if let Some(item) = search_results.get(self.history_state.selected_idx) {
                    let _ = SystemClipboard::copy_text(item.encoded_url());
                    self.toast("URL copied to clipboard!", NotificationLevel::Success);
                }
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                let search_results = self.history_store.search(&self.history_state.search_query);
                if let Some(item) = search_results.get(self.history_state.selected_idx) {
                    if let Ok(qr) = QrGenerator::create_qr(item.encoded_url(), item.ecc_level) {
                        let img =
                            QrGenerator::render_image(&qr, 512, true, self.config.theme, false);
                        let _ = SystemClipboard::copy_image(&img);
                        self.toast("QR image copied to clipboard!", NotificationLevel::Success);
                    }
                }
            }
            KeyCode::Delete | KeyCode::Backspace => {
                let search_results = self.history_store.search(&self.history_state.search_query);
                if let Some(item) = search_results.get(self.history_state.selected_idx) {
                    let id = item.id.clone();
                    let _ = self.history_store.delete(&id);
                    self.toast("Item removed from history.", NotificationLevel::Info);
                }
            }
            _ => {}
        }
    }

    async fn handle_batch_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                let csv_path = PathBuf::from(&self.batch_state.csv_path_input);
                if !csv_path.exists() {
                    self.toast(
                        &format!("CSV file not found: {:?}", csv_path),
                        NotificationLevel::Error,
                    );
                    return;
                }

                let out_dir = self.config.get_resolved_output_dir();
                self.batch_state.is_processing = true;
                self.batch_state
                    .log_messages
                    .push(format!("Starting batch processing of {:?}", csv_path));

                let format = self.config.preferred_format;
                let ecc = self.config.default_ecc_level;
                let theme = self.config.theme;
                let size = self.config.default_qr_size;

                let res = BatchGenerator::process_csv(
                    &csv_path,
                    &out_dir,
                    format,
                    ecc,
                    theme,
                    size,
                    |curr, total, url| {
                        let _ = (curr, total, url);
                    },
                );

                match res {
                    Ok(batch_res) => {
                        self.batch_state.current_progress = batch_res.succeeded;
                        self.batch_state.total_count = batch_res.total;
                        self.batch_state.log_messages.push(format!(
                            "Batch Finished! Succeeded: {}, Failed: {}",
                            batch_res.succeeded, batch_res.failed
                        ));
                        self.toast(
                            &format!("Batch Complete! Saved {} QRs.", batch_res.succeeded),
                            NotificationLevel::Success,
                        );
                    }
                    Err(e) => {
                        self.batch_state
                            .log_messages
                            .push(format!("Batch Error: {}", e));
                        self.toast(&format!("Batch error: {}", e), NotificationLevel::Error);
                    }
                }
                self.batch_state.is_processing = false;
            }
            KeyCode::Esc => {
                self.batch_state.log_messages.clear();
                self.batch_state.current_progress = 0;
                self.batch_state.total_count = 0;
            }
            _ => {}
        }
    }

    async fn handle_config_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                self.config_state.selected_idx = self.config_state.selected_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                self.config_state.selected_idx = (self.config_state.selected_idx + 1).min(7);
            }
            KeyCode::Enter => match self.config_state.selected_idx {
                1 => {
                    self.config.default_qr_size = match self.config.default_qr_size {
                        256 => 512,
                        512 => 1024,
                        1024 => 2048,
                        _ => 256,
                    };
                    let _ = self.config.save();
                }
                2 => {
                    self.config.default_ecc_level = match self.config.default_ecc_level {
                        EccLevel::Low => EccLevel::Medium,
                        EccLevel::Medium => EccLevel::Quartile,
                        EccLevel::Quartile => EccLevel::High,
                        EccLevel::High => EccLevel::Low,
                    };
                    let _ = self.config.save();
                }
                3 => {
                    self.config.preferred_format = match self.config.preferred_format {
                        ExportFormat::Png => ExportFormat::Svg,
                        ExportFormat::Svg => ExportFormat::Jpeg,
                        ExportFormat::Jpeg => ExportFormat::Png,
                        _ => ExportFormat::Png,
                    };
                    let _ = self.config.save();
                }
                4 => {
                    let next = (self.config.theme as usize + 1) % AppTheme::ALL.len();
                    self.config.theme = AppTheme::ALL[next];
                    let _ = self.config.save();
                }
                5 => {
                    self.config.default_quiet_zone = !self.config.default_quiet_zone;
                    let _ = self.config.save();
                }
                6 => {
                    self.config.transparent_bg = !self.config.transparent_bg;
                    let _ = self.config.save();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn validate_step1_input(&mut self) -> bool {
        if self.wizard_state.target_mode_idx == 0 {
            match UrlValidator::validate(&self.wizard_state.url_input) {
                Ok(_) => {
                    self.wizard_state.input_validation_error = None;
                    true
                }
                Err(e) => {
                    self.wizard_state.input_validation_error = Some(e);
                    false
                }
            }
        } else {
            let path_obj = FileOps::sanitize_path(&self.wizard_state.photo_path_input);
            if path_obj.exists() && path_obj.is_file() {
                self.wizard_state.input_validation_error = None;
                true
            } else {
                self.wizard_state.input_validation_error = Some("File not found.".to_string());
                false
            }
        }
    }

    async fn generate_qr_preview(&mut self) {
        // Determine expiration
        let expiration_opt = if self.wizard_state.selected_expiration_idx < 7 {
            ExpirationOption::ALL_PRESETS[self.wizard_state.selected_expiration_idx].clone()
        } else {
            let val = self
                .wizard_state
                .custom_expiration_value
                .parse()
                .unwrap_or(30);
            let unit = match self.wizard_state.custom_expiration_unit_idx {
                0 => TimeUnit::Minutes,
                1 => TimeUnit::Hours,
                2 => TimeUnit::Days,
                3 => TimeUnit::Weeks,
                _ => TimeUnit::Months,
            };
            ExpirationOption::Custom { value: val, unit }
        };

        let short_code = uuid::Uuid::new_v4().to_string()[..8].to_string();

        let (target_type, display_url) = if self.wizard_state.target_mode_idx == 0 {
            let valid_url = match UrlValidator::validate(&self.wizard_state.url_input) {
                Ok(u) => u,
                Err(_) => return,
            };
            (TargetType::Url(valid_url.clone()), valid_url)
        } else {
            let path_obj = FileOps::sanitize_path(&self.wizard_state.photo_path_input);
            let clean_str = path_obj.to_string_lossy().to_string();
            let filename = path_obj
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            (
                TargetType::Photo {
                    file_path: clean_str.clone(),
                    filename: filename.clone(),
                },
                format!("file://{}", clean_str),
            )
        };

        let mut item = QrItem::new(
            display_url.clone(),
            target_type,
            short_code.clone(),
            None,
            expiration_opt.to_timestamp(),
            self.wizard_state.ecc_level,
            self.wizard_state.export_format,
        );

        let encoded_target = if self.wizard_state.use_dynamic_qr {
            if !self.server_running {
                self.start_server();
            }
            let dyn_url = format!("{}/r/{}", self.config.dynamic_server_host, short_code);
            item.dynamic_url = Some(dyn_url.clone());
            dyn_url
        } else {
            display_url.clone()
        };

        if let Ok(qr) = QrGenerator::create_qr(&encoded_target, self.wizard_state.ecc_level) {
            self.wizard_state.qr_unicode_preview =
                QrGenerator::render_unicode(&qr, self.wizard_state.quiet_zone);
            self.wizard_state.generated_qr = Some(qr);

            // Register the item in the shared history store NOW so the
            // redirect server can resolve the short code immediately
            // when a phone scans the QR code.
            let _ = self.history_store.add(item.clone());

            self.wizard_state.generated_item = Some(item);
        }
    }

    async fn perform_save_file(&mut self, target_path: &Path, _overwrite: bool) {
        if let (Some(ref qr), Some(ref item)) = (
            &self.wizard_state.generated_qr,
            &self.wizard_state.generated_item,
        ) {
            let format = self.wizard_state.export_format;
            let size = self.config.default_qr_size;
            let quiet = self.wizard_state.quiet_zone;
            let theme = self.config.theme;
            let transparent = self.wizard_state.transparent_bg;

            match QrGenerator::save_to_file(
                qr,
                target_path,
                format,
                size,
                quiet,
                theme,
                transparent,
            ) {
                Ok(_) => {
                    self.last_saved_file = Some(target_path.to_path_buf());
                    let mut saved_item = item.clone();
                    saved_item.last_saved_path = Some(target_path.to_string_lossy().to_string());
                    let _ = self.history_store.add(saved_item);

                    self.toast(
                        &format!(
                            "Saved QR to {:?}",
                            target_path.file_name().unwrap_or_default()
                        ),
                        NotificationLevel::Success,
                    );
                }
                Err(e) => {
                    self.toast(
                        &format!("Error saving file: {}", e),
                        NotificationLevel::Error,
                    );
                }
            }
        }
    }
}
