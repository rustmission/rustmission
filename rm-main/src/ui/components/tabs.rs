use crate::app;
use rm_shared::action::Action;

use super::Component;
use ratatui::{layout::Flex, prelude::*, widgets::Tabs};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CurrentTab {
    Torrents = 0,
    Search,
}

pub struct TabComponent {
    tabs_list: [&'static str; 2],
    pub current_tab: CurrentTab,
    ctx: app::Ctx,
}

impl TabComponent {
    pub fn new(ctx: app::Ctx) -> Self {
        let tabs_list = {
            if ctx.config.general.beginner_mode {
                ["1. Torrents", "2. Search"]
            } else {
                ["Torrents", "Search"]
            }
        };

        Self {
            ctx,
            tabs_list,
            current_tab: CurrentTab::Torrents,
        }
    }
}

impl Component for TabComponent {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let divider = symbols::DOT;

        let tabs_length =
            self.tabs_list.concat().chars().count() + divider.len() + self.tabs_list.len();

        let center_rect = Layout::horizontal([Constraint::Length(tabs_length.try_into().unwrap())])
            .flex(Flex::Center)
            .split(rect)[0];

        let tabs_highlight_style = Style::default().fg(self.ctx.config.general.accent_color);
        let tabs = Tabs::new(self.tabs_list)
            .style(Style::default().white())
            .highlight_style(tabs_highlight_style)
            .select(self.current_tab as usize)
            .divider(symbols::DOT);

        f.render_widget(tabs, center_rect);
    }

    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::ChangeTab(tab) => match tab {
                1 => self.current_tab = CurrentTab::Torrents,
                2 => self.current_tab = CurrentTab::Search,
                _ => (),
            },
            // left only works on right-most tab (search)
            Action::Left => match self.current_tab {
                CurrentTab::Search => self.current_tab = CurrentTab::Torrents,
                _ => (),
            },
            // right only works on left-most tab (torrents)
            Action::Right => match self.current_tab {
                CurrentTab::Torrents => self.current_tab = CurrentTab::Search,
                _ => (),
            },
            _ => (),
        }
        None
    }
}
