//! Cleanroom Rust port of upstream Go example: `examples/table/main.go`
//! Upstream Target Tag / Version: `v2.0.8`
//!
//! A table of the world's largest cities. Press enter to "go to" the selected
//! city, esc to toggle focus on the table, and q/ctrl+c to quit.

use charming_bubbles::table;
use charming_bubbletea::model::Model as ModelTrait;
use charming_bubbletea::{batch, print_f, quit, Cmd, KeyPressMsg, Msg, Program, View};
use charming_lipgloss::{new_style, Border, Color, Style};

fn base_style() -> Style {
    new_style()
        .border_style(Border::normal())
        .border_foreground(&["240"])
}

fn r(cells: [&str; 4]) -> table::Row {
    cells.iter().map(|s| s.to_string()).collect()
}

struct Model {
    table: table::Model,
}

impl ModelTrait for Model {
    fn init(&self) -> Cmd {
        None
    }

    fn update(&mut self, msg: &dyn Msg) -> Cmd {
        if let Some(k) = msg.as_any().downcast_ref::<KeyPressMsg>() {
            match k.0.to_string().as_str() {
                "esc" => {
                    if self.table.focused() {
                        self.table.blur();
                    } else {
                        self.table.focus();
                    }
                }
                "q" | "ctrl+c" => return quit(),
                "enter" => {
                    if let Some(row) = self.table.selected_row() {
                        return batch(vec![print_f(format_args!("Let's go to {}!", row[1]))]);
                    }
                }
                _ => {}
            }
        }

        self.table.update(msg)
    }

