use rm_shared::action::Action;
use serde::{Deserialize, Serialize};

use super::UserAction;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TorrentsAction {
    AddMagnet,
    Pause,
    DeleteWithFiles,
    DeleteWithoutFiles,
    ShowFiles,
    ShowStats,
}

impl UserAction for TorrentsAction {
    fn desc(&self) -> &'static str {
        match self {
            TorrentsAction::AddMagnet => "add a magnet",
            TorrentsAction::Pause => "pause/unpause",
            TorrentsAction::DeleteWithFiles => "delete with files",
            TorrentsAction::DeleteWithoutFiles => "delete without files",
            TorrentsAction::ShowFiles => "show files",
            TorrentsAction::ShowStats => "show statistics",
        }
    }
}

impl From<TorrentsAction> for Action {
    fn from(value: TorrentsAction) -> Self {
        match value {
            TorrentsAction::AddMagnet => Action::AddMagnet,
            TorrentsAction::Pause => Action::Pause,
            TorrentsAction::DeleteWithFiles => Action::DeleteWithFiles,
            TorrentsAction::DeleteWithoutFiles => Action::DeleteWithoutFiles,
            TorrentsAction::ShowFiles => Action::ShowFiles,
            TorrentsAction::ShowStats => Action::ShowStats,
        }
    }
}
