use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Clear, Paragraph,
    },
};
use transmission_rpc::types::{Id, Torrent, TorrentSetArgs};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{
    app,
    transmission::TorrentAction,
    ui::{centered_rect, components::Component},
};
use rm_shared::action::Action;

pub struct FilesPopup {
    ctx: app::Ctx,
    torrent: Arc<Mutex<Option<Torrent>>>,
    torrent_id: Id,
    tree_state: TreeState<String>,
    tree: Arc<Mutex<Node>>,
    current_focus: CurrentFocus,
    switched_after_fetched_data: bool,
}

async fn fetch_new_files(
    tree: Arc<Mutex<Node>>,
    torrent: Arc<Mutex<Option<Torrent>>>,
    torrent_id: Id,
    ctx: app::Ctx,
) {
    loop {
        let new_torrent = ctx
            .client
            .lock()
            .await
            .torrent_get(None, Some(vec![torrent_id.clone()]))
            .await
            .unwrap()
            .arguments
            .torrents
            .pop()
            .unwrap();
        let new_tree = Node::new_from_torrent(&new_torrent);
        *torrent.lock().unwrap() = Some(new_torrent);
        *tree.lock().unwrap() = new_tree;
        ctx.send_action(Action::Render);
        tokio::time::sleep(Duration::from_secs(6)).await;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CurrentFocus {
    CloseButton,
    Files,
}

impl FilesPopup {
    pub fn new(ctx: app::Ctx, torrent_id: Id) -> Self {
        let torrent = Arc::new(Mutex::new(None));
        let tree_state = TreeState::default();
        let tree = Arc::new(Mutex::new(Node::new()));

        ctx.send_torrent_action(TorrentAction::GetTorrentInfo(
            torrent_id.clone(),
            Arc::clone(&torrent),
        ));

        tokio::task::spawn(fetch_new_files(
            Arc::clone(&tree),
            Arc::clone(&torrent),
            torrent_id.clone(),
            ctx.clone(),
        ));

        Self {
            ctx,
            torrent,
            tree_state,
            tree,
            current_focus: CurrentFocus::CloseButton,
            switched_after_fetched_data: false,
            torrent_id,
        }
    }

    fn switch_focus(&mut self) {
        match self.current_focus {
            CurrentFocus::CloseButton => self.current_focus = CurrentFocus::Files,
            CurrentFocus::Files => self.current_focus = CurrentFocus::CloseButton,
        }
    }
}

impl Component for FilesPopup {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match (action, self.current_focus) {
            (action, _) if action.is_soft_quit() => Some(A::Quit),
            (A::ChangeFocus, _) => {
                self.switch_focus();
                Some(A::Render)
            }
            (A::Confirm, CurrentFocus::CloseButton) => Some(A::Quit),
            (A::Select | A::Confirm, CurrentFocus::Files) => {
                if let Some(torrent) = &mut *self.torrent.lock().unwrap() {
                    let wanted_ids = torrent.wanted.as_mut().unwrap();

                    let selected_ids: Vec<_> = self
                        .tree_state
                        .selected()
                        .iter()
                        .filter_map(|str_id| str_id.parse::<i32>().ok())
                        .collect();

                    if selected_ids.is_empty() {
                        self.tree_state.toggle_selected();
                        return Some(A::Render);
                    }

                    let mut wanted_in_selection_no = 0;
                    for selected_id in &selected_ids {
                        if wanted_ids[*selected_id as usize] == 1 {
                            wanted_in_selection_no += 1;
                        } else {
                            wanted_in_selection_no -= 1;
                        }
                    }

                    if wanted_in_selection_no > 0 {
                        for selected_id in &selected_ids {
                            wanted_ids[*selected_id as usize] = 0;
                        }
                    } else {
                        for selected_id in &selected_ids {
                            wanted_ids[*selected_id as usize] = 1;
                        }
                    }

                    let args = {
                        if wanted_in_selection_no > 0 {
                            for transmission_file in
                                self.tree.lock().unwrap().get_by_ids(&selected_ids)
                            {
                                transmission_file.set_wanted(false);
                            }
                            TorrentSetArgs {
                                files_unwanted: Some(selected_ids),
                                ..Default::default()
                            }
                        } else {
                            for transmission_file in
                                self.tree.lock().unwrap().get_by_ids(&selected_ids)
                            {
                                transmission_file.set_wanted(true);
                            }
                            TorrentSetArgs {
                                files_wanted: Some(selected_ids),
                                ..Default::default()
                            }
                        }
                    };

                    self.ctx.send_torrent_action(TorrentAction::SetArgs(
                        Box::new(args),
                        Some(vec![self.torrent_id.clone()]),
                    ));

                    Some(A::Render)
                } else {
                    None
                }
            }

            (A::Up, CurrentFocus::Files) => {
                self.tree_state.key_up();
                Some(Action::Render)
            }
            (A::Down, CurrentFocus::Files) => {
                self.tree_state.key_down();
                Some(Action::Render)
            }

            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let popup_rect = centered_rect(rect, 75, 75);
        let block_rect = popup_rect.inner(&Margin::new(1, 0));

        let info_text_rect = block_rect.inner(&Margin::new(3, 2));

        let highlight_style = Style::default().fg(self.ctx.config.general.accent_color);
        let bold_highlight_style = highlight_style.on_black().bold();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Title::from(" Files ".set_style(highlight_style)).alignment(Alignment::Left));

        if self.tree_state.selected().is_empty() {
            self.tree_state.select_first();
        }

        if let Some(torrent) = &*self.torrent.lock().unwrap() {
            if !self.switched_after_fetched_data {
                self.current_focus = CurrentFocus::Files;
                self.switched_after_fetched_data = true;
            }

            let close_button_style = {
                match self.current_focus {
                    CurrentFocus::CloseButton => highlight_style.bold(),
                    CurrentFocus::Files => Style::default(),
                }
            };

            let tree_highlight_style = {
                if self.current_focus == CurrentFocus::Files {
                    bold_highlight_style
                } else {
                    Style::default()
                }
            };

            let download_dir = torrent.download_dir.as_ref().expect("Requested");
            let keybinding_tip = {
                if self.ctx.config.general.beginner_mode {
                    "[SPACE] - select"
                } else {
                    ""
                }
            };
            let block = block
                .title(
                    Title::from(format!(" {} ", download_dir).set_style(highlight_style))
                        .alignment(Alignment::Right),
                )
                .title(
                    Title::from(" [ CLOSE ] ".set_style(close_button_style))
                        .alignment(Alignment::Right)
                        .position(Position::Bottom),
                )
                .title(
                    Title::from(keybinding_tip)
                        .alignment(Alignment::Left)
                        .position(Position::Bottom),
                );

            let tree_lock = self.tree.lock().unwrap();
            let tree_items = tree_lock.make_tree();

            let tree_widget = Tree::new(&tree_items)
                .unwrap()
                .block(block)
                .highlight_style(tree_highlight_style);

            f.render_widget(Clear, popup_rect);
            f.render_stateful_widget(tree_widget, block_rect, &mut self.tree_state);
        } else {
            let paragraph = Paragraph::new("Loading...");
            let block = block.title(
                Title::from(" [ CLOSE ] ".set_style(highlight_style.bold()))
                    .alignment(Alignment::Right)
                    .position(Position::Bottom),
            );
            f.render_widget(Clear, popup_rect);
            f.render_widget(paragraph, info_text_rect);
            f.render_widget(block, block_rect);
        }
    }
}

struct TransmissionFile {
    name: String,
    id: usize,
    // TODO: Change to enum
    wanted: bool,
}

impl TransmissionFile {
    fn set_wanted(&mut self, new_wanted: bool) {
        self.wanted = new_wanted;
    }
}

struct Node {
    items: Vec<TransmissionFile>,
    directories: BTreeMap<String, Node>,
}

impl Node {
    fn new() -> Self {
        Self {
            items: vec![],
            directories: BTreeMap::new(),
        }
    }

    fn new_from_torrent(torrent: &Torrent) -> Self {
        let files = torrent.files.as_ref().unwrap();
        let mut root = Self::new();

        for (id, file) in files.iter().enumerate() {
            let path: Vec<String> = file.name.split('/').map(str::to_string).collect();

            let wanted = torrent.wanted.as_ref().unwrap()[id] != 0;

            let file = TransmissionFile {
                id,
                name: path[path.len() - 1].clone(),
                wanted,
            };

            root.add_transmission_file(file, &path);
        }

        root
    }

    fn add_transmission_file(&mut self, file: TransmissionFile, remaining_path: &[String]) {
        if let Some((first, rest)) = remaining_path.split_first() {
            if rest.is_empty() {
                // We've found home for our TransmissionFile! :D
                self.items.push(file);
            } else {
                let child = self
                    .directories
                    .entry(first.to_string())
                    .or_insert_with(Self::new);
                child.add_transmission_file(file, rest);
            }
        }
    }

    fn get_by_ids(&mut self, ids: &[i32]) -> Vec<&mut TransmissionFile> {
        let mut transmission_files = vec![];
        for file in &mut self.items {
            if ids.contains(&(file.id as i32)) {
                transmission_files.push(file);
            }
        }
        for node in self.directories.values_mut() {
            transmission_files.extend(node.get_by_ids(ids))
        }
        transmission_files
    }

    fn make_tree(&self) -> Vec<TreeItem<String>> {
        let mut tree_items = vec![];
        for transmission_file in &self.items {
            let name = {
                if transmission_file.wanted {
                    format!("󰄲 {}", transmission_file.name)
                } else {
                    format!(" {}", transmission_file.name)
                }
            };
            tree_items.push(TreeItem::new_leaf(transmission_file.id.to_string(), name));
        }

        for (key, value) in &self.directories {
            tree_items.push(TreeItem::new(key.clone(), key.clone(), value.make_tree()).unwrap());
        }
        tree_items
    }
}
