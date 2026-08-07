//! Cleanroom Rust port of upstream Go source file: `input.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Input Event Translator
//!
//! Helper logic translating raw terminal events into Bubble Tea v2.0.8 message structs.
//! </public-docs>

use crate::model::Msg;

/// InputEventTranslator helper.
pub fn translate_input_event(msg: Box<dyn Msg>) -> Box<dyn Msg> {
    msg
}
