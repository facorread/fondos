// fondos: Visualizes the evolution of investment funds at Banco Davivienda, Colombia, South America

// Copyright (C) 2020 Fabio A. Correa Duran facorread@gmail.com

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};

enum Mode {
    Header,
    Table,
    Footer,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
/// Represents a record of the money balance in a fund.
struct Balance {
    date: chrono::NaiveDate,
    /// Balance in the fund, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    balance: i64,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
/// Represents a record of an action in a fund.
struct Action {
    date: chrono::NaiveDate,
    /// Amount of the action, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed.
    change: i64,
}

#[derive(Debug)]
struct Repetition {
    fund_index: usize,
    action_index: usize,
    // Let repetition as i32 for subtraction
    repetition: i32,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
struct Series {
    fund: String,
    balance: Vec<Balance>,
    action: Vec<Action>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
struct Table {
    // List of time series, in the same order than self.fund
    table: Vec<Series>,
}

fn calculate_hash(t: &Table) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

fn consume_input() {
    println!("Enter EOF:");
    loop {
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() && input == "EOF\n" {
            return;
        }
    }
}

fn main() {
    use plotters::prelude::*;
    use std::fs;
    // use std::io::Write as IoWrite; // See https://doc.rust-lang.org/std/macro.writeln.html
    let date = chrono::Local::today().naive_local();
    let funds_file_name = "funds.dat";
    let r_err = &*format!("Error reading the file {}", funds_file_name);
    let w_err = &*format!("Error writing to file {}", funds_file_name);
    let db_path = std::path::Path::new(&funds_file_name);
    let mut table: Table = if db_path.exists() {
        let db_file = fs::File::open(db_path).expect(r_err);
        bincode::deserialize_from(db_file).expect(r_err)
    } else {
        println!("Starting a new file. Ctrl + C if this is a mistake.");
        Table {
            table: Vec::<_>::with_capacity(10),
        }
    };
    let original_hash = calculate_hash(&table);
    // dbg![table.clone()];
    {
        println!("Paste the account status here. You can enter EOF if you have no data, or Ctrl + C to close this program:");
        let mut mode = Mode::Header;
        let mut errors = String::new();
        let mut errors_produced = false;
        loop {
            use std::io;
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_number_of_bytes_read) => match mode {
                    Mode::Header => {
                        if input == "Anual**\n" {
                            mode = Mode::Table;
                        } else if input == "EOF\n" {
                            break;
                        }
                    }
                    Mode::Table => {
                        if input.starts_with("Total	") {
                            mode = Mode::Footer;
                        } else if input == "EOF\n" {
                            break;
                        } else {
                            let mut input_iter = input.split('\t'); // Do not use split_whitespace because funds and actions have spaces
                            let fund_str = input_iter.next().unwrap();
                            let balance_raw = input_iter.next().unwrap();
                            let balance_str = balance_raw.replace(&['$', ','][..], "");
                            match balance_str.parse::<f64>() {
                                Ok(balance_f) => {
                                    let balance = (balance_f * 100.0) as i64;
                                    match table.table.iter_mut().find(|s| s.fund == fund_str) {
                                        Some(series) => {
                                            match series
                                                .balance
                                                .iter_mut()
                                                .find(|b: &&mut Balance| b.date == date)
                                            {
                                                Some(x) => {
                                                    if x.balance != balance {
                                                        errors = format!("{}Warning: Fund {} changing balance from {} to {}\n", errors, fund_str, x.balance, balance);
                                                        x.balance = balance;
                                                    }
                                                }
                                                None => {
                                                    series.balance.push(Balance { date, balance })
                                                }
                                            }
                                        }
                                        None => {
                                            table.table.push(Series {
                                                fund: String::from(fund_str),
                                                balance: vec![Balance { date, balance }],
                                                action: Vec::<_>::with_capacity(10),
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    errors = format!(
                                        "{}Failed parsing line {}\nfund = {}, balance_raw = {}, balance_str = {}: {}", errors, input,
                                        fund_str, balance_raw, balance_str, e
                                    );
                                    errors_produced = true;
                                }
                            };
                        }
                    }
                    Mode::Footer => {
                        if (input
                            == "Aprenda aquí sobre el producto	Aprenda aquí sobre el producto\n")
                            || (input == "EOF\n")
                        {
                            break;
                        }
                    }
                },
                Err(error) => println!("Error parsing data: {}", error),
            }
        }
        if !errors.is_empty() {
            consume_input();
            print!("{}", errors);
        }
        if errors_produced {
            return;
        }
    }
    // The page "Recomposición de su inversión en su Dafuturo" should not be used by this program because movements between
    // funds take several days to complete. Instead, use fund actions from the "Últimos Movimientos" pages.
    'fund_changes: loop {
        println!("Paste the 'Ultimos Movimientos' page here. Enter EOF when you are done with the page, or Ctrl + C to close this program:");
        let mut mode = Mode::Header;
        let mut errors = String::new();
        let mut errors_produced = false;
        let mut repetitions = Vec::<Repetition>::with_capacity(10);
        loop {
            use std::io;
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_number_of_bytes_read) => {
                    match mode {
                        Mode::Header => {
                            if input.starts_with(
                                "Fecha	Nombre del multiportafolio	Movimiento	Tipo Aporte	Valor",
                            ) {
                                mode = Mode::Table;
                            } else if input == "EOF\n" {
                                break 'fund_changes;
                            }
                        }
                        Mode::Table => {
                            if input == "\n" {
                                mode = Mode::Footer;
                            } else if input == "EOF\n" {
                                break 'fund_changes;
                            } else {
                                let mut input_iter = input.split("\t"); // Do not use split_whitespace because funds and actions have spaces
                                let date = chrono::NaiveDate::parse_from_str(
                                    input_iter.next().unwrap(),
                                    "%d/%m/%Y",
                                )
                                .unwrap();
                                let fund_str = input_iter.next().unwrap();
                                let action_str = input_iter.next().unwrap();
                                let _type_str = input_iter.next().unwrap();
                                let change_raw = input_iter.next().unwrap();
                                let change_str = change_raw.replace(&['$', ',', '\n'][..], "");
                                let change_f = match change_str.parse::<f64>() {
                                    Ok(c) => match action_str {
                                        "Aporte" | "Aporte por traslado de otro portafolio" => c,
                                        "Aporte por traslado a otro portafolio" => -c,
                                        _ => {
                                            errors = format!("{}Error: Action '{}' not recognized. Please review the code.\n", errors, action_str);
                                            errors_produced = true;
                                            0.
                                        }
                                    },
                                    Err(e) => {
                                        errors = format!("{}Error in line {}: Could not parse '{}' as a number for action {}: {}.\n", errors, input, change_str, action_str, e);
                                        errors_produced = true;
                                        0.
                                    }
                                };
                                let change = (change_f * 100.0) as i64;
                                match table.table.iter().position(|s| s.fund == fund_str) {
                                    Some(fund_index) => {
                                        match table.table[fund_index].action.iter().position(
                                            |a: &Action| a.date == date && a.change == change,
                                        ) {
                                            Some(action_index) => {
                                                // This can happen frequently. See if this has been counted before.
                                                match repetitions.iter_mut().find(
                                                    |r: &&mut Repetition| {
                                                        r.fund_index == fund_index
                                                            && r.action_index == action_index
                                                    },
                                                ) {
                                                    Some(r) => r.repetition += 1,
                                                    None => repetitions.push(Repetition {
                                                        // This push is valid for both completely new actions as well as actions that are already repeated
                                                        fund_index,
                                                        action_index,
                                                        repetition: 1,
                                                    }),
                                                };
                                            }
                                            None => table.table[fund_index]
                                                .action
                                                .push(Action { date, change }),
                                        }
                                    }
                                    None => {
                                        table.table.push(Series {
                                            fund: String::from(fund_str),
                                            balance: vec![],
                                            action: vec![Action { date, change }],
                                        });
                                    }
                                };
                            }
                        }
                        Mode::Footer => {
                            if input == "que origina esta transacción.\n" {
                                break;
                            } else if input == "EOF\n" {
                                break 'fund_changes;
                            }
                        }
                    }
                }
                Err(error) => println!("Error parsing data: {}", error),
            }
        }
        for r in repetitions {
            let action = &mut table.table[r.fund_index].action;
            let a = action[r.action_index].clone();
            let reps = action
                .iter()
                .filter(|b| a.date == b.date && a.change == b.change)
                .count() as i32
                - r.repetition;
            for _repetition in 0..reps {
                action.push(a.clone());
            }
        }
        // dbg![table.clone()];
        if !errors.is_empty() {
            consume_input();
            print!("{}", errors);
        }
        if errors_produced {
            return;
        }
    }
    let table = table; // Read-only
    if calculate_hash(&table) == original_hash {
        println!("Data remains the same. Files remain unchanged.");
    } else {
        println!("Creating new funds file...");
        let new_file_name = "funds.new";
        let new_path = std::path::Path::new(&new_file_name);
        {
            let new_err = &*format!("Error writing to temporary file {}", funds_file_name);
            let new_file = fs::File::create(new_path).expect(new_err);
            bincode::serialize_into(new_file, &table).expect(new_err);
        }
        if db_path.exists() {
            let backup_file_name = format!(
                "funds_backup{}.dat",
                chrono::Local::now().format("%Y%m%dT%H%M%S")
            );
            let to = std::path::Path::new(&backup_file_name);
            fs::rename(db_path, to).expect("Creating backup file");
        }
        fs::rename(new_path, db_path).expect(w_err);
    }
    {
        // Delete any png files from previous runs.
        for dir in &["."] {
            for res in std::fs::read_dir(dir).unwrap() {
                if let Ok(entry) = res {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "png" {
                            if let Some(file_name_os_str) = path.file_name() {
                                if let Some(file_name) = file_name_os_str.to_str() {
                                    if let Err(e) = fs::remove_file(path.clone()) {
                                        panic!(
                                            "Could not remove file {} from a previous run: {}",
                                            file_name, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    {
        let background_color = &BLACK;
        let _background_fill = background_color.filled();
        let _transparent_color = background_color.mix(0.);
        let color0 = &WHITE;
        let color01 = color0.mix(0.1);
        let color02 = color0.mix(0.2);
        let color1 = &plotters::style::RGBColor(255, 192, 0);
        let color2 = &plotters::style::RGBColor(0, 176, 80);
        let color3 = &plotters::style::RGBColor(32, 56, 100);
        let color4 = &plotters::style::RGBColor(255, 231, 146);
        let color5 = &plotters::style::RGBColor(157, 85, 15);
        let color6 = &plotters::style::RGBColor(196, 53, 53);
        let color7 = &plotters::style::RGBColor(158, 138, 227);
        let color8 = &plotters::style::RGBColor(134, 202, 217);
        let color9 = &plotters::style::RGBColor(0, 199, 196);
        let _color_vec = vec![
            color0, color1, color2, color3, color4, color5, color6, color7, color8, color9,
        ];
        let fill0 = color0.filled();
        let _fill01 = color01.filled();
        let _fill02 = color02.filled();
        let fill1 = color1.filled();
        let fill2 = color2.filled();
        let fill3 = color3.filled();
        let fill4 = color4.filled();
        let fill5 = color5.filled();
        let fill6 = color6.filled();
        let fill7 = color7.filled();
        let fill8 = color8.filled();
        let fill9 = color9.filled();
        let _fill_vec = vec![
            fill0, fill1, fill2, fill3, fill4, fill5, fill6, fill7, fill8, fill9,
        ];
        let _x_label_area_size = 70;
        let _y_label_area_size = 90;
        let text_size0 = 60;
        let text_size1 = 44;
        let text_size2 = 34;
        let _background_text = ("Calibri", 1).into_font().color(background_color);
        let _text0 = ("Calibri", text_size0).into_font().color(color0);
        let _text1 = ("Calibri", text_size1).into_font().color(color0);
        let text2 = ("Calibri", text_size2).into_font().color(color0);
        use plotters::style::text_anchor::{HPos, Pos, VPos};
        let _text2c = text2.pos(Pos::new(HPos::Center, VPos::Top));
        let figure_file_name = "fondos00.png";
        let figure_path = std::path::Path::new(&figure_file_name);
        if figure_path.exists() {
            panic!(
                "This program just tried to rewrite {}; please debug",
                figure_path.to_str().unwrap()
            );
        }
        let drawing_area = BitMapBackend::new(figure_path, (1920, 1080)).into_drawing_area();
        drawing_area.fill(background_color).unwrap();
    }
    println!("Figures are ready.");
}
