//! Cleanroom Rust port of upstream Go example: `examples/list-fancy/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A fancy list with a custom title bar, status bar, spinner, pagination,
//! help menu and a random item generator. Press `s` to toggle the spinner,
//! `T` to toggle the title bar, `S` to toggle the status bar, `P` to toggle
//! pagination, `H` to toggle help, `a` to add a random item, `enter` to
//! choose an item and `x` or `backspace` to delete it.
//!
//! Cleanroom Rust port of upstream Go source file: `examples/list-fancy/delegate.go`
//! Cleanroom Rust port of upstream Go source file: `examples/list-fancy/randomitems.go`

use std::any::Any;

use rusty_bubbles::key;
use rusty_bubbles::list;
use rusty_bubbles::list::FilterState;
use rusty_bubbletea::color::BackgroundColorMsg;
use rusty_bubbletea::model::Model as ModelTrait;
use rusty_bubbletea::screen::WindowSizeMsg;
use rusty_bubbletea::{batch, request_background_color, Cmd, KeyPressMsg, Msg, Program, View};
use rusty_lipgloss::color::light_dark;
use rusty_lipgloss::{new_style, Color, Style};

#[derive(Clone)]
struct Styles {
    app: Style,
    title: Style,
    status_message: Style,
}

impl Styles {
    fn new(dark_bg: bool) -> Self {
        Styles {
            app: new_style().padding(&[1, 2]),
            title: new_style()
                .foreground_color(Color::parse("#FFFDF5"))
                .background_color(Color::parse("#25A065"))
                .padding(&[0, 1]),
            status_message: new_style().foreground_color(light_dark(dark_bg)(
                Color::parse("#04B575"),
                Color::parse("#04B575"),
            )),
        }
    }
}

#[derive(Debug, Clone)]
struct Item {
    title: String,
    description: String,
}

impl list::Item for Item {
    fn filter_value(&self) -> String {
        self.title.clone()
    }

    fn box_clone(&self) -> Box<dyn list::Item + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_default_item(&self) -> Option<&dyn list::DefaultItem> {
        Some(self)
    }
}

