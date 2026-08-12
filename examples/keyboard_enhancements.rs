//! Cleanroom Rust port of upstream Go example: `examples/keyboard-enhancements/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A simple example illustrating how to enable enhanced keyboard support
//! (the Kitty keyboard protocol: key disambiguation and key event types).

use charming_bubbletea::color::{request_background_color, BackgroundColorMsg};
use charming_bubbletea::keyboard::KeyboardEnhancementsMsg;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::renderer::print_ln;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, KeyReleaseMsg, Msg, Program, View};
use charming_lipgloss::border;
use charming_lipgloss::color::light_dark;
use charming_lipgloss::{new_style, Color, Style};

/// Styles used by the example.
#[derive(Clone)]
struct Styles {
    /// The UI style. As in the upstream example, it is initialized from the
    /// terminal background color but never actually rendered.
    #[allow(dead_code)]
    ui: Style,
}

/// A model tracking which keyboard enhancements the terminal supports.
struct Model {
    supports_disambiguation: bool,
    supports_event_types: bool,
    styles: Styles,
}

/// Renders a lipgloss Color back into the string form accepted by
/// `border_foreground` (a hex string for TrueColor, the index for ANSI
/// colors).
fn color_to_str(c: &Color) -> String {
    match c {
        Color::TrueColor { r, g, b } => format!("#{:02X}{:02X}{:02X}", r, g, b),
        Color::Ansi256(i) => i.to_string(),
        Color::Ansi16(i) => i.to_string(),
        _ => String::new(),
    }
}

impl Model {
    /// Initialize styles based on whether the terminal background is dark.
    fn update_styles(&mut self, is_dark: bool) {
        let light_dark = light_dark(is_dark);
        let grey = light_dark(Color::parse("239"), Color::parse("245"));
        let dark_gray = light_dark(Color::parse("245"), Color::parse("239"));

        self.styles.ui = new_style()
            .foreground_color(grey)
            .border(border::normal_border(), &[true, false, false, false])
            .border_foreground(&[color_to_str(&dark_gray).as_str()]);
    }
}

/// Initial model, defaulting to dark styles.
fn initial_model() -> Model {
    let mut m = Model {
        supports_disambiguation: false,
        supports_event_types: false,
        styles: Styles { ui: new_style() },
    };
    m.update_styles(true);
    m
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        request_background_color()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        // Bubble Tea will send a KeyboardEnhancementsMsg on startup if the
        // terminal supports keyboard enhancements features.
        //
        // These features extend the capabilities of keyboard input beyond the
        // basic legacy support found in most terminals. This includes features
        // like:
        //  - Key disambiguation: Improved ability to distinguish between
        //    certain key presses like "enter" and "shift+enter" or "tab" and
        //    "ctrl+i".
        //  - Key event types: The ability to report different types of key
        //    events such as key presses and key releases.
        //
        // You can ask Bubble Tea to request additional keyboard enhancements
        // features by setting fields on the View's `keyboard_enhancements`
        // struct in the View method.
        if let Some(ke) = msg.as_any().downcast_ref::<KeyboardEnhancementsMsg>() {
            // Check which features were able to be enabled.
            self.supports_disambiguation = true; // This is always enabled when this msg is received.
            self.supports_event_types = ke.supports_event_types();
        } else if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            let s = k.0.to_string();
            if s == "ctrl+c" {
                return quit();
            }
            return print_ln(format_args!("  press: {}", s));
        } else if let Some(k) = msg.as_any().downcast_ref::<KeyReleaseMsg>() {
            return print_ln(format_args!("release: {}", k.0));
        } else if let Some(bg) = msg.as_any().downcast_ref::<BackgroundColorMsg>() {
            // Initialize styles.
            self.update_styles(bg.is_dark());
        }
        None
    }

    fn view(&self) -> View {
        let b = format!(
            "Terminal supports key releases: {}\nTerminal supports key disambiguation: {}\nThis demo logs key events. Press ctrl+c to quit.\n",
            self.supports_event_types, self.supports_disambiguation
        );
        let mut v = View::new(&b);

        // Attempt to enable reporting key event types (key presses and key
        // releases). By default, only key disambiguation is enabled which
        // improves the ability to distinguish between certain key presses
        // like "enter" and "shift+enter" or "tab" and "ctrl+i".
        v.keyboard_enhancements.report_event_types = true;

        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(initial_model());
    p.run()?;
    Ok(())
}
