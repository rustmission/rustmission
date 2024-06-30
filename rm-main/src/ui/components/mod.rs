pub mod table;
pub mod tabs;

use ratatui::prelude::*;
use ratatui::Frame;

use rm_shared::action::Action;
pub use tabs::TabComponent;

pub trait Component {
    fn handle_actions(&mut self, _action: Action) -> Option<Action> {
        None
    }
    fn render(&mut self, _f: &mut Frame, _rect: Rect) {}
    fn tick(&mut self) -> Option<Action> {
        None
    }
}