impl list::DefaultItem for Item {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// DelegateKeyMap holds the keybindings used by the item delegate, mirroring
/// the upstream `delegateKeyMap` and its `ShortHelp`/`FullHelp`
/// implementations.
#[derive(Clone)]
struct DelegateKeyMap {
    choose: key::Binding,
    remove: key::Binding,
}

fn new_delegate_key_map() -> DelegateKeyMap {
    DelegateKeyMap {
        choose: key::new_binding(vec![
            key::with_keys(&["enter"]),
            key::with_help("enter", "choose"),
        ]),
        remove: key::new_binding(vec![
            key::with_keys(&["x", "backspace"]),
            key::with_help("x", "delete"),
        ]),
    }
}

/// ListKeyMap holds the keybindings for the list itself, mirroring the
/// upstream `listKeyMap`.
#[derive(Clone)]
struct ListKeyMap {
    toggle_spinner: key::Binding,
    toggle_title_bar: key::Binding,
    toggle_status_bar: key::Binding,
    toggle_pagination: key::Binding,
    toggle_help_menu: key::Binding,
    insert_item: key::Binding,
}

fn new_list_key_map() -> ListKeyMap {
    ListKeyMap {
        insert_item: key::new_binding(vec![
            key::with_keys(&["a"]),
            key::with_help("a", "add item"),
        ]),
        toggle_spinner: key::new_binding(vec![
            key::with_keys(&["s"]),
            key::with_help("s", "toggle spinner"),
        ]),
        toggle_title_bar: key::new_binding(vec![
            key::with_keys(&["T"]),
            key::with_help("T", "toggle title"),
        ]),
        toggle_status_bar: key::new_binding(vec![
            key::with_keys(&["S"]),
            key::with_help("S", "toggle status"),
        ]),
        toggle_pagination: key::new_binding(vec![
            key::with_keys(&["P"]),
            key::with_help("P", "toggle pagination"),
        ]),
        toggle_help_menu: key::new_binding(vec![
            key::with_keys(&["H"]),
            key::with_help("H", "toggle help"),
        ]),
    }
}

/// RandomItemGenerator produces grocery items with random titles and
/// descriptions, mirroring the upstream `randomItemGenerator`. The titles and
/// descriptions are shuffled once at construction.
struct RandomItemGenerator {
    titles: Vec<String>,
    descs: Vec<String>,
    title_index: usize,
    desc_index: usize,
}

impl RandomItemGenerator {
    fn new() -> Self {
        let mut r = RandomItemGenerator {
            titles: vec![
                "Artichoke".to_string(),
                "Baking Flour".to_string(),
                "Bananas".to_string(),
                "Barley".to_string(),
                "Bean Sprouts".to_string(),
                "Bitter Melon".to_string(),
                "Black Cod".to_string(),
                "Blood Orange".to_string(),
                "Brown Sugar".to_string(),
                "Cashew Apple".to_string(),
                "Cashews".to_string(),
                "Cat Food".to_string(),
                "Coconut Milk".to_string(),
                "Cucumber".to_string(),
                "Curry Paste".to_string(),
                "Currywurst".to_string(),
                "Dill".to_string(),
                "Dragonfruit".to_string(),
                "Dried Shrimp".to_string(),
                "Eggs".to_string(),
                "Fish Cake".to_string(),
                "Furikake".to_string(),
                "Garlic".to_string(),
                "Gherkin".to_string(),
                "Ginger".to_string(),
                "Granulated Sugar".to_string(),
                "Grapefruit".to_string(),
                "Green Onion".to_string(),
                "Hazelnuts".to_string(),
                "Heavy whipping cream".to_string(),
                "Honey Dew".to_string(),
                "Horseradish".to_string(),
                "Jicama".to_string(),
                "Kohlrabi".to_string(),
                "Leeks".to_string(),
                "Lentils".to_string(),
                "Licorice Root".to_string(),
                "Meyer Lemons".to_string(),
                "Milk".to_string(),
                "Molasses".to_string(),
                "Muesli".to_string(),
                "Nectarine".to_string(),
                "Niagamo Root".to_string(),
                "Nopal".to_string(),
                "Nutella".to_string(),
                "Oat Milk".to_string(),
                "Oatmeal".to_string(),
                "Olives".to_string(),
                "Papaya".to_string(),
                "Party Gherkin".to_string(),
                "Peppers".to_string(),
                "Persian Lemons".to_string(),
                "Pickle".to_string(),
                "Pineapple".to_string(),
                "Plantains".to_string(),
                "Pocky".to_string(),
                "Powdered Sugar".to_string(),
                "Quince".to_string(),
                "Radish".to_string(),
                "Ramps".to_string(),
                "Star Anise".to_string(),
                "Sweet Potato".to_string(),
                "Tamarind".to_string(),
                "Unsalted Butter".to_string(),
                "Watermelon".to_string(),
                "Weißwurst".to_string(),
                "Yams".to_string(),
                "Yeast".to_string(),
                "Yuzu".to_string(),
                "Snow Peas".to_string(),
            ],
            descs: vec![
                "A little weird".to_string(),
                "Bold flavor".to_string(),
                "Can’t get enough".to_string(),
                "Delectable".to_string(),
                "Expensive".to_string(),
                "Expired".to_string(),
                "Exquisite".to_string(),
                "Fresh".to_string(),
                "Gimme".to_string(),
                "In season".to_string(),
                "Kind of spicy".to_string(),
                "Looks fresh".to_string(),
                "Looks good to me".to_string(),
                "Maybe not".to_string(),
                "My favorite".to_string(),
                "Oh my".to_string(),
                "On sale".to_string(),
                "Organic".to_string(),
                "Questionable".to_string(),
                "Really fresh".to_string(),
                "Refreshing".to_string(),
                "Salty".to_string(),
                "Scrumptious".to_string(),
                "Delectable".to_string(),
                "Slightly sweet".to_string(),
                "Smells great".to_string(),
                "Tasty".to_string(),
                "Too ripe".to_string(),
                "At last".to_string(),
                "What?".to_string(),
                "Wow".to_string(),
                "Yum".to_string(),
                "Maybe".to_string(),
                "Sure, why not?".to_string(),
            ],
            title_index: 0,
            desc_index: 0,
        };

        // Shuffle both lists once, mirroring the upstream `sync.Once` block.
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);
        shuffle(&mut r.titles, seed);
        shuffle(&mut r.descs, seed ^ 0xD1B54A32D192ED03);

        r
    }

    fn next(&mut self) -> Item {
        let i = Item {
            title: self.titles[self.title_index].clone(),
            description: self.descs[self.desc_index].clone(),
        };

        self.title_index += 1;
        if self.title_index >= self.titles.len() {
            self.title_index = 0;
        }

        self.desc_index += 1;
        if self.desc_index >= self.descs.len() {
            self.desc_index = 0;
        }

        i
    }
}

/// Shuffle permutes a slice in place using a xorshift64 PRNG, mirroring the
/// upstream `rand.Shuffle`.
fn shuffle(x: &mut [String], seed: u64) {
    let mut state = seed;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for i in (1..x.len()).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        x.swap(i, j);
    }
}

struct Model {
    styles: Styles,
    dark_bg: bool,
    width: usize,
    height: usize,
    list: list::Model,
    item_generator: RandomItemGenerator,
    keys: ListKeyMap,
    delegate_keys: DelegateKeyMap,
}

