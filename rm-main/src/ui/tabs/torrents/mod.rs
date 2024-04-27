pub mod popups;
pub mod rustmission_torrent;
mod stats;
pub mod table_manager;
pub mod task_manager;
pub mod tasks;

use std::sync::{Arc, Mutex};

use crate::ui::tabs::torrents::popups::stats::StatisticsPopup;

use ratatui::prelude::*;
use ratatui::widgets::{Row, Table};
use ratatui_macros::constraints;
use transmission_rpc::types::TorrentStatus;

use crate::action::{Action, TorrentAction};
use crate::ui::components::table::GenericTable;
use crate::ui::components::Component;
use crate::{app, transmission};

use self::popups::PopupManager;
use self::rustmission_torrent::RustmissionTorrent;
use self::stats::StatsComponent;
use self::table_manager::TableManager;
use self::task_manager::TaskManager;

pub struct TorrentsTab {
    table_manager: Arc<Mutex<TableManager>>,
    stats: StatsComponent,
    task: TaskManager,
    popup_manager: PopupManager,
    ctx: app::Ctx,
}

impl TorrentsTab {
    pub fn new(ctx: app::Ctx) -> Self {
        let stats = StatsComponent::default();
        let table = Arc::new(Mutex::new(GenericTable::new(vec![])));
        let rows = vec![];

        let table_manager = Arc::new(Mutex::new(TableManager::new(
            ctx.clone(),
            Arc::clone(&table),
            rows,
        )));

        tokio::spawn(transmission::stats_fetch(
            ctx.clone(),
            Arc::clone(&stats.stats),
        ));

        tokio::spawn(transmission::torrent_fetch(
            ctx.clone(),
            Arc::clone(&table.lock().unwrap().items),
            Arc::clone(&table_manager),
        ));

        Self {
            stats,
            task: TaskManager::new(table_manager.clone(), ctx.clone()),
            table_manager,
            popup_manager: PopupManager::new(),
            ctx,
        }
    }
}

impl Component for TorrentsTab {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let [torrents_list_rect, stats_rect] =
            Layout::vertical(constraints![>=10, ==1]).areas(rect);

        let table_manager = &self.table_manager.lock().unwrap();

        let rows = &table_manager.rows;

        let torrent_rows: Vec<_> = rows
            .iter()
            .map(|torrent| {
                RustmissionTorrent::to_row(torrent, &table_manager.filter.lock().unwrap())
            })
            .filter_map(|row| row)
            .collect();

        table_manager
            .table
            .lock()
            .unwrap()
            .overwrite_len(torrent_rows.len());

        let highlight_table_style = Style::default().on_black().bold().fg(self
            .ctx
            .config
            .general
            .accent_color
            .as_ratatui());
        let table = Table::new(torrent_rows, table_manager.widths)
            .header(Row::new(table_manager.header().iter().map(|s| s.as_str())))
            .highlight_style(highlight_table_style);

        f.render_stateful_widget(
            table,
            torrents_list_rect,
            &mut table_manager.table.lock().unwrap().state.borrow_mut(),
        );

        self.stats.render(f, stats_rect);

        self.task.render(f, stats_rect);

        self.popup_manager.render(f, f.size());
    }

    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        if self.popup_manager.is_showing_popup() {
            return self.popup_manager.handle_actions(action);
        }

        match action {
            A::Up => self.previous_torrent(),
            A::Down => self.next_torrent(),
            A::ShowStats => self.show_statistics_popup(),
            A::Pause => self.pause_current_torrent(),
            other => self.task.handle_actions(other),
        }
    }
}

impl TorrentsTab {
    fn show_statistics_popup(&mut self) -> Option<Action> {
        if let Some(stats) = &*self.stats.stats.lock().unwrap() {
            let popup = StatisticsPopup::new(self.ctx.clone(), stats.clone());
            self.popup_manager.show_popup(popup);
            Some(Action::Render)
        } else {
            None
        }
    }

    fn previous_torrent(&self) -> Option<Action> {
        self.table_manager
            .lock()
            .unwrap()
            .table
            .lock()
            .unwrap()
            .previous();
        Some(Action::Render)
    }

    fn next_torrent(&self) -> Option<Action> {
        self.table_manager
            .lock()
            .unwrap()
            .table
            .lock()
            .unwrap()
            .next();
        Some(Action::Render)
    }

    fn pause_current_torrent(&mut self) -> Option<Action> {
        let table_manager = self.table_manager.lock().unwrap();
        if let Some(torrent) = table_manager.current_item() {
            let torrent_id = torrent.id.clone();
            let torrent_status = torrent.status;
            match torrent_status {
                TorrentStatus::Stopped => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Start(Box::new(vec![torrent_id])));
                }
                _ => {
                    self.ctx
                        .send_torrent_action(TorrentAction::Stop(Box::new(vec![torrent_id])));
                }
            }
        }
        None
    }
}