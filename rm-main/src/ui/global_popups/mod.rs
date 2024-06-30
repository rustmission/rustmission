mod error;
mod help;

use ratatui::prelude::*;

pub use error::ErrorPopup;
pub use help::HelpPopup;

use crate::app;
use rm_shared::action::Action;

use super::components::Component;

pub(super) struct GlobalPopupManager {
    pub error_popup: Option<ErrorPopup>,
    pub help_popup: Option<HelpPopup>,
    ctx: app::Ctx,
}

impl GlobalPopupManager {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            error_popup: None,
            help_popup: None,
            ctx,
        }
    }

    pub const fn needs_action(&self) -> bool {
        self.error_popup.is_some() || self.help_popup.is_some()
    }

    fn toggle_help(&mut self) -> Option<Action> {
        if self.help_popup.is_some() {
            self.help_popup = None;
        } else {
            self.help_popup = Some(HelpPopup::new(self.ctx.clone()));
        }
        Some(Action::Render)
    }

    fn handle_popups(&mut self, action: Action) -> Option<Action> {
        if let Some(popup) = &mut self.error_popup {
            if popup
                .handle_actions(action)
                .is_some_and(|a| a.is_soft_quit())
            {
                self.error_popup = None;
                return Some(Action::Render);
            }
        } else if let Some(popup) = &mut self.help_popup {
            if popup
                .handle_actions(action)
                .is_some_and(|a| a.is_soft_quit())
            {
                self.help_popup = None;
                return Some(Action::Render);
            }
        }
        None
    }
}

impl Component for GlobalPopupManager {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        if action == A::ShowHelp {
            return self.toggle_help();
        }

        self.handle_popups(action)
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        if let Some(popup) = &mut self.error_popup {
            popup.render(f, rect)
        } else if let Some(popup) = &mut self.help_popup {
            popup.render(f, rect);
        }
    }
}
