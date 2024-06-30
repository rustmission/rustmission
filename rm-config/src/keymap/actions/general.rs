use rm_shared::action::Action;
use serde::{Deserialize, Serialize};

use super::UserAction;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeneralAction {
    ShowHelp,
    Quit,
    Close,
    SwitchToTorrents,
    SwitchToSearch,
    Left,
    Right,
    Down,
    Up,
    Search,
    SwitchFocus,
    Confirm,
    Select,
    ScrollPageDown,
    ScrollPageUp,
    GoToBeginning,
    GoToEnd,
}

impl UserAction for GeneralAction {
    fn desc(&self) -> &'static str {
        match self {
            GeneralAction::ShowHelp => "toggle help",
            GeneralAction::Quit => "quit Rustmission / a popup",
            GeneralAction::Close => "close a popup / task",
            GeneralAction::SwitchToTorrents => "switch to torrents tab",
            GeneralAction::SwitchToSearch => "switch to search tab",
            GeneralAction::Left => "switch to tab left",
            GeneralAction::Right => "switch to tab right",
            GeneralAction::Down => "move down",
            GeneralAction::Up => "move up",
            GeneralAction::Search => "search",
            GeneralAction::SwitchFocus => "switch focus",
            GeneralAction::Confirm => "confirm",
            GeneralAction::Select => "select",
            GeneralAction::ScrollPageDown => "scroll page down",
            GeneralAction::ScrollPageUp => "scroll page up",
            GeneralAction::GoToBeginning => "scroll to the beginning",
            GeneralAction::GoToEnd => "scroll to the end",
        }
    }
}

impl From<GeneralAction> for Action {
    fn from(value: GeneralAction) -> Self {
        match value {
            GeneralAction::ShowHelp => Action::ShowHelp,
            GeneralAction::Quit => Action::Quit,
            GeneralAction::Close => Action::Close,
            GeneralAction::SwitchToTorrents => Action::ChangeTab(1),
            GeneralAction::SwitchToSearch => Action::ChangeTab(2),
            GeneralAction::Left => Action::Left,
            GeneralAction::Right => Action::Right,
            GeneralAction::Down => Action::Down,
            GeneralAction::Up => Action::Up,
            GeneralAction::Search => Action::Search,
            GeneralAction::SwitchFocus => Action::ChangeFocus,
            GeneralAction::Confirm => Action::Confirm,
            GeneralAction::Select => Action::Select,
            GeneralAction::ScrollPageDown => Action::ScrollDownPage,
            GeneralAction::ScrollPageUp => Action::ScrollUpPage,
            GeneralAction::GoToBeginning => Action::Home,
            GeneralAction::GoToEnd => Action::End,
        }
    }
}
