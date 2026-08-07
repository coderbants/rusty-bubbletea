//! Cleanroom Rust port of upstream Go source file: `key.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Key Messages & Events
//!
//! Key structures (`Key`, `KeyPressMsg`, `KeyReleaseMsg`), modifiers (`KeyMod`), and key symbol constants.
//! </public-docs>

use std::fmt;

/// KeyMod bitflags representing modifier keys (ctrl, alt, shift, meta, hyper, super).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyMod(pub u8);

impl KeyMod {
    /// Ctrl key modifier flag.
    pub const CTRL: KeyMod = KeyMod(1 << 0);
    /// Alt key modifier flag.
    pub const ALT: KeyMod = KeyMod(1 << 1);
    /// Shift key modifier flag.
    pub const SHIFT: KeyMod = KeyMod(1 << 2);
    /// Meta key modifier flag.
    pub const META: KeyMod = KeyMod(1 << 3);
    /// Hyper key modifier flag.
    pub const HYPER: KeyMod = KeyMod(1 << 4);
    /// Super key modifier flag.
    pub const SUPER: KeyMod = KeyMod(1 << 5);

    /// Checks if a modifier flag is contained.
    pub fn contains(&self, flag: KeyMod) -> bool {
        (self.0 & flag.0) != 0
    }
}

/// Special key symbols.
pub const KEY_UP: char = '\u{E000}';
/// KeyDown symbol.
pub const KEY_DOWN: char = '\u{E001}';
/// KeyRight symbol.
pub const KEY_RIGHT: char = '\u{E002}';
/// KeyLeft symbol.
pub const KEY_LEFT: char = '\u{E003}';
/// KeyHome symbol.
pub const KEY_HOME: char = '\u{E004}';
/// KeyEnd symbol.
pub const KEY_END: char = '\u{E005}';
/// KeyPgUp symbol.
pub const KEY_PG_UP: char = '\u{E006}';
/// KeyPgDown symbol.
pub const KEY_PG_DOWN: char = '\u{E007}';
/// KeyEnter symbol.
pub const KEY_ENTER: char = '\r';
/// KeyTab symbol.
pub const KEY_TAB: char = '\t';
/// KeyBackspace symbol.
pub const KEY_BACKSPACE: char = '\u{007F}';
/// KeyEscape symbol.
pub const KEY_ESCAPE: char = '\u{001B}';
/// KeySpace symbol.
pub const KEY_SPACE: char = ' ';

/// Key represents a Key press or release event in v2.0.8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// Text contains actual characters received.
    pub text: String,
    /// Modifier keys pressed.
    pub mod_keys: KeyMod,
    /// Key code character/rune.
    pub code: char,
    /// Shifted key code.
    pub shifted_code: Option<char>,
    /// Base key code on US PC-101 layout.
    pub base_code: Option<char>,
    /// Whether key is auto-repeating.
    pub is_repeat: bool,
}

impl Key {
    /// Creates a new simple Key.
    pub fn new(code: char, text: &str, mod_keys: KeyMod) -> Self {
        Self {
            text: text.to_string(),
            mod_keys,
            code,
            shifted_code: None,
            base_code: None,
            is_repeat: false,
        }
    }

    /// String representation of key event. Returns "space" for spacebar.
    pub fn to_string(&self) -> String {
        if self.code == ' ' {
            "space".to_string()
        } else if !self.text.is_empty() {
            self.text.clone()
        } else {
            self.keystroke()
        }
    }

    /// Keystroke representation with explicit modifier ordering (`ctrl+alt+shift...`).
    pub fn keystroke(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.mod_keys.contains(KeyMod::CTRL) {
            parts.push("ctrl".to_string());
        }
        if self.mod_keys.contains(KeyMod::ALT) {
            parts.push("alt".to_string());
        }
        if self.mod_keys.contains(KeyMod::SHIFT) {
            parts.push("shift".to_string());
        }
        if self.mod_keys.contains(KeyMod::META) {
            parts.push("meta".to_string());
        }

        let name = match self.code {
            KEY_UP => "up".to_string(),
            KEY_DOWN => "down".to_string(),
            KEY_RIGHT => "right".to_string(),
            KEY_LEFT => "left".to_string(),
            KEY_HOME => "home".to_string(),
            KEY_END => "end".to_string(),
            KEY_PG_UP => "pgup".to_string(),
            KEY_PG_DOWN => "pgdown".to_string(),
            KEY_ENTER => "enter".to_string(),
            KEY_TAB => "tab".to_string(),
            KEY_BACKSPACE => "backspace".to_string(),
            KEY_ESCAPE => "esc".to_string(),
            ' ' => "space".to_string(),
            c => c.to_string(),
        };

        if parts.is_empty() {
            name
        } else {
            parts.push(name);
            parts.join("+")
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// KeyPressMsg represents a key press message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPressMsg(pub Key);

impl fmt::Display for KeyPressMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

/// KeyReleaseMsg represents a key release message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyReleaseMsg(pub Key);

impl fmt::Display for KeyReleaseMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

/// KeyMsg trait/enum abstraction covering KeyPressMsg and KeyReleaseMsg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyMsg {
    /// Key press variant.
    Press(KeyPressMsg),
    /// Key release variant.
    Release(KeyReleaseMsg),
}

impl KeyMsg {
    /// Returns the underlying Key struct.
    pub fn key(&self) -> &Key {
        match self {
            KeyMsg::Press(k) => &k.0,
            KeyMsg::Release(k) => &k.0,
        }
    }
}

impl fmt::Display for KeyMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key().to_string())
    }
}
