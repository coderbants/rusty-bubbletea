//! Cleanroom Rust port of upstream Go source file: `key.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Key
//!
//! Key represents keyboard events, key types, runes, and matching helpers.
//! </public-docs>

use std::fmt;

/// <upstream-comment>
/// KeyType indicates the type of key pressed (e.g. Runes, Enter, Backspace, Esc).
/// </upstream-comment>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyType {
    /// <upstream-comment>
    /// KeyRunes is a key containing standard unicode characters.
    /// </upstream-comment>
    KeyRunes,
    /// <upstream-comment>
    /// KeyEnter is the Enter / Return key.
    /// </upstream-comment>
    KeyEnter,
    /// <upstream-comment>
    /// KeyBackspace is the Backspace key.
    /// </upstream-comment>
    KeyBackspace,
    /// <upstream-comment>
    /// KeyTab is the Tab key.
    /// </upstream-comment>
    KeyTab,
    /// <upstream-comment>
    /// KeySpace is the Space key.
    /// </upstream-comment>
    KeySpace,
    /// <upstream-comment>
    /// KeyEsc is the Escape key.
    /// </upstream-comment>
    KeyEsc,
    /// <upstream-comment>
    /// KeyUp is the Up Arrow key.
    /// </upstream-comment>
    KeyUp,
    /// <upstream-comment>
    /// KeyDown is the Down Arrow key.
    /// </upstream-comment>
    KeyDown,
    /// <upstream-comment>
    /// KeyRight is the Right Arrow key.
    /// </upstream-comment>
    KeyRight,
    /// <upstream-comment>
    /// KeyLeft is the Left Arrow key.
    /// </upstream-comment>
    KeyLeft,
    /// <upstream-comment>
    /// KeyHome is the Home key.
    /// </upstream-comment>
    KeyHome,
    /// <upstream-comment>
    /// KeyEnd is the End key.
    /// </upstream-comment>
    KeyEnd,
    /// <upstream-comment>
    /// KeyPgUp is the Page Up key.
    /// </upstream-comment>
    KeyPgUp,
    /// <upstream-comment>
    /// KeyPgDown is the Page Down key.
    /// </upstream-comment>
    KeyPgDown,
    /// <upstream-comment>
    /// KeyDelete is the Delete key.
    /// </upstream-comment>
    KeyDelete,
    /// <upstream-comment>
    /// KeyShiftTab is Shift+Tab key.
    /// </upstream-comment>
    KeyShiftTab,
    /// <upstream-comment>
    /// KeyCtrlC is Ctrl+C key.
    /// </upstream-comment>
    KeyCtrlC,
    /// <upstream-comment>
    /// KeyUnknown represents an unrecognized key.
    /// </upstream-comment>
    KeyUnknown,
}

/// <upstream-comment>
/// KeyMsg contains information about a keypress event.
/// </upstream-comment>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMsg {
    /// <upstream-comment>
    /// Type is the classification of the key pressed.
    /// </upstream-comment>
    pub key_type: KeyType,

    /// <upstream-comment>
    /// Runes are the unicode characters associated with the keypress.
    /// </upstream-comment>
    pub runes: Vec<char>,

    /// <upstream-comment>
    /// Alt indicates if the Alt modifier was active.
    /// </upstream-comment>
    pub alt: bool,
}

impl KeyMsg {
    /// Creates a new KeyMsg for a given KeyType.
    pub fn new(key_type: KeyType) -> Self {
        Self {
            key_type,
            runes: Vec::new(),
            alt: false,
        }
    }

    /// Creates a new KeyMsg for runes.
    pub fn from_runes(runes: &[char], alt: bool) -> Self {
        Self {
            key_type: KeyType::KeyRunes,
            runes: runes.to_vec(),
            alt,
        }
    }

    /// <upstream-comment>
    /// String returns a readable string representation of the key event.
    /// </upstream-comment>
    pub fn to_string_rep(&self) -> String {
        let base = match self.key_type {
            KeyType::KeyRunes => self.runes.iter().collect::<String>(),
            KeyType::KeyEnter => "enter".to_string(),
            KeyType::KeyBackspace => "backspace".to_string(),
            KeyType::KeyTab => "tab".to_string(),
            KeyType::KeySpace => " ".to_string(),
            KeyType::KeyEsc => "esc".to_string(),
            KeyType::KeyUp => "up".to_string(),
            KeyType::KeyDown => "down".to_string(),
            KeyType::KeyRight => "right".to_string(),
            KeyType::KeyLeft => "left".to_string(),
            KeyType::KeyHome => "home".to_string(),
            KeyType::KeyEnd => "end".to_string(),
            KeyType::KeyPgUp => "pgup".to_string(),
            KeyType::KeyPgDown => "pgdown".to_string(),
            KeyType::KeyDelete => "delete".to_string(),
            KeyType::KeyShiftTab => "shift+tab".to_string(),
            KeyType::KeyCtrlC => "ctrl+c".to_string(),
            KeyType::KeyUnknown => "unknown".to_string(),
        };

        if self.alt {
            format!("alt+{}", base)
        } else {
            base
        }
    }
}

impl fmt::Display for KeyMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_rep())
    }
}
