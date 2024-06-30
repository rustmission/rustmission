pub mod components;
pub mod global_popups;
pub mod tabs;

use crate::ui::{global_popups::ErrorPopup, tabs::torrents::TorrentsTab};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use tui_input::InputRequest;

use rm_shared::action::Action;

use crate::app::{self};

use self::{
    components::{tabs::CurrentTab, Component, TabComponent},
    global_popups::GlobalPopupManager,
    tabs::search::SearchTab,
};

pub struct MainWindow {
    // TODO: make tabs hold torrents_tab and search_tab
    tabs: TabComponent,
    torrents_tab: TorrentsTab,
    search_tab: SearchTab,
    global_popup_manager: GlobalPopupManager,
}

impl MainWindow {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            tabs: TabComponent::new(ctx.clone()),
            torrents_tab: TorrentsTab::new(ctx.clone()),
            search_tab: SearchTab::new(ctx.clone()),
            global_popup_manager: GlobalPopupManager::new(ctx),
        }
    }
}

impl Component for MainWindow {
    // Rewrite this to one big match
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;

        match action {
            A::Error(error) => {
                let error_popup = ErrorPopup::new(&error.title, error.message);
                self.global_popup_manager.error_popup = Some(error_popup);
                Some(A::Render)
            }
            A::ShowHelp => self.global_popup_manager.handle_actions(action),
            _ if self.global_popup_manager.needs_action() => {
                self.global_popup_manager.handle_actions(action)
            }
            A::ChangeTab(_) | A::Left | A::Right => {
                self.tabs.handle_actions(action);
                Some(A::Render)
            }
            _ if self.tabs.current_tab == CurrentTab::Torrents => {
                self.torrents_tab.handle_actions(action)
            }
            _ if self.tabs.current_tab == CurrentTab::Search => {
                self.search_tab.handle_actions(action)
            }
            _ => unreachable!(),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_bar, main_window] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(rect);

        self.tabs.render(f, top_bar);

        match self.tabs.current_tab {
            CurrentTab::Torrents => self.torrents_tab.render(f, main_window),
            CurrentTab::Search => self.search_tab.render(f, main_window),
        }

        self.global_popup_manager.render(f, f.size());
    }
}

const fn to_input_request(key_event: KeyEvent) -> Option<InputRequest> {
    use InputRequest as R;

    match (key_event.code, key_event.modifiers) {
        (KeyCode::Backspace, KeyModifiers::ALT) => Some(R::DeletePrevWord),
        (KeyCode::Backspace, _) => Some(R::DeletePrevChar),
        (KeyCode::Delete, _) => Some(R::DeleteNextChar),
        (KeyCode::Char(char), _) => Some(R::InsertChar(char)),
        _ => None,
    }
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
