//! Cleanroom Rust port of upstream Go source file: `profile.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Terminal Color Profile
//!
//! `ColorProfileMsg` representing detected terminal color profile capabilities in Bubble Tea v2.0.8.
//! </public-docs>

/// ColorProfile enum representing terminal color support level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProfile {
    /// TrueColor (24-bit RGB direct color).
    TrueColor,
    /// ANSI256 (256 indexed colors).
    ANSI256,
    /// ANSI (16 basic colors).
    ANSI,
    /// Ascii (monochrome/no color).
    Ascii,
}

/// ColorProfileMsg describes the terminal's color profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorProfileMsg {
    /// Profile enum.
    pub profile: ColorProfile,
}
