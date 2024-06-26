use std::{collections::HashMap, marker::PhantomData, path::PathBuf, sync::OnceLock};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use toml::Table;

use crate::utils;
use rm_shared::action::Action;

#[derive(Serialize, Deserialize, Clone)]
pub struct KeymapConfig {
    pub general: General<GeneralAction>,
    pub torrents_tab: TorrentsTab<TorrentsAction>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct General<T: Into<Action>> {
    pub keybindings: Vec<Keybinding<T>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeneralAction {
    ShowHelp,
    Quit,
    SoftQuit,
    SwitchToTorrents,
    SwitchToSearch,
    Left,
    Right,
    Down,
    Up,
    Search,
    SwitchFocus,
    Confirm,
    ScrollPageDown,
    ScrollPageUp,
    GoToBeginning,
    GoToEnd,
}

pub trait UserAction: Into<Action> {
    fn desc(&self) -> &'static str;
}

impl UserAction for GeneralAction {
    fn desc(&self) -> &'static str {
        match self {
            GeneralAction::ShowHelp => "toggle help",
            GeneralAction::Quit => "quit Rustmission / a popup",
            GeneralAction::SoftQuit => "close a popup / task",
            GeneralAction::SwitchToTorrents => "switch to torrents tab",
            GeneralAction::SwitchToSearch => "switch to search tab",
            GeneralAction::Left => "switch to tab left",
            GeneralAction::Right => "switch to tab right",
            GeneralAction::Down => "move down",
            GeneralAction::Up => "move up",
            GeneralAction::Search => "search",
            GeneralAction::SwitchFocus => "switch focus",
            GeneralAction::Confirm => "confirm",
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
            GeneralAction::SoftQuit => Action::SoftQuit,
            GeneralAction::SwitchToTorrents => Action::ChangeTab(1),
            GeneralAction::SwitchToSearch => Action::ChangeTab(2),
            GeneralAction::Left => Action::Left,
            GeneralAction::Right => Action::Right,
            GeneralAction::Down => Action::Down,
            GeneralAction::Up => Action::Up,
            GeneralAction::Search => Action::Search,
            GeneralAction::SwitchFocus => Action::ChangeFocus,
            GeneralAction::Confirm => Action::Confirm,
            GeneralAction::ScrollPageDown => Action::ScrollDownPage,
            GeneralAction::ScrollPageUp => Action::ScrollUpPage,
            GeneralAction::GoToBeginning => Action::Home,
            GeneralAction::GoToEnd => Action::End,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TorrentsTab<T: Into<Action>> {
    pub keybindings: Vec<Keybinding<T>>,
}

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

#[derive(Serialize, Clone)]
pub struct Keybinding<T: Into<Action>> {
    pub on: KeyCode,
    #[serde(default)]
    pub modifier: KeyModifier,
    pub action: T,
}

impl<T: Into<Action>> Keybinding<T> {
    pub fn keycode_string(&self) -> String {
        let key = match self.on {
            KeyCode::Backspace => "Backspace".into(),
            KeyCode::Enter => "Enter".into(),
            KeyCode::Left => "".into(),
            KeyCode::Right => "".into(),
            KeyCode::Up => "".into(),
            KeyCode::Down => "".into(),
            KeyCode::Home => "Home".into(),
            KeyCode::End => "End".into(),
            KeyCode::PageUp => "PageUp".into(),
            KeyCode::PageDown => "PageDown".into(),
            KeyCode::Tab => "Tab".into(),
            KeyCode::BackTab => todo!(),
            KeyCode::Delete => todo!(),
            KeyCode::Insert => "Insert".into(),
            KeyCode::F(i) => format!("F{i}"),
            KeyCode::Char(c) => c.into(),
            KeyCode::Null => todo!(),
            KeyCode::Esc => "Esc".into(),
            KeyCode::CapsLock => todo!(),
            KeyCode::ScrollLock => todo!(),
            KeyCode::NumLock => todo!(),
            KeyCode::PrintScreen => todo!(),
            KeyCode::Pause => todo!(),
            KeyCode::Menu => todo!(),
            KeyCode::KeypadBegin => todo!(),
            KeyCode::Media(_) => todo!(),
            KeyCode::Modifier(_) => todo!(),
        };

        if !self.modifier.is_none() {
            format!("{}-{key}", self.modifier.to_str())
        } else {
            key
        }
    }
}

impl<T: Into<Action>> Keybinding<T> {
    fn new(on: KeyCode, action: T, modifier: Option<KeyModifier>) -> Self {
        Self {
            on,
            modifier: modifier.unwrap_or(KeyModifier::None),
            action,
        }
    }
}

impl<'de, T: Into<Action> + Deserialize<'de>> Deserialize<'de> for Keybinding<T> {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            On,
            Modifier,
            Action,
        }

        struct KeybindingVisitor<T> {
            phantom: PhantomData<T>,
        }

        impl<'de, T: Into<Action> + Deserialize<'de>> Visitor<'de> for KeybindingVisitor<T> {
            type Value = Keybinding<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Keybinding")
            }

            fn visit_map<A>(self, mut map: A) -> std::prelude::v1::Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut on = None;
                let mut modifier = None;
                let mut action = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::On => {
                            if on.is_some() {
                                return Err(de::Error::duplicate_field("on"));
                            }
                            let key = map.next_value::<String>()?;

                            if key.len() == 1 {
                                on = Some(KeyCode::Char(key.chars().next().unwrap()));
                            } else if key.starts_with('F') && (key.len() == 2 || key.len() == 3) {
                                let which_f = key[1..].parse::<u8>().map_err(|_| {
                                    de::Error::invalid_value(
                                        de::Unexpected::Str(&key),
                                        &"something_correct",
                                    )
                                })?;
                                on = Some(KeyCode::F(which_f));
                            } else {
                                on = {
                                    match key.to_lowercase().as_str() {
                                        "enter" => Some(KeyCode::Enter),
                                        "esc" => Some(KeyCode::Esc),
                                        "up" => Some(KeyCode::Up),
                                        "down" => Some(KeyCode::Down),
                                        "left" => Some(KeyCode::Left),
                                        "right" => Some(KeyCode::Right),
                                        "home" => Some(KeyCode::Home),
                                        "end" => Some(KeyCode::End),
                                        "pageup" => Some(KeyCode::PageUp),
                                        "pagedown" => Some(KeyCode::PageDown),
                                        "tab" => Some(KeyCode::Tab),
                                        "backspace" => Some(KeyCode::Backspace),
                                        "delete" => Some(KeyCode::Delete),

                                        _ => {
                                            return Err(de::Error::invalid_value(
                                                de::Unexpected::Str(&key),
                                                &"something correct",
                                            ))
                                        }
                                    }
                                };
                            }
                        }
                        Field::Modifier => {
                            if modifier.is_some() {
                                return Err(de::Error::duplicate_field("modifier"));
                            }
                            modifier = Some(map.next_value());
                        }
                        Field::Action => {
                            if action.is_some() {
                                return Err(de::Error::duplicate_field("action"));
                            }
                            action = Some(map.next_value());
                        }
                    }
                }
                let on = on.ok_or_else(|| de::Error::missing_field("on"))?;
                let action = action.ok_or_else(|| de::Error::missing_field("action"))??;
                Ok(Keybinding::new(on, action, modifier.transpose().unwrap()))
            }
        }

        const FIELDS: &[&str] = &["on", "modifier", "action"];
        deserializer.deserialize_struct(
            "Keybinding",
            FIELDS,
            KeybindingVisitor {
                phantom: PhantomData::default(),
            },
        )
    }
}

