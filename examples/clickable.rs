//! Cleanroom Rust port of upstream Go example: `examples/clickable/main.go`
//! (plus its `examples/clickable/words.go` support file)
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A demo of composable, clickable layers: click the background to spawn
//! dialog boxes, drag them around, and click the "Run Away" button to close
//! them.
//!
//! Deviations from upstream:
//! - `segmentio/ksuid` IDs are replaced with an incrementing counter
//!   (`dialog-N` / `button-N`) since the Rust port has no ksuid equivalent;
//!   IDs only need to be unique within the app.
//! - A dialog keeps a clone of the special-word style rather than a pointer
//!   to the model's style; since the background color is queried once at
//!   startup (before any dialog can be spawned), the behavior is identical.
//! - `math/rand` (words shuffling) is replaced with a small xorshift64*
//!   PRNG seeded from the system clock.

use std::sync::Mutex;
use std::time::SystemTime;

use charming_bubbletea::color::{request_background_color, BackgroundColorMsg};
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::mouse::{MouseButton, MouseMsg};
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::view::MouseMode;
use charming_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::border;
use charming_lipgloss::layer::{new_compositor, new_layer, Layer};
use charming_lipgloss::position::place;
use charming_lipgloss::size;
use charming_lipgloss::whitespace::{with_whitespace_chars, with_whitespace_style};
use charming_lipgloss::{new_style, Color, Style, LEFT, TOP};

/// LayerHitMsg is a message that is sent to the program when a layer is hit by
/// a mouse event. This is used to determine which layer in a composable view
/// was hit by the mouse event. The layer is identified by its ID, which is a
/// string that is unique to the layer.
#[derive(Debug, Clone)]
struct LayerHitMsg {
    id: String,
    mouse: MouseMsg,
}

/// The maximum number of dialogs we can spawn.
const MAX_DIALOGS: usize = 999;

/// Styles, mirroring the upstream global styles.
fn bg_text_style() -> Style {
    new_style()
        .foreground_color(Color::parse("239"))
        .padding(&[1, 2])
}

/// Whitespace options for the background, mirroring `bgWhitespace`.
fn bg_whitespace() -> Vec<charming_lipgloss::whitespace::Whitespace> {
    vec![
        with_whitespace_chars("/"),
        with_whitespace_style(new_style().foreground_color(Color::parse("238"))),
    ]
}

fn dialog_word_style() -> Style {
    new_style().foreground_color(Color::parse("#E7E1CC"))
}

fn dialog_style() -> Style {
    dialog_word_style()
        .width(36)
        .height(8)
        .padding(&[1, 3])
        .border(border::rounded_border(), &[true, true, true, true])
        .border_foreground(&["#874BFD"])
}

fn hovered_dialog_style() -> Style {
    dialog_style().border_foreground(&["#F25D94"])
}

const SPECIAL_WORD_LIGHT_COLOR: &str = "#43BF6D";
const SPECIAL_WORD_DARK_COLOR: &str = "#73F59F";

fn button_style() -> Style {
    new_style()
        .padding(&[0, 3])
        .foreground_color(Color::parse("#FFF7DB"))
        .background_color(Color::parse("#6124DF"))
}

fn hovered_button_style() -> Style {
    button_style().background_color(Color::parse("#FF5F87"))
}

/// A draggable dialog window.
struct Dialog {
    special_word_style: Style,
    id: String,
    button_id: String,
    x: usize,
    y: usize,
    text: String,
    hovering: bool,
    hovering_button: bool,
}

impl Dialog {
    /// Renders the "Run Away" button.
    fn button_view(&self) -> String {
        const LABEL: &str = "Run Away";
        if self.hovering_button {
            hovered_button_style().render(LABEL)
        } else {
            button_style().render(LABEL)
        }
    }

    /// Renders the dialog window content.
    fn window_view(&self) -> String {
        let style = if self.hovering {
            hovered_dialog_style()
        } else {
            dialog_style()
        };

        let s = self.special_word_style.render(&self.text)
            + &dialog_word_style().render(" draws near. Command?");
        style.render(&s)
    }

