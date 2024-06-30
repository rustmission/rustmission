use crossterm::event::KeyCode;
use ratatui::prelude::*;
use transmission_rpc::types::Id;

use crate::{
    app,
    transmission::TorrentAction,
    ui::{components::Component, tabs::torrents::input_manager::InputManager, to_input_request},
};
use rm_shared::action::Action;

pub struct DeleteBar {
    torrents_to_delete: Vec<Id>,
    ctx: app::Ctx,
    input_mgr: InputManager,
    mode: Mode,
}

pub enum Mode {
    WithFiles,
    WithoutFiles,
}

impl DeleteBar {
    pub fn new(ctx: app::Ctx, to_delete: Vec<Id>, mode: Mode) -> Self {
        let prompt = {
            match mode {
                Mode::WithFiles => "Really delete selected WITH files? (y/n) ".to_string(),
                Mode::WithoutFiles => "Really delete selected without files? (y/n) ".to_string(),
            }
        };

        Self {
            torrents_to_delete: to_delete,
            input_mgr: InputManager::new(ctx.clone(), prompt),
            ctx,
            mode,
        }
    }
}

impl Component for DeleteBar {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Input(input) => {
                if input.code == KeyCode::Esc {
                    return Some(Action::Quit);
                }

                if input.code == KeyCode::Enter {
                    let text = self.input_mgr.text().to_lowercase();
                    if text == "y" || text == "yes" {
                        let torrents_to_delete = self.torrents_to_delete.clone();
                        match self.mode {
                            Mode::WithFiles => self.ctx.send_torrent_action(
                                TorrentAction::DeleteWithFiles(torrents_to_delete),
                            ),
                            Mode::WithoutFiles => {
                                self.ctx
                                    .send_torrent_action(TorrentAction::DeleteWithoutFiles(
                                        torrents_to_delete,
                                    ))
                            }
                        }
                        return Some(Action::Quit);
                    } else if text == "n" || text == "no" {
                        return Some(Action::Quit);
                    }
                }

                if let Some(req) = to_input_request(input) {
                    self.input_mgr.handle(req);
                    return Some(Action::Render);
                }

                None
            }
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input_mgr.render(f, rect)
    }
}
