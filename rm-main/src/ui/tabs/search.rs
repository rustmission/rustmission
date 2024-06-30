use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use crossterm::event::{KeyCode, KeyEvent};
use magnetease::{Magnet, Magnetease};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Cell, Paragraph, Row, Table},
};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::{self, UnboundedSender};
use tui_input::Input;

use crate::{
    app,
    transmission::TorrentAction,
    ui::{
        components::{table::GenericTable, Component},
        to_input_request,
    },
    utils::bytes_to_human_format,
};
use rm_shared::action::Action;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SearchFocus {
    Search,
    List,
}

pub(crate) struct SearchTab {
    search_focus: SearchFocus,
    input: Input,
    req_sender: UnboundedSender<String>,
    table: Arc<Mutex<GenericTable<Magnet>>>,
    // TODO: Change it to enum, and combine table with search_result_info, to be behind one mutex
    search_result_info: Arc<Mutex<SearchResultState>>,
    currently_displaying_no: u16,
    ctx: app::Ctx,
}

impl SearchTab {
    pub(crate) fn new(ctx: app::Ctx) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let table = Arc::new(Mutex::new(GenericTable::new(vec![])));
        let search_result_info = Arc::new(Mutex::new(SearchResultState::new()));

        let ctx_clone = ctx.clone();
        let table_clone = Arc::clone(&table);
        let search_result_info_clone = Arc::clone(&search_result_info);
        tokio::task::spawn(async move {
            let magnetease = Magnetease::new();
            while let Some(search_phrase) = rx.recv().await {
                search_result_info_clone
                    .lock()
                    .unwrap()
                    .searching(Arc::new(Mutex::new(ThrobberState::default())));
                ctx_clone.send_action(Action::Render);
                let res = magnetease.search(&search_phrase).await.unwrap();
                if res.is_empty() {
                    search_result_info_clone.lock().unwrap().not_found();
                } else {
                    search_result_info_clone.lock().unwrap().found(res.len());
                }

                table_clone.lock().unwrap().set_items(res);
                ctx_clone.send_action(Action::Render);
            }
        });

        Self {
            search_focus: SearchFocus::List,
            input: Input::default(),
            table,
            search_result_info,
            req_sender: tx,
            currently_displaying_no: 0,
            ctx,
        }
    }

    fn magnet_to_row(magnet: &Magnet) -> Row {
        let size = bytes_to_human_format(magnet.bytes as i64);
        Row::new([
            Cell::from(Cow::Owned(magnet.seeders.to_string())).light_green(),
            Cell::from(Cow::Borrowed(&*magnet.title)),
            Cell::from(Cow::Owned(size)),
        ])
    }

    #[must_use]
    fn change_focus(&mut self) -> Option<Action> {
        if self.search_focus == SearchFocus::Search {
            self.search_focus = SearchFocus::List;
        } else {
            self.search_focus = SearchFocus::Search;
        }
        Some(Action::Render)
    }

    fn add_torrent(&mut self) -> Option<Action> {
        let magnet_url = self
            .table
            .lock()
            .unwrap()
            .current_item()
            .map(|magnet| magnet.url);
        if let Some(magnet_url) = magnet_url {
            self.ctx
                .send_torrent_action(TorrentAction::Add(magnet_url, None));
        }
        None
    }

    fn handle_input(&mut self, input: KeyEvent) -> Option<Action> {
        use Action as A;

        match input.code {
            KeyCode::Enter => {
                self.req_sender.send(self.input.to_string()).unwrap();
                self.search_focus = SearchFocus::List;
                Some(A::SwitchToNormalMode)
            }
            KeyCode::Esc => {
                self.search_focus = SearchFocus::List;
                Some(A::SwitchToNormalMode)
            }
            _ => {
                if let Some(req) = to_input_request(input) {
                    self.input.handle(req);
                    Some(A::Render)
                } else {
                    None
                }
            }
        }
    }

    fn start_search(&mut self) -> Option<Action> {
        self.search_focus = SearchFocus::Search;
        Some(Action::SwitchToInputMode)
    }

    fn next_torrent(&mut self) -> Option<Action> {
        self.table.lock().unwrap().next();
        Some(Action::Render)
    }

    fn previous_torrent(&mut self) -> Option<Action> {
        self.table.lock().unwrap().previous();
        Some(Action::Render)
    }

    fn scroll_down_page(&mut self) -> Option<Action> {
        self.table
            .lock()
            .unwrap()
            .scroll_down_by(self.currently_displaying_no as usize);
        Some(Action::Render)
    }

    fn scroll_up_page(&mut self) -> Option<Action> {
        self.table
            .lock()
            .unwrap()
            .scroll_up_by(self.currently_displaying_no as usize);
        Some(Action::Render)
    }

    fn scroll_to_end(&mut self) -> Option<Action> {
        self.table.lock().unwrap().scroll_to_end();
        Some(Action::Render)
    }

    fn scroll_to_home(&mut self) -> Option<Action> {
        self.table.lock().unwrap().scroll_to_home();
        Some(Action::Render)
    }
}

