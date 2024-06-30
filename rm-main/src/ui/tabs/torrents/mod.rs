mod bottom_stats;
mod input_manager;
pub mod popups;
pub mod rustmission_torrent;
pub mod table_manager;
pub mod task_manager;
pub mod tasks;

use std::sync::{Arc, Mutex};

use crate::transmission::TorrentAction;
use crate::ui::tabs::torrents::popups::stats::StatisticsPopup;

use ratatui::prelude::*;
use ratatui::widgets::{Row, Table};
use transmission_rpc::types::TorrentStatus;

use crate::ui::components::table::GenericTable;
use crate::ui::components::Component;
use crate::{app, transmission};
use rm_shared::action::Action;

use self::bottom_stats::BottomStats;
use self::popups::files::FilesPopup;
use self::popups::{CurrentPopup, PopupManager};
use self::table_manager::TableManager;
use self::task_manager::TaskManager;

pub struct TorrentsTab {
    ctx: app::Ctx,
    table_manager: Arc<Mutex<TableManager>>,
    popup_manager: PopupManager,
    task_manager: TaskManager,
    bottom_stats: BottomStats,
}

impl TorrentsTab {
    pub fn new(ctx: app::Ctx) -> Self {
        let table = GenericTable::new(vec![]);
        let table_manager = Arc::new(Mutex::new(TableManager::new(ctx.clone(), table)));
        let stats = Arc::new(Mutex::new(None));
        let free_space = Arc::new(Mutex::new(None));
        let bottom_stats = BottomStats::new(stats, free_space, Arc::clone(&table_manager));

        tokio::spawn(transmission::fetchers::stats(
            ctx.clone(),
            Arc::clone(&bottom_stats.stats),
        ));

        tokio::spawn(transmission::fetchers::torrents(
            ctx.clone(),
            Arc::clone(&bottom_stats.table_manager),
        ));

        tokio::spawn(transmission::fetchers::free_space(
            ctx.clone(),
            Arc::clone(&bottom_stats.free_space),
        ));

        Self {
            bottom_stats,
            task_manager: TaskManager::new(table_manager.clone(), ctx.clone()),
            table_manager,
            popup_manager: PopupManager::new(),
            ctx,
        }
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [torrents_list_rect, stats_rect] =
            Layout::vertical([Constraint::Min(10), Constraint::Length(1)]).areas(rect);

        self.render_table(f, torrents_list_rect);

        self.bottom_stats.render(f, stats_rect);

        self.task_manager.render(f, stats_rect);

        self.popup_manager.render(f, f.size());
    }

    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        if self.popup_manager.is_showing_popup() {
            return self.popup_manager.handle_actions(action);
        }

        if action.is_quit() {
            return Some(Action::Quit);
        }

        match action {
            A::Up => self.previous_torrent(),
            A::Down => self.next_torrent(),
            A::ScrollUpPage => self.scroll_page_up(),
            A::ScrollDownPage => self.scroll_page_down(),
            A::Home => self.scroll_to_home(),
            A::End => self.scroll_to_end(),
            A::ShowStats => self.show_statistics_popup(),
            A::ShowFiles => self.show_files_popup(),
            A::Pause => self.pause_current_torrent(),
            other => self.task_manager.handle_actions(other),
        }
    }
}

impl TorrentsTab {
    fn render_table(&mut self, f: &mut Frame, rect: Rect) {
        let table_manager_lock = &mut *self.table_manager.lock().unwrap();
        table_manager_lock.torrents_displaying_no = rect.height;

        let torrent_rows = table_manager_lock.rows();

        let highlight_table_style = Style::default().on_black().bold().fg(self
            .ctx
            .config
            .general
            .accent_color);

        let table_widget = {
            let table = Table::new(torrent_rows, table_manager_lock.widths)
                .highlight_style(highlight_table_style);
            if !self.ctx.config.general.headers_hide {
                table.header(Row::new(
                    table_manager_lock.header().iter().map(|s| s.as_str()),
                ))
            } else {
                table
            }
        };

        f.render_stateful_widget(
            table_widget,
            rect,
            &mut table_manager_lock.table.state.borrow_mut(),
        );
    }

    fn show_files_popup(&mut self) -> Option<Action> {
        if let Some(highlighted_torrent) = self.table_manager.lock().unwrap().current_torrent() {
            let popup = FilesPopup::new(self.ctx.clone(), highlighted_torrent.id.clone());
            self.popup_manager.show_popup(CurrentPopup::Files(popup));
            Some(Action::Render)
        } else {
            None
        }
    }

    fn show_statistics_popup(&mut self) -> Option<Action> {
        if let Some(stats) = &*self.bottom_stats.stats.lock().unwrap() {
            let popup = StatisticsPopup::new(self.ctx.clone(), stats.clone());
            self.popup_manager.show_popup(CurrentPopup::Stats(popup));
            Some(Action::Render)
        } else {
            None
        }
    }

    fn previous_torrent(&self) -> Option<Action> {
        self.table_manager.lock().unwrap().table.previous();
        Some(Action::Render)
    }

    fn next_torrent(&self) -> Option<Action> {
        self.table_manager.lock().unwrap().table.next();
        Some(Action::Render)
    }

    fn scroll_page_down(&self) -> Option<Action> {
        let table_manager = &mut self.table_manager.lock().unwrap();
        let scroll_by = table_manager.torrents_displaying_no;
        table_manager.table.scroll_down_by(scroll_by as usize);
        Some(Action::Render)
    }

    fn scroll_page_up(&self) -> Option<Action> {
        let table_manager = &mut self.table_manager.lock().unwrap();
        let scroll_by = table_manager.torrents_displaying_no;
        table_manager.table.scroll_up_by(scroll_by as usize);
        Some(Action::Render)
    }

    fn scroll_to_home(&self) -> Option<Action> {
        let table_manager = &mut self.table_manager.lock().unwrap();
        table_manager.table.scroll_to_home();
        Some(Action::Render)
    }

    fn scroll_to_end(&self) -> Option<Action> {
        let table_manager = &mut self.table_manager.lock().unwrap();
        table_manager.table.scroll_to_end();
        Some(Action::Render)
    }

    fn pause_current_torrent(&mut self) -> Option<Action> {
        if let Some(torrent) = self.table_manager.lock().unwrap().current_torrent() {
            let torrent_id = torrent.id.clone();
            match torrent.status() {
                TorrentStatus::Stopped => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Start(vec![torrent_id]));
                    torrent.update_status(TorrentStatus::Downloading);
                    return Some(Action::Render);
                }
                _ => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Stop(vec![torrent_id]));
                    torrent.update_status(TorrentStatus::Stopped);
                    return Some(Action::Render);
                }
            }
        }
        None
    }
}