    /// Builds the layer tree for this dialog: the window layer with the
    /// button layer on top of it.
    fn view(&self) -> Layer {
        const H_GAP: usize = 3;
        const V_GAP: usize = 1;

        let window = self.window_view();
        let button = self.button_view();

        let button_x = size::width(&window).saturating_sub(size::width(&button)) - 1 - H_GAP;
        let button_y = size::height(&window).saturating_sub(size::height(&button)) - 1 - V_GAP;

        let button_layer = new_layer(&button, &[])
            .id(&self.button_id)
            .x(button_x as isize)
            .y(button_y as isize);

        new_layer(&window, &[button_layer])
            .id(&self.id)
            .x(self.x as isize)
            .y(self.y as isize)
    }
}

/// The main model.
struct Model {
    special_word_style: Style,
    width: usize,
    height: usize,
    dialogs: Vec<Dialog>,
    mouse_down: bool,
    press_id: String,
    drag_id: String,
    drag_offset_x: usize,
    drag_offset_y: usize,
    /// Counter replacing `segmentio/ksuid` IDs (see header deviation note).
    next_id: u64,
}

impl Model {
    /// Creates a new dialog centered on the given coordinates, mirroring
    /// `newDialog` in the upstream example.
    fn new_dialog(&mut self, x: usize, y: usize) -> Dialog {
        let mut d = Dialog {
            special_word_style: self.special_word_style.clone(),
            id: String::new(),
            button_id: String::new(),
            x: 0,
            y: 0,
            text: next_random_word(),
            hovering: false,
            hovering_button: false,
        };

        let dummy_view = d.window_view();
        let w = size::width(&dummy_view);
        let h = size::height(&dummy_view);

        d.x = clamp(
            x as isize - (w / 2) as isize,
            0,
            self.width as isize - w as isize,
        )
        .max(0) as usize;
        d.y = clamp(
            y as isize - (h / 2) as isize,
            0,
            self.height as isize - h as isize,
        )
        .max(0) as usize;

        self.next_id += 1;
        d.id = format!("dialog-{}", self.next_id);
        self.next_id += 1;
        d.button_id = format!("button-{}", self.next_id);

        d
    }

