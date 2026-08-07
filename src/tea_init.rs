//! Cleanroom Rust port of upstream Go source file: `tea_init.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Tea Init
//!
//! Model initialization logic matching tea_init.go.
//! </public-docs>

use crate::model::{Cmd, Model};

/// Model initialization helper.
pub fn init_model<M: Model>(m: &M) -> Cmd {
    m.init()
}