impl Component for SearchTab {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match action {
            A::Quit => Some(A::Quit),
            A::Search => self.start_search(),
            A::ChangeFocus => self.change_focus(),
            A::Input(input) => self.handle_input(input),
            A::Down => self.next_torrent(),
            A::Up => self.previous_torrent(),
            A::ScrollDownPage => self.scroll_down_page(),
            A::ScrollUpPage => self.scroll_up_page(),
            A::Home => self.scroll_to_home(),
            A::End => self.scroll_to_end(),
            A::Confirm => self.add_torrent(),
            A::Tick => self.tick(),

            _ => None,
        }
    }

    fn tick(&mut self) -> Option<Action> {
        self.search_result_info.lock().unwrap().tick();
        Some(Action::Render)
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [top_line, rest, bottom_line] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(1),
        ])
        .areas(rect);

        self.currently_displaying_no = rest.height;

        let search_rect = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .flex(Flex::Center)
        .split(top_line)[1];

        let input = {
            if self.input.value().is_empty() && self.search_focus != SearchFocus::Search {
                "press / to search"
            } else {
                self.input.value()
            }
        };

        let search_style = {
            if self.search_focus == SearchFocus::Search {
                Style::default()
                    .underlined()
                    .fg(self.ctx.config.general.accent_color)
            } else {
                Style::default().underlined().gray()
            }
        };

        let paragraph_text = format!(" {input}");
        let prefix_len = paragraph_text.len() - input.len() - 2;
        let paragraph = Paragraph::new(paragraph_text).style(search_style);

        f.render_widget(paragraph, search_rect);

        let cursor_offset = self.input.visual_cursor() + prefix_len;
        f.set_cursor(
            search_rect.x + u16::try_from(cursor_offset).unwrap(),
            search_rect.y,
        );

        let header = Row::new(["S", "Title", "Size"]);

        let table_lock = self.table.lock().unwrap();
        let table_items = &table_lock.items;

        let longest_title = table_items.iter().map(|magnet| magnet.title.len()).max();
        let items = table_items.iter().map(Self::magnet_to_row);

        let widths = [
            Constraint::Length(5),                                  // Seeders
            Constraint::Length(longest_title.unwrap_or(10) as u16), // Title
            Constraint::Length(8),                                  // Size
        ];

        let table_higlight_style = Style::default().on_black().bold().fg(self
            .ctx
            .config
            .general
            .accent_color);

        let table = {
            let table = Table::new(items, widths).highlight_style(table_higlight_style);
            if !self.ctx.config.general.headers_hide {
                table.header(header)
            } else {
                table
            }
        };

        f.render_stateful_widget(table, rest, &mut *table_lock.state.borrow_mut());

        self.search_result_info
            .lock()
            .unwrap()
            .render(f, bottom_line);
    }
}

#[derive(Clone)]
enum SearchResultState {
    Nothing,
    NoResults,
    Searching(Arc<Mutex<ThrobberState>>),
    Found(usize),
}

impl SearchResultState {
    fn new() -> Self {
        Self::Nothing
    }

    fn searching(&mut self, state: Arc<Mutex<ThrobberState>>) {
        *self = Self::Searching(state);
    }

    fn not_found(&mut self) {
        *self = Self::NoResults;
    }

    fn found(&mut self, count: usize) {
        *self = Self::Found(count);
    }
}

impl Component for SearchResultState {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match self {
            SearchResultState::Nothing => return,
            SearchResultState::Searching(state) => {
                let default_throbber = throbber_widgets_tui::Throbber::default()
                    .label("Searching...")
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                f.render_stateful_widget(
                    default_throbber.clone(),
                    rect,
                    &mut state.lock().unwrap(),
                );
            }
            SearchResultState::NoResults => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().red()));
                line.push_span(Span::raw(" No results"));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
            SearchResultState::Found(count) => {
                let mut line = Line::default();
                line.push_span(Span::styled("", Style::default().green()));
                line.push_span(Span::raw(format!(" Found {count}")));
                let paragraph = Paragraph::new(line);
                f.render_widget(paragraph, rect);
            }
        }
    }

    fn tick(&mut self) -> Option<Action> {
        match self {
            SearchResultState::Searching(state) => {
                state.lock().unwrap().calc_next();
                Some(Action::Render)
            }
            _ => None,
        }
    }
}
