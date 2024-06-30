use self::{files::FilesPopup, stats::StatisticsPopup};
use crate::ui::components::Component;
use rm_shared::action::Action;

use ratatui::prelude::*;

pub mod files;
pub mod stats;

pub struct PopupManager {
    current_popup: Option<CurrentPopup>,
}

pub enum CurrentPopup {
    Stats(StatisticsPopup),
    Files(FilesPopup),
}

impl PopupManager {
    pub const fn new() -> Self {
        Self {
            current_popup: None,
        }
    }

    pub const fn is_showing_popup(&self) -> bool {
        self.current_popup.is_some()
    }

    pub fn show_popup(&mut self, popup: CurrentPopup) {
        self.current_popup = Some(popup);
    }

    pub fn close_popup(&mut self) {
        self.current_popup = None
    }
}

impl Component for PopupManager {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        if let Some(current_popup) = &mut self.current_popup {
            match current_popup {
                CurrentPopup::Stats(popup) => {
                    if popup
                        .handle_actions(action)
                        .is_some_and(|a| a.is_soft_quit())
                    {
                        self.close_popup();
                        return Some(Action::Render);
                    };
                }
                CurrentPopup::Files(popup) => {
                    if let Some(action) = popup.handle_actions(action) {
                        match action {
                            _ if action.is_soft_quit() => {
                                self.close_popup();
                                return Some(Action::Render);
                            }
                            Action::Render => return Some(Action::Render),
                            _ => (),
                        }
                    }
                }
            }
            return None;
        }
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(current_popup) = &mut self.current_popup {
            match current_popup {
                CurrentPopup::Stats(popup) => {
                    popup.render(f, rect);
                }
                CurrentPopup::Files(popup) => {
                    popup.render(f, rect);
                }
            }
        }
    }
}
