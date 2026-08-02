use ratatui::{layout::Rect, text::Line, widgets::Tabs, Frame};

use crate::ui::theme::ThemeStyles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Wizard,
    History,
    Batch,
    Server,
    Settings,
}

impl AppTab {
    pub const ALL: &'static [AppTab] = &[
        AppTab::Wizard,
        AppTab::History,
        AppTab::Batch,
        AppTab::Server,
        AppTab::Settings,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            AppTab::Wizard => " Wizard ",
            AppTab::History => " History ",
            AppTab::Batch => " Batch CSV ",
            AppTab::Server => " Server ",
            AppTab::Settings => " Settings ",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            AppTab::Wizard => 0,
            AppTab::History => 1,
            AppTab::Batch => 2,
            AppTab::Server => 3,
            AppTab::Settings => 4,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => AppTab::Wizard,
            1 => AppTab::History,
            2 => AppTab::Batch,
            3 => AppTab::Server,
            4 => AppTab::Settings,
            _ => AppTab::Wizard,
        }
    }
}

pub struct NavbarWidget;

impl NavbarWidget {
    pub fn draw(frame: &mut Frame, area: Rect, active_tab: AppTab, styles: &ThemeStyles) {
        let titles: Vec<Line> = AppTab::ALL.iter().map(|t| Line::from(t.title())).collect();

        let tabs = Tabs::new(titles)
            .select(active_tab.index())
            .style(styles.tab_inactive)
            .highlight_style(styles.tab_active);

        frame.render_widget(tabs, area);
    }
}