    fn view(&self) -> View {
        let s = format!(
            "{}\n  {}\n",
            base_style().render(&self.table.view()),
            self.table.help_view()
        );
        View::new(&s)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let columns = [
        table::Column {
            title: "Rank".to_string(),
            width: 4,
        },
        table::Column {
            title: "City".to_string(),
            width: 10,
        },
        table::Column {
            title: "Country".to_string(),
            width: 10,
        },
        table::Column {
            title: "Population".to_string(),
            width: 10,
        },
    ];

    let rows: Vec<table::Row> = vec![
        r(["1", "Tokyo", "Japan", "37,274,000"]),
        r(["2", "Delhi", "India", "32,065,760"]),
        r(["3", "Shanghai", "China", "28,516,904"]),
        r(["4", "Dhaka", "Bangladesh", "22,478,116"]),
        r(["5", "São Paulo", "Brazil", "22,429,800"]),
        r(["6", "Mexico City", "Mexico", "22,085,140"]),
        r(["7", "Cairo", "Egypt", "21,750,020"]),
        r(["8", "Beijing", "China", "21,333,332"]),
        r(["9", "Mumbai", "India", "20,961,472"]),
        r(["10", "Osaka", "Japan", "19,059,856"]),
        r(["11", "Chongqing", "China", "16,874,740"]),
        r(["12", "Karachi", "Pakistan", "16,839,950"]),
        r(["13", "Istanbul", "Turkey", "15,636,243"]),
        r(["14", "Kinshasa", "DR Congo", "15,628,085"]),
        r(["15", "Lagos", "Nigeria", "15,387,639"]),
        r(["16", "Buenos Aires", "Argentina", "15,369,919"]),
        r(["17", "Kolkata", "India", "15,133,888"]),
        r(["18", "Manila", "Philippines", "14,406,059"]),
        r(["19", "Tianjin", "China", "14,011,828"]),
        r(["20", "Guangzhou", "China", "13,964,637"]),
        r(["21", "Rio De Janeiro", "Brazil", "13,634,274"]),
        r(["22", "Lahore", "Pakistan", "13,541,764"]),
        r(["23", "Bangalore", "India", "13,193,035"]),
        r(["24", "Shenzhen", "China", "12,831,330"]),
        r(["25", "Moscow", "Russia", "12,640,818"]),
        r(["26", "Chennai", "India", "11,503,293"]),
        r(["27", "Bogota", "Colombia", "11,344,312"]),
        r(["28", "Paris", "France", "11,142,303"]),
        r(["29", "Jakarta", "Indonesia", "11,074,811"]),
        r(["30", "Lima", "Peru", "11,044,607"]),
        r(["31", "Bangkok", "Thailand", "10,899,698"]),
        r(["32", "Hyderabad", "India", "10,534,418"]),
        r(["33", "Seoul", "South Korea", "9,975,709"]),
        r(["34", "Nagoya", "Japan", "9,571,596"]),
        r(["35", "London", "United Kingdom", "9,540,576"]),
        r(["36", "Chengdu", "China", "9,478,521"]),
        r(["37", "Nanjing", "China", "9,429,381"]),
        r(["38", "Tehran", "Iran", "9,381,546"]),
        r(["39", "Ho Chi Minh City", "Vietnam", "9,077,158"]),
        r(["40", "Luanda", "Angola", "8,952,496"]),
        r(["41", "Wuhan", "China", "8,591,611"]),
        r(["42", "Xi An Shaanxi", "China", "8,537,646"]),
        r(["43", "Ahmedabad", "India", "8,450,228"]),
        r(["44", "Kuala Lumpur", "Malaysia", "8,419,566"]),
        r(["45", "New York City", "United States", "8,177,020"]),
        r(["46", "Hangzhou", "China", "8,044,878"]),
        r(["47", "Surat", "India", "7,784,276"]),
        r(["48", "Suzhou", "China", "7,764,499"]),
        r(["49", "Hong Kong", "Hong Kong", "7,643,256"]),
        r(["50", "Riyadh", "Saudi Arabia", "7,538,200"]),
        r(["51", "Shenyang", "China", "7,527,975"]),
        r(["52", "Baghdad", "Iraq", "7,511,920"]),
        r(["53", "Dongguan", "China", "7,511,851"]),
        r(["54", "Foshan", "China", "7,497,263"]),
        r(["55", "Dar Es Salaam", "Tanzania", "7,404,689"]),
        r(["56", "Pune", "India", "6,987,077"]),
        r(["57", "Santiago", "Chile", "6,856,939"]),
        r(["58", "Madrid", "Spain", "6,713,557"]),
        r(["59", "Haerbin", "China", "6,665,951"]),
        r(["60", "Toronto", "Canada", "6,312,974"]),
        r(["61", "Belo Horizonte", "Brazil", "6,194,292"]),
        r(["62", "Khartoum", "Sudan", "6,160,327"]),
        r(["63", "Johannesburg", "South Africa", "6,065,354"]),
        r(["64", "Singapore", "Singapore", "6,039,577"]),
        r(["65", "Dalian", "China", "5,930,140"]),
        r(["66", "Qingdao", "China", "5,865,232"]),
        r(["67", "Zhengzhou", "China", "5,690,312"]),
        r(["68", "Ji Nan Shandong", "China", "5,663,015"]),
        r(["69", "Barcelona", "Spain", "5,658,472"]),
        r(["70", "Saint Petersburg", "Russia", "5,535,556"]),
        r(["71", "Abidjan", "Ivory Coast", "5,515,790"]),
        r(["72", "Yangon", "Myanmar", "5,514,454"]),
        r(["73", "Fukuoka", "Japan", "5,502,591"]),
        r(["74", "Alexandria", "Egypt", "5,483,605"]),
        r(["75", "Guadalajara", "Mexico", "5,339,583"]),
        r(["76", "Ankara", "Turkey", "5,309,690"]),
        r(["77", "Chittagong", "Bangladesh", "5,252,842"]),
        r(["78", "Addis Ababa", "Ethiopia", "5,227,794"]),
        r(["79", "Melbourne", "Australia", "5,150,766"]),
        r(["80", "Nairobi", "Kenya", "5,118,844"]),
        r(["81", "Hanoi", "Vietnam", "5,067,352"]),
        r(["82", "Sydney", "Australia", "5,056,571"]),
        r(["83", "Monterrey", "Mexico", "5,036,535"]),
        r(["84", "Changsha", "China", "4,809,887"]),
        r(["85", "Brasilia", "Brazil", "4,803,877"]),
        r(["86", "Cape Town", "South Africa", "4,800,954"]),
        r(["87", "Jiddah", "Saudi Arabia", "4,780,740"]),
        r(["88", "Urumqi", "China", "4,710,203"]),
        r(["89", "Kunming", "China", "4,657,381"]),
        r(["90", "Changchun", "China", "4,616,002"]),
        r(["91", "Hefei", "China", "4,496,456"]),
        r(["92", "Shantou", "China", "4,490,411"]),
        r(["93", "Xinbei", "Taiwan", "4,470,672"]),
        r(["94", "Kabul", "Afghanistan", "4,457,882"]),
        r(["95", "Ningbo", "China", "4,405,292"]),
        r(["96", "Tel Aviv", "Israel", "4,343,584"]),
        r(["97", "Yaounde", "Cameroon", "4,336,670"]),
        r(["98", "Rome", "Italy", "4,297,877"]),
        r(["99", "Shijiazhuang", "China", "4,285,135"]),
        r(["100", "Montreal", "Canada", "4,276,526"]),
    ];

    let mut t = table::new(vec![
        table::with_columns(&columns),
        table::with_rows(&rows),
        table::with_focused(true),
        table::with_height(7),
        table::with_width(42),
    ]);

    let mut s = table::default_styles();
    s.header = s
        .header
        .border_style(Border::normal())
        .border_foreground(&["240"])
        .border(Border::normal(), &[false, false, true, false])
        .bold(false);
    s.selected = s
        .selected
        .foreground_color(Color::parse("229"))
        .background_color(Color::parse("57"))
        .bold(false);
    t.set_styles(s);

    let p = Program::new(Model { table: t });
    p.run()?;
    Ok(())
}
