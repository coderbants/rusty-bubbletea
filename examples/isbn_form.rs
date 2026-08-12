//! Cleanroom Rust port of upstream Go example: `examples/isbn-form/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A form with two text inputs: an ISBN-13 input (validated with a checksum)
//! and a book title input (validated against banned words). Switch between
//! the inputs with `up`/`down` and press `enter` to submit once both inputs
//! are valid.

use charming_bubbles::textinput;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::{batch, quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::{new_style, Color, Style};

// Charmtone palette colors (from `charmbracelet/x/exp/charmtone`).
const TANG: &str = "#FF985A";
const ANCHOVY: &str = "#719AFC";
const GUAC: &str = "#12C78F";
const CHERRY: &str = "#FF388B";

/// ErrMsg represents a validation error message.
struct ErrMsg(String);

fn input_style() -> Style {
    new_style().foreground_color(Color::parse(TANG))
}

fn continue_style() -> Style {
    new_style().foreground_color(Color::parse(ANCHOVY))
}

fn valid_style() -> Style {
    new_style().foreground_color(Color::parse(GUAC))
}

fn err_style() -> Style {
    new_style().foreground_color(Color::parse(CHERRY))
}

struct Model {
    isbn_input: textinput::Model,
    title_input: textinput::Model,
    focused_input: usize,
    err: Option<String>,
}

impl Model {
    /// CanFindBook returns whether the find button is to be pressed.
    fn can_find_book(&self) -> bool {
        let correct_isbn_given =
            self.isbn_input.err.is_none() && !self.isbn_input.value().is_empty();
        let correct_title_given =
            self.title_input.err.is_none() && !self.title_input.value().is_empty();

        correct_isbn_given && correct_title_given
    }

    fn initial_model() -> Self {
        let mut isbn_input = textinput::new();
        let _ = isbn_input.focus();
        isbn_input.placeholder = "978-X-XXX-XXXXX-X".to_string();
        isbn_input.char_limit = 17;
        isbn_input.set_width(30);
        isbn_input.prompt = String::new();
        isbn_input.validate = Some(Box::new(isbn13_validator));

        let mut title_input = textinput::new();
        title_input.blur();
        title_input.placeholder = "Title".to_string();
        title_input.char_limit = 100;
        title_input.set_width(100);
        title_input.prompt = String::new();
        title_input.validate = Some(Box::new(book_title_validator));

        Model {
            isbn_input,
            title_input,
            focused_input: 0,
            err: None,
        }
    }
}

/// Validator function to ensure valid input: a valid ISBN-13 looks like
/// `978-3-548-37257-0` or `9783548372570` without any spaces.
fn isbn13_validator(s: &str) -> Result<(), String> {
    // Remove dashes.
    let s = s.replace('-', "");
    if s.len() != 13 {
        return Err("ISBN is of wrong length".to_string());
    }

    if !s.chars().all(|c| c.is_ascii_digit()) {
        return Err("ISBN contains invalid characters".to_string());
    }

    let gs1_prefix = &s[..3];
    if gs1_prefix != "978" && gs1_prefix != "979" {
        return Err("ISBN has invalid GS1 prefix".to_string());
    }

    // The last digit, the check digit, must make the checksum a multiple of
    // 10. All digits are added up after being multiplied by either 1 or 3
    // alternately.
    let sum: usize = s
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let mut n = c.to_digit(10).unwrap() as usize;
            // Multiply the uneven indices by 3.
            if i % 2 != 0 {
                n *= 3;
            }
            n
        })
        .sum();

    if !sum.is_multiple_of(10) {
        return Err("ISBN has invalid check digit".to_string());
    }

    Ok(())
}

const BANNED_TITLE_WORDS: &[&str] = &[
    "very", "bad", "words", "that", "should", "not", "appear", "in", "book", "titles",
];

fn book_title_validator(s: &str) -> Result<(), String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("Book title is empty".to_string());
    }

    for banned_word in BANNED_TITLE_WORDS {
        if s.contains(banned_word) {
            return Err(format!("Book title contains banned word {:?}", banned_word));
        }
    }

    Ok(())
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        Some(Box::new(|| Some(textinput::blink())))
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "up" | "down" => {
                    // Switch between text inputs.
                    match self.focused_input {
                        0 => {
                            self.focused_input = 1;
                            let _ = self.title_input.focus();
                            self.isbn_input.blur();
                        }
                        _ => {
                            self.focused_input = 0;
                            let _ = self.isbn_input.focus();
                            self.title_input.blur();
                        }
                    }
                }
                "enter" => {
                    // Enter is blocked until all inputs are ok.
                    if self.can_find_book() {
                        return quit();
                    }
                }
                "ctrl+c" | "esc" => return quit(),
                _ => {}
            }
        }

        // We handle errors just like any other message.
        if let Some(err) = msg.as_any().downcast_ref::<ErrMsg>() {
            self.err = Some(err.0.clone());
            return None;
        }

        let isbn_command = self.isbn_input.update(msg);
        let title_command = self.title_input.update(msg);

        batch(vec![isbn_command, title_command])
    }

    fn view(&self) -> View {
        let mut continue_text = String::new();
        if self.can_find_book() {
            continue_text = continue_style().render("Find ->");
        }

        let mut isbn_error_text = String::new();
        if !self.isbn_input.value().is_empty() {
            if let Some(err) = &self.isbn_input.err {
                isbn_error_text = err_style().render(err);
            } else {
                isbn_error_text = valid_style().render("Valid ISBN");
            }
        }

        let mut title_error_text = String::new();
        if !self.title_input.value().is_empty() {
            if let Some(err) = &self.title_input.err {
                title_error_text = err_style().render(err);
            } else {
                title_error_text = valid_style().render("Valid title");
            }
        }

        View::new(
            &(format!(
                " Search book:\n {} \n {} \n {} \n\n {} \n {} \n {} \n\n {} \n",
                input_style().width(30).render("ISBN"),
                self.isbn_input.view(),
                isbn_error_text,
                input_style().width(30).render("Title"),
                self.title_input.view(),
                title_error_text,
                continue_text,
            )),
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
