use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use rm_shared::action::Action;
use transmission_rpc::types::Id;

use crate::{
    app,
    transmission::TorrentAction,
    ui::{components::Component, tabs::torrents::input_manager::InputManager, to_input_request},
};

pub struct MoveBar {
    torrents_to_move: Vec<Id>,
    ctx: app::Ctx,
    input_mgr: InputManager,
}

impl MoveBar {
    pub fn new(ctx: app::Ctx, torrents_to_move: Vec<Id>, existing_location: String) -> Self {
        let prompt = format!("New directory: ");

        Self {
            torrents_to_move,
            input_mgr: InputManager::new_with_value(ctx.clone(), prompt, existing_location),
            ctx,
        }
    }

    fn handle_input(&mut self, input: KeyEvent) -> Option<Action> {
        if input.code == KeyCode::Enter {
            let new_location = self.input_mgr.text();
            let torrents_to_move = self.torrents_to_move.clone();
            self.ctx
                .send_torrent_action(TorrentAction::Move(torrents_to_move, new_location));
            return Some(Action::Quit);
        }

        if input.code == KeyCode::Esc {
            return Some(Action::Quit);
        }

        if let Some(req) = to_input_request(input) {
            self.input_mgr.handle(req);
            return Some(Action::Render);
        }

        None
    }
}

impl Component for MoveBar {
    fn handle_actions(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Input(input) => self.handle_input(input),
            _ => None,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.input_mgr.render(f, rect)
    }
}
