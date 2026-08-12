//! Cleanroom Rust port of upstream Go example: `examples/table-resize/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A table that resizes with the window, using the lipgloss table component.

use charming_bubbletea::key::KeyPressMsg;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::screen::WindowSizeMsg;
use charming_bubbletea::{quit, Cmd, Msg, Program, View};
use charming_lipgloss::table;
use charming_lipgloss::Border;

// Pokemon types.
const NONE: &str = "";
const BUG: &str = "Bug";
const ELECTRIC: &str = "Electric";
const FIRE: &str = "Fire";
const FLYING: &str = "Flying";
const GRASS: &str = "Grass";
const GROUND: &str = "Ground";
const NORMAL: &str = "Normal";
const POISON: &str = "Poison";
const WATER: &str = "Water";

struct Model {
    width: usize,
    height: usize,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(ws) = msg.as_any().downcast_ref::<WindowSizeMsg>() {
            self.width = ws.width;
            self.height = ws.height;
        }

        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "q" | "ctrl+c" => return quit(),
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> View {
        let mut t = build_table().width(self.width).height(self.height);
        let mut v = View::new(&format!("\n{}\n", t.string()));
        v.alt_screen = true;
        v
    }
}

fn build_table() -> table::Table {
    let base_style = charming_lipgloss::new_style().padding(&[0, 1]);
    let header_style = base_style.clone().foreground("252").bold(true);
    let selected_style = base_style
        .clone()
        .foreground("#01BE85")
        .background("#00432F");
    let type_colors: Vec<(String, String)> = vec![
        (BUG.to_string(), "#D7FF87".to_string()),
        (ELECTRIC.to_string(), "#FDFF90".to_string()),
        (FIRE.to_string(), "#FF7698".to_string()),
        (FLYING.to_string(), "#FF87D7".to_string()),
        (GRASS.to_string(), "#75FBAB".to_string()),
        (GROUND.to_string(), "#FF875F".to_string()),
        (NORMAL.to_string(), "#929292".to_string()),
        (POISON.to_string(), "#7D5AFC".to_string()),
        (WATER.to_string(), "#00E2C7".to_string()),
    ];
    let dim_type_colors: Vec<(String, String)> = vec![
        (BUG.to_string(), "#97AD64".to_string()),
        (ELECTRIC.to_string(), "#FCFF5F".to_string()),
        (FIRE.to_string(), "#BA5F75".to_string()),
        (FLYING.to_string(), "#C97AB2".to_string()),
        (GRASS.to_string(), "#59B980".to_string()),
        (GROUND.to_string(), "#C77252".to_string()),
        (NORMAL.to_string(), "#727272".to_string()),
        (POISON.to_string(), "#634BD0".to_string()),
        (WATER.to_string(), "#439F8E".to_string()),
    ];

    let headers = ["#", "NAME", "TYPE 1", "TYPE 2", "JAPANESE", "OFFICIAL ROM."];
    let rows: Vec<Vec<&str>> = vec![
        vec!["1", "Bulbasaur", GRASS, POISON, "フシギダネ", "Bulbasaur"],
        vec!["2", "Ivysaur", GRASS, POISON, "フシギソウ", "Ivysaur"],
        vec!["3", "Venusaur", GRASS, POISON, "フシギバナ", "Venusaur"],
        vec!["4", "Charmander", FIRE, NONE, "ヒトカゲ", "Hitokage"],
        vec!["5", "Charmeleon", FIRE, NONE, "リザード", "Lizardo"],
        vec!["6", "Charizard", FIRE, FLYING, "リザードン", "Lizardon"],
        vec!["7", "Squirtle", WATER, NONE, "ゼニガメ", "Zenigame"],
        vec!["8", "Wartortle", WATER, NONE, "カメール", "Kameil"],
        vec!["9", "Blastoise", WATER, NONE, "カメックス", "Kamex"],
        vec!["10", "Caterpie", BUG, NONE, "キャタピー", "Caterpie"],
        vec!["11", "Metapod", BUG, NONE, "トランセル", "Trancell"],
        vec!["12", "Butterfree", BUG, FLYING, "バタフリー", "Butterfree"],
        vec!["13", "Weedle", BUG, POISON, "ビードル", "Beedle"],
        vec!["14", "Kakuna", BUG, POISON, "コクーン", "Cocoon"],
        vec!["15", "Beedrill", BUG, POISON, "スピアー", "Spear"],
        vec!["16", "Pidgey", NORMAL, FLYING, "ポッポ", "Poppo"],
        vec!["17", "Pidgeotto", NORMAL, FLYING, "ピジョン", "Pigeon"],
        vec!["18", "Pidgeot", NORMAL, FLYING, "ピジョット", "Pigeot"],
        vec!["19", "Rattata", NORMAL, NONE, "コラッタ", "Koratta"],
        vec!["20", "Raticate", NORMAL, NONE, "ラッタ", "Ratta"],
        vec!["21", "Spearow", NORMAL, FLYING, "オニスズメ", "Onisuzume"],
        vec!["22", "Fearow", NORMAL, FLYING, "オニドリル", "Onidrill"],
        vec!["23", "Ekans", POISON, NONE, "アーボ", "Arbo"],
        vec!["24", "Arbok", POISON, NONE, "アーボック", "Arbok"],
        vec!["25", "Pikachu", ELECTRIC, NONE, "ピカチュウ", "Pikachu"],
        vec!["26", "Raichu", ELECTRIC, NONE, "ライチュウ", "Raichu"],
        vec!["27", "Sandshrew", GROUND, NONE, "サンド", "Sand"],
        vec!["28", "Sandslash", GROUND, NONE, "サンドパン", "Sandpan"],
    ];

    let rows_refs: Vec<&[&str]> = rows.iter().map(|r| r.as_slice()).collect();
    let rows_slice: &[&[&str]] = &rows_refs;
    let type_colors_clone = type_colors.clone();
    let dim_type_colors_clone = dim_type_colors.clone();
    let base_style_clone = base_style.clone();

    let t = table::new()
        .headers(&headers)
        .rows(rows_slice)
        .border(Border::normal())
        .border_style(charming_lipgloss::new_style().foreground("238"))
        .style_func(Box::new(move |row: isize, col: usize| {
            if row == 0 {
                return header_style.clone();
            }

            let row_index = row - 1;
            if row_index < 0 || row_index as usize >= rows.len() {
                return base_style_clone.clone();
            }
            let row_index = row_index as usize;

            if rows[row_index][1] == "Pikachu" {
                return selected_style.clone();
            }

            let even = row % 2 == 0;

            match col {
                2 | 3 => {
                    // Type 1 + 2
                    let c = if even {
                        &dim_type_colors_clone
                    } else {
                        &type_colors_clone
                    };

                    if col >= rows[row_index].len() {
                        return base_style_clone.clone();
                    }

                    if let Some((_, color)) = c.iter().find(|(t, _)| *t == rows[row_index][col]) {
                        return base_style_clone.clone().foreground(color);
                    }
                    base_style_clone.clone()
                }
                _ => {
                    if even {
                        base_style_clone.clone().foreground("245")
                    } else {
                        base_style_clone.clone().foreground("252")
                    }
                }
            }
        }))
        .border(charming_lipgloss::border::thick_border());

    t
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Program::new(Model {
        width: 0,
        height: 0,
    });
    p.run()?;
    Ok(())
}