    /// Removes the dialog at the given index, mirroring `removeDialog`.
    fn remove_dialog(&mut self, index: usize) {
        if index >= self.dialogs.len() {
            return;
        }
        self.dialogs.remove(index);
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        batch(vec![request_background_color()])
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
        } else if let Some(bg) = msg.as_any().downcast_ref::<BackgroundColorMsg>() {
            let color = if bg.is_dark() {
                Color::parse(SPECIAL_WORD_DARK_COLOR)
            } else {
                Color::parse(SPECIAL_WORD_LIGHT_COLOR)
            };
            self.special_word_style = self.special_word_style.clone().foreground_color(color);
        } else if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "ctrl+c" | "esc" => return quit(),
                _ => {}
            }
        } else if let Some(lh) = msg.as_any().downcast_ref::<LayerHitMsg>() {
            let mouse = lh.mouse.mouse().clone();

            match &lh.mouse {
                MouseMsg::Click(_) => {
                    if mouse.button != MouseButton::MouseLeft {
                        return None;
                    }

                    // Initial press.
                    if !self.mouse_down {
                        self.mouse_down = true;
                        self.press_id = lh.id.clone();

                        // Did we press on a dialog box?
                        let mut drag_index = None;
                        for (i, d) in self.dialogs.iter().enumerate() {
                            if d.id != lh.id {
                                continue;
                            }

                            // Init drag.
                            self.drag_id = lh.id.clone();
                            self.drag_offset_x = mouse.x.saturating_sub(d.x);
                            self.drag_offset_y = mouse.y.saturating_sub(d.y);

                            if self.dialogs.len() >= 2 {
                                drag_index = Some(i);
                            }
                            break;
                        }

                        // Move the one we're going to drag to the end of the
                        // slice so that it gets the highest z-index when we do
                        // compositing later. There are, of course, lots of
                        // other ways you could manage the z-index, too.
                        if let Some(i) = drag_index {
                            let d = self.dialogs.remove(i);
                            self.dialogs.push(d);
                        }
                    }
                }
                MouseMsg::Motion(_) => {
                    // Dragging.
                    if self.mouse_down && !self.drag_id.is_empty() {
                        // Find the dialog box we're dragging.
                        for d in self.dialogs.iter_mut() {
                            if d.id != self.drag_id {
                                continue;
                            }

                            // Move the dialog box with the cursor.
                            let w = size::width(&d.window_view()) as isize;
                            let h = size::height(&d.window_view()) as isize;
                            d.x = clamp(
                                mouse.x as isize - self.drag_offset_x as isize,
                                0,
                                self.width as isize - w,
                            )
                            .max(0) as usize;
                            d.y = clamp(
                                mouse.y as isize - self.drag_offset_y as isize,
                                0,
                                self.height as isize - h,
                            )
                            .max(0) as usize;

                            break;
                        }
                    }

                    // Are we hovering over a dialog box?
                    for d in self.dialogs.iter_mut() {
                        d.hovering = false;
                        d.hovering_button = false;

                        if d.id == lh.id {
                            d.hovering = true;
                            continue;
                        }
                        if d.button_id == lh.id {
                            d.hovering = true;
                            d.hovering_button = true;
                            continue;
                        }
                    }
                }
                MouseMsg::Release(_) => {
                    // Make sure we're releasing on something with an ID. A
                    // successful click is a press and release.
                    if self.press_id.is_empty() {
                        return None;
                    }

                    // Did we click a button?
                    let mut button_index = None;
                    for (i, d) in self.dialogs.iter().enumerate() {
                        if lh.id == d.button_id && self.press_id == d.button_id {
                            button_index = Some(i);
                            break;
                        }
                    }
                    if let Some(i) = button_index {
                        // "Close" the window.
                        self.remove_dialog(i);
                    }

                    // Clicking the background spawns a new dialog.
                    if lh.id == "bg" && self.press_id == "bg" && self.dialogs.len() < MAX_DIALOGS {
                        let d = self.new_dialog(mouse.x, mouse.y);
                        self.dialogs.push(d);
                    }

                    self.mouse_down = false;
                    self.drag_id = String::new();
                    self.press_id = String::new();
                }
                MouseMsg::Wheel(_) => {}
            }
        }

        None
    }

    fn view(&self) -> View {
        let mut v = View::default();
        let mut body = String::new();

        let n = self.dialogs.len();
        if n > 0 {
            body += "Drag to move. ";
        }
        if n == 0 && n < MAX_DIALOGS {
            body += "Click to spawn.";
        } else if (1..MAX_DIALOGS).contains(&n) {
            body += &format!("Click to spawn up to {} more.", MAX_DIALOGS - n);
        }
        body += "\n\nPress q to quit.";

        let bg = place(
            self.width,
            self.height,
            TOP,
            LEFT,
            &bg_text_style().render(&body),
            &bg_whitespace(),
        );

        if std::env::var("CLICK_DEBUG").is_ok() {
            eprintln!("BG: {:?}", bg);
        }
        let mut root = new_layer(&bg, &[]).id("bg");
        for (i, d) in self.dialogs.iter().enumerate() {
            root.add_layers(&[d.view().z((i + 1) as isize)]);
        }

        let mut comp = new_compositor(&[root]);

        v.mouse_mode = MouseMode::MouseModeAllMotion;
        v.alt_screen = true;
        let content = comp.render();
        if std::env::var("CLICK_DEBUG").is_ok() {
            eprintln!("COMP: {:?}", content);
        }
        v.set_content(&content);
        v.on_mouse = Some(std::sync::Arc::new(move |msg: MouseMsg| {
            let (x, y) = {
                let m = msg.mouse();
                (m.x, m.y)
            };
            let id = comp.hit(x, y).id().to_string();
            if id.is_empty() {
                return None;
            }
            Some(Box::new(move || {
                Some(Box::new(LayerHitMsg { id, mouse: msg }))
            }))
        }));

        v
    }
}

/// Clamps n to [min, max], mirroring the upstream `clamp`.
fn clamp(n: isize, min: isize, max: isize) -> isize {
    if n < min {
        return min;
    }
    if n > max {
        return max;
    }
    n
}

// ---------------------------------------------------------------------------
// Random words (port of `examples/clickable/words.go`)
// ---------------------------------------------------------------------------

const UNCAPITALIZED: &str = " of a an and ’n’ ";

const ADJECTIVES: [&str; 56] = [
    "a hot",
    "a cute",
    "a fresh",
    "a nice",
    "a lovely",
    "an eager",
    "a soft",
    "an expensive",
    "a new",
    "an old",
    "a happy",
    "a messy",
    "a good",
    "a bad",
    "a cheesy",
    "a friendly",
    "a free",
    "a cold",
    "a gorgeous",
    "a glamorous",
    "a handsome",
    "an exquisite",
    "a tantalizing",
    "a suspicious",
    "an american",
    "a wooden",
    "a golden",
    "a dirty",
    "a hairy",
    "a lukewarm",
    "a burning hot",
    "a shiny",
    "a rogue",
    "a green",
    "a late night",
    "a mass produced",
    "a handmade",
    "a wild",
    "a clean",
    "a rugged",
    "the #1",
    "the best",
    "the worst",
    "a famous",
    "an infamous",
    "a clever",
    "a microwaved",
    "a 3D printed",
    "your favorite",
    "your least favorite",
    "someone’s",
    "a precious",
    "a fake",
    "a genuine",
    "a bejeweled",
    "a good-smelling",
];

