//! Cleanroom Rust port of upstream Go source file: `program.go`
//! Upstream Target Tag / Version: `v1.3.4`
//!
//! <public-docs>
//! # Program
//!
//! Program runner managing the Elm architecture event loop, model updates, rendering, and async command dispatches.
//! </public-docs>

use crate::commands::{BatchMsg, QuitMsg};
use crate::model::{Model, Msg};
use std::sync::mpsc::{channel, Receiver, Sender};

/// <upstream-comment>
/// Program is the runner for a Bubble Tea application.
/// </upstream-comment>
pub struct Program<M: Model> {
    model: M,
}

impl<M: Model> Program<M> {
    /// <upstream-comment>
    /// NewProgram creates a new Program instance for the given Model.
    /// </upstream-comment>
    pub fn new(model: M) -> Self {
        Self { model }
    }

    /// Helper to process a message and dispatch generated commands.
    fn handle_msg(&mut self, msg: Box<dyn Msg>, tx: &Sender<Box<dyn Msg>>) -> bool {
        if msg.as_ref().as_any().is::<QuitMsg>() {
            return true;
        }

        if let Some(_batch_msg) = msg.as_ref().as_any().downcast_ref::<BatchMsg>() {
            return false;
        }

        let cmd = self.model.update(msg);
        let view = self.model.view();
        if !view.is_empty() {
            print!("{}", view);
        }

        if let Some(c) = cmd {
            if let Some(new_msg) = c() {
                let _ = tx.send(new_msg);
            }
        }
        false
    }

    /// <upstream-comment>
    /// Run initializes the model, starts the event loop, executes commands, and runs until quit.
    /// </upstream-comment>
    pub fn run(mut self) -> Result<M, Box<dyn std::error::Error>> {
        let (tx, rx): (Sender<Box<dyn Msg>>, Receiver<Box<dyn Msg>>) = channel();

        // Run initial command
        if let Some(cmd) = self.model.init() {
            if let Some(msg) = cmd() {
                let _ = tx.send(msg);
            }
        }

        // Print initial view
        let view = self.model.view();
        if !view.is_empty() {
            print!("{}", view);
        }

        // Event loop processing
        while let Ok(msg) = rx.recv() {
            if self.handle_msg(msg, &tx) {
                break;
            }
        }

        Ok(self.model)
    }
}
