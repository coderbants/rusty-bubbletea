//! Cleanroom Rust port of upstream Go source file: `environ.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! <public-docs>
//! # Environment Messages
//!
//! `EnvMsg` representing program environment variables for local/SSH sessions in Bubble Tea v2.0.8.
//! </public-docs>

use std::collections::HashMap;

/// EnvMsg represents program environment variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvMsg {
    /// Environment variables map.
    pub vars: HashMap<String, String>,
}

impl EnvMsg {
    /// Creates a new EnvMsg from key-value pairs.
    pub fn new(pairs: Vec<(String, String)>) -> Self {
        let vars = pairs.into_iter().collect();
        Self { vars }
    }

    /// Creates an EnvMsg from the process environment.
    pub fn from_std() -> Self {
        let vars = std::env::vars().collect();
        Self { vars }
    }

    /// Returns value of environment variable or empty string if unset.
    pub fn getenv(&self, key: &str) -> String {
        self.vars.get(key).cloned().unwrap_or_default()
    }

    /// Retrieves value and boolean presence flag.
    pub fn lookup_env(&self, key: &str) -> (String, bool) {
        match self.vars.get(key) {
            Some(v) => (v.clone(), true),
            None => (String::new(), false),
        }
    }
}