const NOUNS: [&str; 41] = [
    "pear",
    "banana",
    "bowl of ramen",
    "currywurst",
    "quince",
    "pie",
    "cake",
    "burrito",
    "sushi",
    "basket of fish ’n’ chips",
    "burger",
    "kohlrabi",
    "pineapple",
    "cantaloupe",
    "sausage roll",
    "yuzu",
    "grapefruit",
    "espresso shot",
    "sandwich",
    "bowl of chow mein",
    "lemon",
    "cup of coffee",
    "bottle of hot sauce",
    "can of beer",
    "glass of wine",
    "muffin",
    "bagel",
    "glass of champagne",
    "bottle of rosé",
    "pengu",
    "badger",
    "mango",
    "okonomiyaki",
    "meatball",
    "box of wine",
    "artichoke",
    "TUI",
    "linux distro",
    "dotfile",
    "weißwurst",
    "computer",
];

/// The shuffled word stacks, initialized on first use (mirroring the
/// upstream `sync.Once` shuffle).
static WORDS: Mutex<Option<WordStack>> = Mutex::new(None);

/// The word stacks.
struct WordStack {
    adjectives: Vec<&'static str>,
    nouns: Vec<&'static str>,
}

impl WordStack {
    /// Shuffles the words with a random shuffle, mirroring `shuffleWords`.
    fn new() -> WordStack {
        let mut adjectives = ADJECTIVES.to_vec();
        let mut nouns = NOUNS.to_vec();
        shuffle(&mut adjectives);
        shuffle(&mut nouns);
        WordStack { adjectives, nouns }
    }

    /// Cycles both stacks and returns the next capitalized word pair,
    /// mirroring `nextRandomWord`.
    fn next(&mut self) -> String {
        self.adjectives.rotate_left(1);
        self.nouns.rotate_left(1);
        capitalize(&format!("{} {}", self.adjectives[0], self.nouns[0]))
    }
}

/// Returns the next random word, mirroring `nextRandomWord`.
fn next_random_word() -> String {
    let mut words = WORDS.lock().unwrap();
    if words.is_none() {
        *words = Some(WordStack::new());
    }
    words.as_mut().unwrap().next()
}

/// Randomly shuffles the given list (Fisher-Yates), mirroring
/// `rand.Shuffle`.
fn shuffle<T>(v: &mut [T]) {
    for i in (1..v.len()).rev() {
        let j = (rand_u64() % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
}

/// Capitalizes the first letter of each word, leaving the words in
/// `UNCAPITALIZED` (besides the first) lowercase, mirroring `capitalize`.
fn capitalize(s: &str) -> String {
    let uncap: Vec<String> = UNCAPITALIZED
        .split_whitespace()
        .map(|w| w.to_string())
        .collect();
    s.split_whitespace()
        .enumerate()
        .map(|(i, w)| {
            if i > 0 && uncap.iter().any(|u| u == w) {
                w.to_string()
            } else {
                title(w)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Title-cases a single word (uppercases the first character), mirroring
/// `strings.Title` on a single word.
fn title(w: &str) -> String {
    let mut cs = w.chars();
    match cs.next() {
        Some(c) => c.to_uppercase().collect::<String>() + cs.as_str(),
        None => String::new(),
    }
}

/// A tiny xorshift64* PRNG seeded from the system clock, standing in for
/// `math/rand`.
static PRNG: Mutex<u64> = Mutex::new(0);

fn rand_u64() -> u64 {
    let mut g = PRNG.lock().unwrap();
    if *g == 0 {
        *g = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
    }
    let mut x = *g;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *g = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        special_word_style: new_style(),
        width: 0,
        height: 0,
        dialogs: Vec::new(),
        mouse_down: false,
        press_id: String::new(),
        drag_id: String::new(),
        drag_offset_x: 0,
        drag_offset_y: 0,
        next_id: 0,
    });
    let _ = p.run()?;
    Ok(())
}
