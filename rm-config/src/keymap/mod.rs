pub mod actions;

use std::{
    collections::HashMap, io::ErrorKind, marker::PhantomData, path::PathBuf, sync::OnceLock,
};

use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyModifiers as CrosstermKeyModifiers};
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};

use crate::utils;
use rm_shared::action::Action;

use self::actions::{general::GeneralAction, torrents_tab::TorrentsAction};

#[derive(Serialize, Deserialize, Clone)]
pub struct KeymapConfig {
    pub general: KeybindsHolder<GeneralAction>,
    pub torrents_tab: KeybindsHolder<TorrentsAction>,
    #[serde(skip)]
    pub keymap: HashMap<(KeyCode, CrosstermKeyModifiers), Action>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeybindsHolder<T: Into<Action>> {
    pub keybindings: Vec<Keybinding<T>>,
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
            KeyCode::Char(c) => {
                if c == ' ' {
                    "Space".into()
                } else {
                    c.into()
                }
            }
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
                let modifier = modifier.transpose().unwrap();

                if modifier.is_some() {
                    if let KeyCode::Char(char) = on {
                        if char.is_uppercase() {
                            return Err(de::Error::custom(
                                "you can't have a modifier with an uppercase letter, sorry",
                            ));
                        }
                    }
                }

                Ok(Keybinding::new(on, action, modifier))
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
    Alt,
    Super,
    Meta,
}

impl KeyModifier {
    fn to_str(self) -> &'static str {
        match self {
            KeyModifier::None => "",
            KeyModifier::Ctrl => "CTRL",
            KeyModifier::Shift => "SHIFT",
            KeyModifier::Alt => "ALT",
            KeyModifier::Super => "SUPER",
            KeyModifier::Meta => "META",
        }
    }

    fn is_none(self) -> bool {
        self == KeyModifier::None
    }
}

impl From<KeyModifier> for CrosstermKeyModifiers {
    fn from(value: KeyModifier) -> Self {
        match value {
            KeyModifier::None => CrosstermKeyModifiers::NONE,
            KeyModifier::Ctrl => CrosstermKeyModifiers::CONTROL,
            KeyModifier::Shift => CrosstermKeyModifiers::SHIFT,
            KeyModifier::Alt => CrosstermKeyModifiers::ALT,
            KeyModifier::Super => CrosstermKeyModifiers::SUPER,
            KeyModifier::Meta => CrosstermKeyModifiers::META,
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
    const DEFAULT_CONFIG: &'static str = include_str!("../../defaults/keymap.toml");

    pub fn init() -> Result<Self> {
        match utils::fetch_config::<Self>(Self::FILENAME) {
            Ok(mut keymap_config) => {
                keymap_config.populate_hashmap();
                return Ok(keymap_config);
            }
            Err(e) => match e {
                utils::ConfigFetchingError::Io(e) if e.kind() == ErrorKind::NotFound => {
                    let mut keymap_config =
                        utils::put_config::<Self>(Self::DEFAULT_CONFIG, Self::FILENAME)?;
                    keymap_config.populate_hashmap();
                    return Ok(keymap_config);
                }
                _ => anyhow::bail!(e),
            },
        }
    }

    pub fn get_keys_for_action(&self, action: Action) -> Option<String> {
        let mut keys = vec![];

        for keybinding in &self.general.keybindings {
            if action == keybinding.action.into() {
                keys.push(keybinding.keycode_string());
            }
        }
        for keybinding in &self.torrents_tab.keybindings {
            if action == keybinding.action.into() {
                keys.push(keybinding.keycode_string());
            }
        }

        if keys.is_empty() {
            return None;
        } else {
            Some(keys.join("/"))
        }
    }

    fn populate_hashmap(&mut self) {
        for keybinding in &self.general.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            self.keymap.insert(hash_value, keybinding.action.into());
        }
        for keybinding in &self.torrents_tab.keybindings {
            let hash_value = (keybinding.on, keybinding.modifier.into());
            self.keymap.insert(hash_value, keybinding.action.into());
        }
    }

    pub fn path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| utils::get_config_path(Self::FILENAME))
    }
}
