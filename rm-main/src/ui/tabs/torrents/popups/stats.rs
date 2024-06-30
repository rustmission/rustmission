use ratatui::{
    prelude::*,
    style::Styled,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Clear, Paragraph,
    },
};
use transmission_rpc::types::SessionStats;

use crate::{
    app,
    ui::{centered_rect, components::Component},
    utils::bytes_to_human_format,
};
use rm_shared::action::Action;

pub struct StatisticsPopup {
    stats: SessionStats,
    ctx: app::Ctx,
}

impl StatisticsPopup {
    pub const fn new(ctx: app::Ctx, stats: SessionStats) -> Self {
        Self { ctx, stats }
    }
}

impl Component for StatisticsPopup {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match action {
            _ if action.is_soft_quit() => Some(action),
            A::Confirm => Some(Action::Close),
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 50, 50);
        let block_rect = popup_rect.inner(Margin::new(1, 1));
        let text_rect = block_rect.inner(Margin::new(3, 2));

        let title_style = Style::default().fg(self.ctx.config.general.accent_color);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Statistics ".set_style(title_style)))
            .title(
                Title::from(" [ CLOSE ] ".set_style(title_style.bold()))
                    .alignment(Alignment::Right)
                    .position(Position::Bottom),
            );

        let uploaded_bytes = self.stats.cumulative_stats.uploaded_bytes;
        let downloaded_bytes = self.stats.cumulative_stats.downloaded_bytes;
        let uploaded = bytes_to_human_format(uploaded_bytes);
        let downloaded = bytes_to_human_format(downloaded_bytes);
        let ratio = uploaded_bytes as f64 / downloaded_bytes as f64;
        let text = format!("Uploaded: {uploaded}\nDownloaded: {downloaded}\nRatio: {ratio:.2}");
        let paragraph = Paragraph::new(text);

        f.render_widget(Clear, popup_rect);
        f.render_widget(block, block_rect);
        f.render_widget(paragraph, text_rect);
    }
}
