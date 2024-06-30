use crate::{app, ui::components::Component};

use ratatui::prelude::*;

pub struct DefaultBar {
    ctx: app::Ctx,
}

impl DefaultBar {
    pub const fn new(ctx: app::Ctx) -> Self {
        Self { ctx }
    }
}

impl Component for DefaultBar {
    fn render(&mut self, f: &mut ratatui::Frame<'_>, rect: Rect) {
        if self.ctx.config.general.beginner_mode {
            if let Some(keys) = self
                .ctx
                .config
                .keybindings
                .get_keys_for_action(rm_shared::action::Action::ShowHelp)
            {
                f.render_widget(format!("󰘥 {keys} - help"), rect)
            }
        }
    }
}