#[derive(Serialize, Deserialize, Hash, Clone, Copy, PartialEq, Eq)]
pub enum KeyModifier {
    None,
    Ctrl,
    Shift,
}

impl KeyModifier {
    fn to_str(self) -> &'static str {
        match self {
            KeyModifier::None => "",
            KeyModifier::Ctrl => "CTRL",
            KeyModifier::Shift => "SHIFT",
        }
    }

    fn is_none(self) -> bool {
        self == KeyModifier::None
    }
}

impl From<KeyModifier> for KeyModifiers {
    fn from(value: KeyModifier) -> Self {
        match value {
            KeyModifier::None => KeyModifiers::NONE,
            KeyModifier::Ctrl => KeyModifiers::CONTROL,
            KeyModifier::Shift => KeyModifiers::SHIFT,
        }
    }
}

impl Default for KeyModifier {
    fn default() -> Self {
        Self::None
    }
}

impl KeymapConfig {
    pub const FILENAME: &'static str = "keymap.toml";

    pub fn init() -> Result<Self> {
        let table = {
            // TODO: handle errors or there will be hell to pay
            if let Ok(table) = utils::fetch_config(Self::FILENAME) {
                table
            } else {
                todo!();
            }
        };

        Self::table_to_keymap(&table)
    }

    pub fn to_map(self) -> HashMap<(KeyCode, KeyModifiers), Action> {
        let mut map = HashMap::new();
        for keybinding in self.general.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            map.insert(hash_value, keybinding.action.into());
        }
        for keybinding in self.torrents_tab.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            map.insert(hash_value, keybinding.action.into());
        }
        map
    }

    fn table_to_keymap(table: &Table) -> Result<Self> {
        let config_string = table.to_string();
        let config = toml::from_str(&config_string)?;
        Ok(config)
    }

    pub fn path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| utils::get_config_path(Self::FILENAME))
    }
}