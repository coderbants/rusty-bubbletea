//! Cleanroom Rust port of upstream Go source file: `keyboard.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Keyboard Enhancements
//!
//! Kitty keyboard protocol enhancement message structures and capability queries.
//! </public-docs>

/// Bitmask constants matching Kitty keyboard protocol enhancements.
pub const KITTY_REPORT_EVENT_TYPES: i32 = 1 << 0;
/// Report alternate keys capability flag.
pub const KITTY_REPORT_ALTERNATE_KEYS: i32 = 1 << 1;
/// Report all keys as escape codes capability flag.
pub const KITTY_REPORT_ALL_KEYS_AS_ESCAPE_CODES: i32 = 1 << 2;
/// Report associated keys capability flag.
pub const KITTY_REPORT_ASSOCIATED_KEYS: i32 = 1 << 3;

/// KeyboardEnhancements struct matching tea.KeyboardEnhancements in v2.0.8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyboardEnhancements {
    /// Report press, release, and repeat events.
    pub report_event_types: bool,
    /// Report alternate keys.
    pub report_alternate_keys: bool,
    /// Report all keys as escape codes.
    pub report_all_keys_as_escape_codes: bool,
    /// Report associated text.
    pub report_associated_text: bool,
}

/// KeyboardEnhancementsMsg is sent when the terminal supports keyboard enhancements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardEnhancementsMsg {
    /// Flags bitmask of enabled features.
    pub flags: i32,
}

impl KeyboardEnhancementsMsg {
    /// Returns whether terminal supports key disambiguation.
    pub fn supports_key_disambiguation(&self) -> bool {
        self.flags > 0
    }

    /// Returns whether terminal supports reporting press, release, and repeat event types.
    pub fn supports_event_types(&self) -> bool {
        (self.flags & KITTY_REPORT_EVENT_TYPES) != 0
    }

    /// Returns whether terminal supports alternate key codes.
    pub fn supports_alternate_keys(&self) -> bool {
        (self.flags & KITTY_REPORT_ALTERNATE_KEYS) != 0
    }

    /// Returns whether terminal supports reporting all keys as escape codes.
    pub fn supports_all_keys_as_escape_codes(&self) -> bool {
        (self.flags & KITTY_REPORT_ALL_KEYS_AS_ESCAPE_CODES) != 0
    }

    /// Returns whether terminal supports associated text.
    pub fn supports_associated_text(&self) -> bool {
        (self.flags & KITTY_REPORT_ASSOCIATED_KEYS) != 0
    }
}
