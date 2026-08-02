use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::ui::app::App;
use crate::ui::components::*;
use crate::ui::theme::ThemeStyles;

pub fn draw_app(frame: &mut Frame, app: &mut App) {
    let styles = ThemeStyles::from_theme(app.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header Banner
            Constraint::Length(1), // Navbar Tabs
            Constraint::Min(10),   // Main Tab Content
        ])
        .split(frame.area());

    // 1. Header Banner
    HeaderWidget::draw(frame, chunks[0], &styles, app.server_running);

    // 2. Navbar Tabs
    NavbarWidget::draw(frame, chunks[1], app.active_tab, &styles);

    // 3. Main Active Tab View
    match app.active_tab {
        AppTab::Wizard => {
            StepWizardWidget::draw(
                frame,
                chunks[2],
                &app.wizard_state,
                &styles,
                app.config.theme,
            );
        }
        AppTab::History => {
            HistoryViewWidget::draw(
                frame,
                chunks[2],
                &app.history_store,
                &app.history_state,
                &styles,
            );
        }
        AppTab::Batch => {
            BatchViewWidget::draw(frame, chunks[2], &app.batch_state, &styles);
        }
        AppTab::Server => {
            RedirectViewWidget::draw(
                frame,
                chunks[2],
                app.server_running,
                app.config.dynamic_server_port,
                &app.config.dynamic_server_host,
                &styles,
            );
        }
        AppTab::Settings => {
            ConfigViewWidget::draw(frame, chunks[2], &app.config, &app.config_state, &styles);
        }
    }

    // 4. Modal Dialogs (if open)
    if let Some(ref modal) = app.active_modal {
        DialogWidget::draw_modal(frame, frame.area(), modal, &styles);
    }

    // 5. Toast Notifications (if active)
    if let Some(ref toast) = app.current_toast {
        NotificationWidget::draw(frame, toast, frame.area());
    }
}
