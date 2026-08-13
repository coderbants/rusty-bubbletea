//! Cleanroom Rust port of upstream Go example: `examples/print-key/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! Prints key events to the terminal above the program and quits on ctrl+c.

use rusty_bubbletea::keyboard::KeyboardEnhancements;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::{Cmd, KeyPressMsg, KeyboardEnhancementsMsg, Msg, Program, View};

struct KeyModel;

impl ModelTrait for KeyModel {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(enh) = msg.as_any().downcast_ref::<KeyboardEnhancementsMsg>() {
            return rusty_bubbletea::renderer::print_ln(format_args!(
                "Keyboard enhancements: EventTypes: {}",
                enh.supports_event_types()
            ));
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if k.0.to_string() == "ctrl+c" {
                return rusty_bubbletea::quit();
            }
            // Mirror upstream `fmt.Sprintf("(%T) You pressed: %s", msg, ...)`
            // with Go's type name for KeyPressMsg.
            let mut format = format!("(tea.KeyPressMsg) You pressed: {}", k.0);
            if !k.0.text.is_empty() {
                format += &format!(" (text: {:?})", k.0.text);
            }
            return rusty_bubbletea::renderer::print_ln(format_args!("{}", format));
        }

        if let Some(k) = msg
            .as_any()
            .downcast_ref::<rusty_bubbletea::KeyReleaseMsg>()
        {
            let mut format = format!("(tea.KeyReleaseMsg) You pressed: {}", k.0);
            if !k.0.text.is_empty() {
                format += &format!(" (text: {:?})", k.0.text);
            }
            return rusty_bubbletea::renderer::print_ln(format_args!("{}", format));
        }

        None
    }

    fn view(&self) -> View {
        let mut v = View::new(
            "Press any key to see its details printed to the terminal. Press 'ctrl+c' to quit.",
        );
        v.keyboard_enhancements = KeyboardEnhancements {
            report_event_types: true,
            ..KeyboardEnhancements::default()
        };
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(KeyModel);
    p.run()?;
    Ok(())
}
