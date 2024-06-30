use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

use crate::{
    app,
    transmission::TorrentAction,
    ui::{components::Component, tabs::torrents::input_manager::InputManager, to_input_request},
};
use rm_shared::{action::Action, status_task::StatusTask};

pub struct AddMagnetBar {
    input_magnet_mgr: InputManager,
    input_location_mgr: InputManager,
    stage: Stage,
    ctx: app::Ctx,
}

enum Stage {
    AskMagnet,
    AskLocation,
}

impl AddMagnetBar {
    pub fn new(ctx: app::Ctx) -> Self {
        Self {
            input_magnet_mgr: InputManager::new(
                ctx.clone(),
                "Add (Magnet URL / Torrent path): ".to_string(),
            ),
            input_location_mgr: InputManager::new_with_value(
                ctx.clone(),
                "Directory: ".to_string(),
                ctx.session_info.download_dir.clone(),
            ),
            stage: Stage::AskMagnet,
            ctx,
        }
    }

    fn handle_input(&mut self, input: KeyEvent) -> Option<Action> {
        match self.stage {
            Stage::AskMagnet => self.handle_magnet_input(input),
            Stage::AskLocation => self.handle_location_input(input),
        }
    }

    fn handle_magnet_input(&mut self, input: KeyEvent) -> Option<Action> {
        if input.code == KeyCode::Enter {
            self.stage = Stage::AskLocation;
            return Some(Action::Render);
        }
        if input.code == KeyCode::Esc {
            return Some(Action::Quit);
        }

        if let Some(req) = to_input_request(input) {
            self.input_magnet_mgr.handle(req);
            return Some(Action::Render);
        }
        None
    }

    fn handle_location_input(&mut self, input: KeyEvent) -> Option<Action> {
        if input.code == KeyCode::Enter {
            self.ctx.send_torrent_action(TorrentAction::Add(
                self.input_magnet_mgr.text(),
                Some(self.input_location_mgr.text()),
            ));
            return Some(Action::TaskPending(StatusTask::Add(
                self.input_magnet_mgr.text(),
            )));
        }
        if input.code == KeyCode::Esc {
            return Some(Action::Quit);
        }

        if let Some(req) = to_input_request(input) {
            self.input_location_mgr.handle(req);
            return Some(Action::Render);
        }
        None
    }
}

impl Component for AddMagnetBar {
    #[must_use]
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Input(input) => self.handle_input(input),
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        match self.stage {
            Stage::AskMagnet => self.input_magnet_mgr.render(f, rect),
            Stage::AskLocation => self.input_location_mgr.render(f, rect),
        }
    }
}
