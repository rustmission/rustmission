use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{prelude::*, widgets::Row};
use std::sync::{Arc, Mutex};

use crate::{app, ui::components::table::GenericTable};

use super::rustmission_torrent::RustmissionTorrent;

pub struct TableManager {
    ctx: app::Ctx,
    pub table: GenericTable<RustmissionTorrent>,
    pub widths: [Constraint; 8],
    pub filter: Arc<Mutex<Option<String>>>,
    pub torrents_displaying_no: u16,
    header: Vec<String>,
}

impl TableManager {
    pub fn new(ctx: app::Ctx, table: GenericTable<RustmissionTorrent>) -> Self {
        let widths = Self::default_widths();
        Self {
            ctx,
            table,
            widths,
            filter: Arc::new(Mutex::new(None)),
            torrents_displaying_no: 0,
            header: vec![
                "Name".to_owned(),
                "".to_owned(),
                "Size".to_owned(),
                "Progress".to_owned(),
                "ETA".to_owned(),
                "Download".to_owned(),
                "Upload".to_owned(),
                "Directory".to_owned(),
            ],
        }
    }

    pub fn rows(&self) -> Vec<Row<'_>> {
        if let Some(filter) = &*self.filter.lock().unwrap() {
            let rows = self.filtered_torrents_rows(&self.table.items, filter);
            self.table.overwrite_len(rows.len());
            rows
        } else {
            self.table
                .items
                .iter()
                .map(RustmissionTorrent::to_row)
                .collect()
        }
    }

    pub const fn header(&self) -> &Vec<String> {
        &self.header
    }

    pub fn current_torrent(&mut self) -> Option<&mut RustmissionTorrent> {
        let matcher = SkimMatcherV2::default();
        let index = self.table.state.borrow().selected()?;

        if let Some(filter) = &*self.filter.lock().unwrap() {
            let mut loop_index = 0;
            for rustmission_torrent in &mut self.table.items {
                if matcher
                    .fuzzy_match(&rustmission_torrent.torrent_name, filter)
                    .is_some()
                {
                    if index == loop_index {
                        return Some(rustmission_torrent);
                    }
                    loop_index += 1;
                }
            }
            return None;
        }
        self.table.items.get_mut(index)
    }

    pub fn set_new_rows(&mut self, rows: Vec<RustmissionTorrent>) {
        self.table.items = rows;
        self.widths = self.header_widths(&self.table.items);
    }

    fn filtered_torrents_rows<'a>(
        &self,
        torrents: &'a [RustmissionTorrent],
        filter: &str,
    ) -> Vec<Row<'a>> {
        let matcher = SkimMatcherV2::default();
        let mut rows = vec![];

        let highlight_style = Style::default().fg(self.ctx.config.general.accent_color);

        for torrent in torrents {
            if let Some((_, indices)) = matcher.fuzzy_indices(&torrent.torrent_name, filter) {
                rows.push(torrent.to_row_with_higlighted_indices(indices, highlight_style))
            }
        }

        rows
    }

    const fn default_widths() -> [Constraint; 8] {
        [
            Constraint::Max(70),    // Name
            Constraint::Length(5),  // <padding>
            Constraint::Length(12), // Size
            Constraint::Length(12), // Progress
            Constraint::Length(12), // ETA
            Constraint::Length(12), // Download
            Constraint::Length(12), // Upload
            Constraint::Max(70),    // Download directory
        ]
    }

    fn header_widths(&self, rows: &[RustmissionTorrent]) -> [Constraint; 8] {
        if !self.ctx.config.general.auto_hide {
            return Self::default_widths();
        }

        let mut download_width = 0;
        let mut upload_width = 0;
        let mut progress_width = 0;
        let mut eta_width = 0;

        for row in rows {
            if !row.download_speed.is_empty() {
                download_width = 11;
            }
            if !row.upload_speed.is_empty() {
                upload_width = 11;
            }
            if !row.progress.is_empty() {
                progress_width = 11;
            }

            if !row.eta_secs.is_empty() {
                eta_width = 11;
            }
        }

        [
            Constraint::Max(70),                // Name
            Constraint::Length(5),              // <padding>
            Constraint::Length(11),             // Size
            Constraint::Length(progress_width), // Progress
            Constraint::Length(eta_width),      // ETA
            Constraint::Length(download_width), // Download
            Constraint::Length(upload_width),   // Upload
            Constraint::Max(70),                // Download directory
        ]
    }
}
