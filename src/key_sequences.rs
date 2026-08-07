//! Cleanroom Rust port of upstream Go source file: `key_sequences.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Key Sequences
//!
//! ANSI escape sequence parser, key sequence detection, and bracketed paste parsing.
//! </public-docs>

use crate::key::{KeyMsg, KeyType};

/// <upstream-comment>
/// Detects bracketed paste events from raw byte streams.
/// </upstream-comment>
pub fn detect_bracketed_paste(input: &[u8]) -> Option<(usize, String)> {
    const BP_START: &[u8] = b"\x1b[200~";
    const BP_END: &[u8] = b"\x1b[201~";

    if input.len() < BP_START.len() || &input[..BP_START.len()] != BP_START {
        return None;
    }

    let rest = &input[BP_START.len()..];
    if let Some(pos) = rest.windows(BP_END.len()).position(|w| w == BP_END) {
        let content_bytes = &rest[..pos];
        let total_consumed = BP_START.len() + pos + BP_END.len();
        let text = String::from_utf8_lossy(content_bytes).to_string();
        Some((total_consumed, text))
    } else {
        None
    }
}

/// <upstream-comment>
/// Detects known ANSI escape key sequences and returns matching KeyMsg.
/// </upstream-comment>
pub fn detect_sequence(input: &[u8]) -> Option<(usize, KeyMsg)> {
    if input.is_empty() {
        return None;
    }

    match input {
        [0x1b, b'[', b'A', ..] => Some((3, KeyMsg::new(KeyType::KeyUp))),
        [0x1b, b'[', b'B', ..] => Some((3, KeyMsg::new(KeyType::KeyDown))),
        [0x1b, b'[', b'C', ..] => Some((3, KeyMsg::new(KeyType::KeyRight))),
        [0x1b, b'[', b'D', ..] => Some((3, KeyMsg::new(KeyType::KeyLeft))),
        [0x1b, b'[', b'H', ..] => Some((3, KeyMsg::new(KeyType::KeyHome))),
        [0x1b, b'[', b'F', ..] => Some((3, KeyMsg::new(KeyType::KeyEnd))),
        [0x1b, b'[', b'5', b'~', ..] => Some((4, KeyMsg::new(KeyType::KeyPgUp))),
        [0x1b, b'[', b'6', b'~', ..] => Some((4, KeyMsg::new(KeyType::KeyPgDown))),
        [0x1b, b'[', b'3', b'~', ..] => Some((4, KeyMsg::new(KeyType::KeyDelete))),
        [0x1b, b'[', b'Z', ..] => Some((3, KeyMsg::new(KeyType::KeyShiftTab))),
        _ => None,
    }
}
