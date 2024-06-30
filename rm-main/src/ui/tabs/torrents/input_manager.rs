use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};
use tui_input::{Input, InputRequest};

use crate::{app, ui::components::Component};
use rm_shared::action::Action;

pub struct InputManager {
    input: Input,
    prompt: String,
    ctx: app::Ctx,
}

impl InputManager {
    pub fn new(ctx: app::Ctx, prompt: String) -> Self {
        Self {
            ctx,
            prompt,
            input: Input::default(),
        }
    }

    pub fn new_with_value(ctx: app::Ctx, prompt: String, value: String) -> Self {
        Self {
            ctx,
            prompt,
            input: Input::default().with_value(value),
        }
    }

    pub fn text(&self) -> String {
        self.input.to_string()
    }

    pub fn handle(&mut self, req: InputRequest) {
        self.input.handle(req);
    }
}

impl Component for InputManager {
    fn handle_actions(&mut self, _action: Action) -> Option<Action> {
        None
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(Clear, rect);

        let mut spans = vec![];

        spans.push(Span::styled(
            self.prompt.as_str(),
            Style::default().fg(self.ctx.config.general.accent_color),
        ));

        spans.push(Span::raw(self.text()));

        let input = self.input.to_string();
        let prefix_len = self.prompt.len() + self.text().len() - input.len();

        let paragraph = Paragraph::new(Line::from(spans));
        f.render_widget(paragraph, rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        f.set_cursor(rect.x + u16::try_from(cursor_offset).unwrap(), rect.y);
    }
}
