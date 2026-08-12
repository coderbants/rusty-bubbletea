//! Cleanroom Rust port of upstream Go example: `examples/list-default/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A list with the default delegate, listing the author's favorite things.

use std::any::Any;

use charming_bubbles::list;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::new_style;

/// DocStyle wraps the whole list with a margin, mirroring the upstream
/// `docStyle` global.
fn doc_style() -> charming_lipgloss::Style {
    new_style().margin(&[1, 2])
}

#[derive(Debug, Clone)]
struct Item {
    title: String,
    desc: String,
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
        self.desc.clone()
    }
}

struct Model {
    list: list::Model,
}

impl Model {
    fn new() -> Self {
        let items: Vec<Box<dyn list::Item + Send + Sync>> = vec![
            Box::new(Item {
                title: "Raspberry Pi’s".to_string(),
                desc: "I have ’em all over my house".to_string(),
            }),
            Box::new(Item {
                title: "Nutella".to_string(),
                desc: "It's good on toast".to_string(),
            }),
            Box::new(Item {
                title: "Bitter melon".to_string(),
                desc: "It cools you down".to_string(),
            }),
            Box::new(Item {
                title: "Nice socks".to_string(),
                desc: "And by that I mean socks without holes".to_string(),
            }),
            Box::new(Item {
                title: "Eight hours of sleep".to_string(),
                desc: "I had this once".to_string(),
            }),
            Box::new(Item {
                title: "Cats".to_string(),
                desc: "Usually".to_string(),
            }),
            Box::new(Item {
                title: "Plantasia, the album".to_string(),
                desc: "My plants love it too".to_string(),
            }),
            Box::new(Item {
                title: "Pour over coffee".to_string(),
                desc: "It takes forever to make though".to_string(),
            }),
            Box::new(Item {
                title: "VR".to_string(),
                desc: "Virtual reality...what is there to say?".to_string(),
            }),
            Box::new(Item {
                title: "Noguchi Lamps".to_string(),
                desc: "Such pleasing organic forms".to_string(),
            }),
            Box::new(Item {
                title: "Linux".to_string(),
                desc: "Pretty much the best OS".to_string(),
            }),
            Box::new(Item {
                title: "Business school".to_string(),
                desc: "Just kidding".to_string(),
            }),
            Box::new(Item {
                title: "Pottery".to_string(),
                desc: "Wet clay is a great feeling".to_string(),
            }),
            Box::new(Item {
                title: "Shampoo".to_string(),
                desc: "Nothing like clean hair".to_string(),
            }),
            Box::new(Item {
                title: "Table tennis".to_string(),
                desc: "It’s surprisingly exhausting".to_string(),
            }),
            Box::new(Item {
                title: "Milk crates".to_string(),
                desc: "Great for packing in your extra stuff".to_string(),
            }),
            Box::new(Item {
                title: "Afternoon tea".to_string(),
                desc: "Especially the tea sandwich part".to_string(),
            }),
            Box::new(Item {
                title: "Stickers".to_string(),
                desc: "The thicker the vinyl the better".to_string(),
            }),
            Box::new(Item {
                title: "20° Weather".to_string(),
                desc: "Celsius, not Fahrenheit".to_string(),
            }),
            Box::new(Item {
                title: "Warm light".to_string(),
                desc: "Like around 2700 Kelvin".to_string(),
            }),
            Box::new(Item {
                title: "The vernal equinox".to_string(),
                desc: "The autumnal equinox is pretty good too".to_string(),
            }),
            Box::new(Item {
                title: "Gaffer’s tape".to_string(),
                desc: "Basically sticky fabric".to_string(),
            }),
            Box::new(Item {
                title: "Terrycloth".to_string(),
                desc: "In other words, towel fabric".to_string(),
            }),
        ];

        let mut m = Model {
            list: list::new(items, Box::new(list::new_default_delegate()), 0, 0),
        };
        m.list.title = "My Fave Things".to_string();
        m
    }
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            if k.0.to_string() == "ctrl+c" {
                return quit();
            }
        }

        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            let (h, v) = doc_style().get_frame_size();
            self.list.set_size(ws.width - h, ws.height - v);
            return None;
        }

        self.list.update(msg)
    }

    fn view(&self) -> View {
        let mut v = View::new(&doc_style().render(&self.list.view()));
        v.alt_screen = true;
        v
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model::new());
    p.run()?;
    Ok(())
}
