use std::fmt;

use serde::{Deserialize, Serialize};

use super::Modifier;

/// A keyboard chord such as `Ctrl+Shift+Esc`, stored in JSON as that string.
///
/// Grammar (case-insensitive, `+`-separated, spaces around tokens ignored): zero or more distinct
/// modifiers (`Ctrl`/`Control`, `Shift`, `Alt`, `Win`/`Windows`) followed by exactly one key.
/// Mirrors `parseShortcut` in `src/shared/schema/shortcut.ts`; shared fixtures keep them equal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Shortcut {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

/// Keys a shortcut may end with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// `A`–`Z`, stored upper-case.
    Letter(char),
    /// `0`–`9`.
    Digit(u8),
    /// `F1`–`F24`.
    Function(u8),
    Named(NamedKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    Esc,
    Tab,
    Enter,
    Space,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    PrintScreen,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    MediaPlayPause,
    MediaNext,
    MediaPrevious,
    MediaStop,
}

const NAMED: [(&str, NamedKey); 23] = [
    ("Esc", NamedKey::Esc),
    ("Tab", NamedKey::Tab),
    ("Enter", NamedKey::Enter),
    ("Space", NamedKey::Space),
    ("Backspace", NamedKey::Backspace),
    ("Delete", NamedKey::Delete),
    ("Insert", NamedKey::Insert),
    ("Home", NamedKey::Home),
    ("End", NamedKey::End),
    ("PageUp", NamedKey::PageUp),
    ("PageDown", NamedKey::PageDown),
    ("Up", NamedKey::Up),
    ("Down", NamedKey::Down),
    ("Left", NamedKey::Left),
    ("Right", NamedKey::Right),
    ("PrintScreen", NamedKey::PrintScreen),
    ("VolumeUp", NamedKey::VolumeUp),
    ("VolumeDown", NamedKey::VolumeDown),
    ("VolumeMute", NamedKey::VolumeMute),
    ("MediaPlayPause", NamedKey::MediaPlayPause),
    ("MediaNext", NamedKey::MediaNext),
    ("MediaPrevious", NamedKey::MediaPrevious),
    ("MediaStop", NamedKey::MediaStop),
];

/// Accepted alternative spellings, mapped to the canonical name.
const ALIASES: [(&str, &str); 3] = [("Escape", "Esc"), ("Return", "Enter"), ("Del", "Delete")];

impl Shortcut {
    /// Parses a shortcut string.
    ///
    /// # Errors
    ///
    /// A plain-English message naming the problem.
    pub fn parse(text: &str) -> Result<Self, String> {
        let tokens: Vec<&str> = text.split('+').map(str::trim).collect();
        if tokens.iter().any(|token| token.is_empty()) {
            return Err(format!("shortcut `{text}` has an empty part"));
        }
        let (last, modifier_tokens) = tokens
            .split_last()
            .ok_or_else(|| "shortcut must not be empty".to_owned())?;

        let mut modifiers = Vec::new();
        for token in modifier_tokens {
            let modifier = parse_modifier(token)
                .ok_or_else(|| format!("`{token}` in shortcut `{text}` is not a modifier"))?;
            if modifiers.contains(&modifier) {
                return Err(format!("shortcut `{text}` repeats `{token}`"));
            }
            modifiers.push(modifier);
        }
        if parse_modifier(last).is_some() {
            return Err(format!(
                "shortcut `{text}` must end with a key, not a modifier"
            ));
        }
        let key = parse_key(last).ok_or_else(|| format!("`{last}` is not a supported key"))?;
        Ok(Self { modifiers, key })
    }
}

fn parse_modifier(token: &str) -> Option<Modifier> {
    match token.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some(Modifier::Ctrl),
        "shift" => Some(Modifier::Shift),
        "alt" => Some(Modifier::Alt),
        "win" | "windows" => Some(Modifier::Win),
        _ => None,
    }
}

fn parse_key(token: &str) -> Option<Key> {
    let mut chars = token.chars();
    if let (Some(only), None) = (chars.next(), chars.next()) {
        if only.is_ascii_alphabetic() {
            return Some(Key::Letter(only.to_ascii_uppercase()));
        }
        if let Some(digit) = only.to_digit(10) {
            return u8::try_from(digit).ok().map(Key::Digit);
        }
    }
    if let Some(number) = token
        .strip_prefix(['F', 'f'])
        .and_then(|rest| rest.parse::<u8>().ok())
        && (1..=24).contains(&number)
        && !token[1..].starts_with('0')
    {
        return Some(Key::Function(number));
    }
    let canonical = ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(token))
        .map_or(token, |(_, canonical)| *canonical);
    NAMED
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(canonical))
        .map(|(_, key)| Key::Named(*key))
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Letter(letter) => write!(f, "{letter}"),
            Self::Digit(digit) => write!(f, "{digit}"),
            Self::Function(number) => write!(f, "F{number}"),
            Self::Named(named) => {
                let name = NAMED
                    .iter()
                    .find(|(_, key)| key == named)
                    .map_or("?", |(name, _)| *name);
                f.write_str(name)
            }
        }
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.modifiers {
            let name = match modifier {
                Modifier::Ctrl => "Ctrl",
                Modifier::Shift => "Shift",
                Modifier::Alt => "Alt",
                Modifier::Win => "Win",
            };
            write!(f, "{name}+")?;
        }
        write!(f, "{}", self.key)
    }
}

impl TryFrom<String> for Shortcut {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::parse(&text)
    }
}

impl From<Shortcut> for String {
    fn from(shortcut: Shortcut) -> Self {
        shortcut.to_string()
    }
}