impl Model {
    fn initial_model() -> Self {
        let mut m = Model {
            styles: Styles::new(false), // default to dark background styles
            dark_bg: false,
            width: 0,
            height: 0,
            list: list::new(vec![], Box::new(list::new_default_delegate()), 0, 0),
            item_generator: RandomItemGenerator::new(),
            keys: new_list_key_map(),
            delegate_keys: new_delegate_key_map(),
        };

        // Make initial list of items.
        const NUM_ITEMS: usize = 24;
        let items: Vec<Box<dyn list::Item + Send + Sync>> = (0..NUM_ITEMS)
            .map(|_| Box::new(m.item_generator.next()) as Box<dyn list::Item + Send + Sync>)
            .collect();

        // Setup list. The "choose" and "remove" keybindings are handled by
        // the model itself (see `update`), mirroring the upstream
        // `newItemDelegate` update function; the delegate still contributes
        // its keybindings to the short and full help views.
        let mut delegate = list::new_default_delegate();
        let dk = m.delegate_keys.clone();
        delegate.short_help_func =
            Some(Box::new(move || vec![dk.choose.clone(), dk.remove.clone()]));
        let dk = m.delegate_keys.clone();
        delegate.full_help_func = Some(Box::new(move || {
            vec![vec![dk.choose.clone(), dk.remove.clone()]]
        }));

        m.list = list::new(items, Box::new(delegate), 0, 0);
        m.list.title = "Groceries".to_string();
        m.list.styles.title = m.styles.title.clone();

        // Additional full help entries for the list-level keybindings.
        let lk = m.keys.clone();
        m.list.additional_full_help_keys = Some(Box::new(move || {
            vec![
                lk.toggle_spinner.clone(),
                lk.insert_item.clone(),
                lk.toggle_title_bar.clone(),
                lk.toggle_status_bar.clone(),
                lk.toggle_pagination.clone(),
                lk.toggle_help_menu.clone(),
            ]
        }));

        m
    }

    /// UpdateListProperties updates the list size and the model and list
    /// styles.
    fn update_list_properties(&mut self) {
        // Update list size.
        let (h, v) = self.styles.app.get_frame_size();
        self.list.set_size(self.width - h, self.height - v);

        // Update the model and list styles.
        self.styles = Styles::new(self.dark_bg);
        self.list.styles.title = self.styles.title.clone();
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        request_background_color()
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(bg) = msg.as_any().downcast_ref::<BackgroundColorMsg>() {
            self.dark_bg = bg.is_dark();
            self.update_list_properties();
            return None;
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
            self.update_list_properties();
            return None;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            // Don't match any of the keys below if we're actively filtering.
            if self.list.filter_state() == FilterState::Filtering {
                return self.list.update(msg);
            }

            let kstr = &k.0;

            // Delegate keybindings: choose and remove. These mirror the
            // upstream delegate's update function, which requires mutable
            // access to the list model.
            if key::matches(kstr, std::slice::from_ref(&self.delegate_keys.choose)) {
                let title = self
                    .list
                    .selected_item()
                    .and_then(|i| i.as_any().downcast_ref::<Item>())
                    .map(|i| i.title.clone())
                    .unwrap_or_default();
                return self.list.new_status_message(
                    &self
                        .styles
                        .status_message
                        .render(&format!("You chose {}", title)),
                );
            }
            if key::matches(kstr, std::slice::from_ref(&self.delegate_keys.remove)) {
                let title = self
                    .list
                    .selected_item()
                    .and_then(|i| i.as_any().downcast_ref::<Item>())
                    .map(|i| i.title.clone())
                    .unwrap_or_default();
                let index = self.list.index();
                self.list.remove_item(index);
                if self.list.items().is_empty() {
                    self.delegate_keys.remove.set_enabled(false);
                }
                return self.list.new_status_message(
                    &self
                        .styles
                        .status_message
                        .render(&format!("Deleted {}", title)),
                );
            }

            if key::matches(kstr, std::slice::from_ref(&self.keys.toggle_spinner)) {
                return self.list.toggle_spinner();
            }

            if key::matches(kstr, std::slice::from_ref(&self.keys.toggle_title_bar)) {
                let v = !self.list.show_title();
                self.list.set_show_title(v);
                self.list.set_show_filter(v);
                self.list.set_filtering_enabled(v);
                return None;
            }

            if key::matches(kstr, std::slice::from_ref(&self.keys.toggle_status_bar)) {
                let v = !self.list.show_status_bar();
                self.list.set_show_status_bar(v);
                return None;
            }

            if key::matches(kstr, std::slice::from_ref(&self.keys.toggle_pagination)) {
                let v = !self.list.show_pagination();
                self.list.set_show_pagination(v);
                return None;
            }

            if key::matches(kstr, std::slice::from_ref(&self.keys.toggle_help_menu)) {
                let v = !self.list.show_help();
                self.list.set_show_help(v);
                return None;
            }

            if key::matches(kstr, std::slice::from_ref(&self.keys.insert_item)) {
                self.delegate_keys.remove.set_enabled(true);
                let new_item = self.item_generator.next();
                let ins_cmd = self.list.insert_item(0, Box::new(new_item.clone()));
                let status_cmd = self.list.new_status_message(
                    &self
                        .styles
                        .status_message
                        .render(&format!("Added {}", new_item.title)),
                );
                return batch(vec![ins_cmd, status_cmd]);
            }
        }

        // This will also call our delegate's update function.
        self.list.update(msg)
    }

    fn view(&self) -> View {
        let mut v = View::new(&self.styles.app.render(&self.list.view()));
        v.alt_screen = true;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::initial_model());
    p.run()?;
    Ok(())
}
