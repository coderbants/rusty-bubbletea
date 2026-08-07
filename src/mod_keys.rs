//! Cleanroom Rust port of upstream Go source file: `mod.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Key Modifiers
//!
//! Key modifier flags (`ModShift`, `ModAlt`, `ModCtrl`, `ModMeta`, `ModHyper`, `ModSuper`, `ModCapsLock`, `ModNumLock`, `ModScrollLock`).
//! </public-docs>

use crate::key::KeyMod;

/// ModShift modifier flag.
pub const MOD_SHIFT: KeyMod = KeyMod::SHIFT;
/// ModAlt modifier flag.
pub const MOD_ALT: KeyMod = KeyMod::ALT;
/// ModCtrl modifier flag.
pub const MOD_CTRL: KeyMod = KeyMod::CTRL;
/// ModMeta modifier flag.
pub const MOD_META: KeyMod = KeyMod::META;
/// ModHyper modifier flag.
pub const MOD_HYPER: KeyMod = KeyMod::HYPER;
/// ModSuper modifier flag.
pub const MOD_SUPER: KeyMod = KeyMod::SUPER;
