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

enum Mode1 {
    Header,
    SkipSubHeader,
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
/// Represents a record of whole fund value and unit value.
struct FundValue {
    date: chrono::NaiveDate,
    /// Value of the whole fund, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    fund_value: i64,
    /// Value of a fund unit, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    unit_value: i64,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
/// Represents a record of whole fund value, unit value, and returns on equity.
struct FundValueNew {
    date: chrono::NaiveDate,
    /// Value of the whole fund, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    fund_value: i64,
    /// Value of a fund unit, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    unit_value: i64,
    /// Return on equity for next-to-last year, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_next_to_last_year: i64,
    /// Return on equity for last year, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_last_year: i64,
    /// Return on equity for year to date, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_year_to_date: i64,
    /// Return on equity for today, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_day: i64,
    /// Return on equity for today, annualized, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_day_annualized: i64,
    /// Return on equity for the last month, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_month: i64,
    /// Return on equity for the last trimester, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_trimester: i64,
    /// Return on equity for the last semester, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_semester: i64,
    /// Return on equity for the last year, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_year: i64,
    /// Return on equity for the last 2 years, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_2_years: i64,
    /// Return on equity from the beginning of the fund, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    roe_total: i64,
}

impl From<FundValue> for FundValueNew {
    fn from(rhs: FundValue) -> Self {
        Self {
            date: rhs.date,
            fund_value: rhs.fund_value,
            unit_value: rhs.unit_value,
            roe_next_to_last_year: 0,
            roe_last_year: 0,
            roe_year_to_date: 0,
            roe_day: 0,
            roe_day_annualized: 0,
            roe_month: 0,
            roe_trimester: 0,
            roe_semester: 0,
            roe_year: 0,
            roe_2_years: 0,
            roe_total: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
/// Represents a record of an action in a fund.
struct Action {
    date: chrono::NaiveDate,
    /// Amount of the action, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed.
    change: i64,
}

type Date = chrono::Date<chrono::Utc>;

/// Represents variation of a fund.
type Variation = (
    Date,
    // Variation in the fund, as a proportion of the previous record.
    f64,
);

#[derive(Debug)]
/// Represents a repetition of a fund action.
struct Repetition {
    fund_index: usize,
    action_index: usize,
    // Let repetition as i32 for subtraction
    repetition: i32,
}

#[derive(Clone, Debug)]
/// Represents a label in the figure.
struct Label<'label_lifetime> {
    /// Index of the fund; useful for coloring
    index: usize,
    /// Reference to the name of the fund
    fund: &'label_lifetime String,
    /// Variation over the full range of dates
    variation: f64,
    /// coordinate in the backend system (pixels)
    backend_coord: plotters_backend::BackendCoord,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
struct Series {
    fund: String,
    balance: Vec<Balance>,
    action: Vec<Action>,
    fund_value: Vec<FundValue>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
struct SeriesNew {
    fund: String,
    balance: Vec<Balance>,
    action: Vec<Action>,
    fund_value: Vec<FundValueNew>,
}

impl From<Series> for SeriesNew {
    fn from(rhs: Series) -> Self {
        Self {
            fund: rhs.fund,
            balance: rhs.balance,
            action: rhs.action,
            fund_value: rhs.fund_value.into_iter().map(|f| FundValueNew::from(f)).collect(),
        }
    }
}

#[derive(Clone, Debug)]
struct PlotSeries {
    fund: String,
    variation: Vec<Variation>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
struct Table {
    // List of time series, in the same order than self.fund
    table: Vec<Series>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
struct TableNew {
    // List of time series, in the same order than self.fund
    table: Vec<SeriesNew>,
}

impl From<Table> for TableNew {
    fn from(rhs: Table) -> Self {
        Self {
            table: rhs.table.into_iter().map(|s| SeriesNew::from(s)).collect(),
        }
    }
}

fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    use std::hash::Hasher;
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

fn columns(n_durations: usize) -> usize {
    n_durations / 2 + n_durations % 2
}

fn main() {
    use plotters::prelude::*;
    use std::fs;
    // Useful for debugging on vscode.
    let interactive_run = false;
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
    if interactive_run {
        println!("Paste the account status here.\nEnter EOF if you have no data, or Ctrl + C to close this program:");
        let mut mode = Mode::Header;
        let mut errors = String::new();
        let mut errors_produced = false;
        let mut input_err = |message: &str, input, is_error| {
            if is_error {
                errors = format!(
                    "{}{}\n{}", errors, message, input
                );
                errors_produced = true;
            } else {
                errors = format!(
                    "{}{}", errors, message
                );
            }
        };
        loop {
            use std::io;
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_number_of_bytes_read) => {
                    let input = input; // Turned into read-only
                    match mode {
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
                                match input_iter.next() {
                                    Some(fund_str) => {
                                        let mut fund_str_err = |message: &str, input, is_error| {
                                            input_err(&format!("fund_str = {}, {}", fund_str, message), input, is_error);
                                        };
                                        match input_iter.next() {
                                            Some(balance_raw) => {
                                                let balance_str = balance_raw.replace(&['$', ',', ' '][..], "");
                                                let mut balance_str_err = |message: &str, input, is_error| {
                                                    fund_str_err(&format!("balance_raw = {}, balance_str = {}, {}", balance_raw, balance_str, message), input, is_error);
                                                };
                                                match balance_str.parse::<f64>() {
                                                    Ok(balance_f) => {
                                                        let balance = (balance_f * 100.0) as i64;
                                                        let mut balance_err = |message: &str, input, is_error| {
                                                            fund_str_err(&format!("balance_f = {}, balance = {}, {}", balance_f, balance, message), input, is_error);
                                                        };
                                                        match table.table.iter_mut().find(|s| s.fund == fund_str) {
                                                            Some(series) => {
                                                                match series
                                                                    .balance
                                                                    .iter_mut()
                                                                    .find(|b: &&mut Balance| b.date == date)
                                                                {
                                                                    Some(x) => {
                                                                        if x.balance != balance {
                                                                            balance_err(&format!("Warning: Fund changing balance from {} to {}", x.balance, balance), input.clone(), false);
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
                                                                    fund_value: Vec::<_>::with_capacity(10),
                                                                });
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        balance_str_err(&format!("error code 66bVN54K: Error parsing balance_f: {}", e), input.clone(), true);
                                                    }
                                                }
                                            }
                                            None => {
                                                fund_str_err("error code nG9Y1h3k: Error parsing balance_raw", input.clone(), true);
                                            }
                                        }
                                    }
                                    None => {
                                        input_err("Error code M1xs4YCd - Error parsing fund_str", input, true);
                                    }
                                }
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
                    }
                },
                Err(error) => {
                    input_err(&format!("Error code Ug7eRN7t - Invalid data: {}", error), input, true);
                }
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
    let balance_histories = {
        let mut balance_histories = String::from("");
        let mut print_record = |fund: &str, record: &Balance| {
            balance_histories = format!(
                "{}{}\t{}\t{}\n",
                balance_histories, record.date, record.balance, fund
            )
        };
        table.table.iter().for_each(|series| {
            let mut it = series.balance.iter().rev();
            if let Some(last_record) = it.next() {
                if let Some(next_to_last_record) = it.next() {
                    print_record(&series.fund, next_to_last_record);
                }
                print_record(&series.fund, last_record);
            }
        });
        balance_histories
    };
    // The page "Recomposición de su inversión en su Dafuturo" should not be used by this program because movements between
    // funds take several days to complete. Instead, use fund actions from the "Últimos Movimientos" pages.
    if interactive_run {
        'fund_changes: loop {
            println!("{}Paste the 'Ultimos Movimientos' page here.\nEnter EOF when you are done with all pages, or Ctrl + C to close this program:", balance_histories);
            let mut mode = Mode::Header;
            let mut errors = String::new();
            let mut errors_produced = false;
            let mut repetitions = Vec::<Repetition>::with_capacity(10);
            let mut input_err = |message: &str, input| {
                errors = format!(
                    "{}{}\n{:?}", errors, message, input
                );
                errors_produced = true;
            };
            loop {
                use std::io;
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_number_of_bytes_read) => {
                        let input = input; // Turned into read-only
                        match mode {
                            Mode::Header => {
                                if input.starts_with("Fecha	Nombre del ") {
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
                                    let mut input_iter = input.split('\t'); // Do not use split_whitespace because funds and actions have spaces
                                    match input_iter.next() {
                                        Some(date_str) => {
                                            let mut date_str_err = |message: &str, input|{
                                                input_err(&format!("date_str = {}, {}", date_str, message), input);
                                            };
                                            match chrono::NaiveDate::parse_from_str(
                                                date_str,
                                                "%d/%m/%Y",
                                                ) {
                                                Ok(date) => {
                                                    let mut date_err = |message: &str, input|{
                                                        date_str_err(&format!("date = {}, {}", date, message), input);
                                                    };
                                                    match input_iter.next() {
                                                        Some(fund_str) => {
                                                            let mut fund_str_err = |message: &str, input|{
                                                                date_err(&format!("fund_str = {}, {}", fund_str, message), input);
                                                            };
                                                            match input_iter.next() {
                                                                Some(action_str) => {
                                                                    let mut action_str_err = |message: &str, input|{
                                                                        fund_str_err(&format!("action_str = {}, {}", action_str, message), input);
                                                                    };
                                                                    match input_iter.next() {
                                                                        Some(type_str) => {
                                                                            let mut type_str_err = |message: &str, input|{
                                                                                action_str_err(&format!("type_str = {}, {}", type_str, message), input);
                                                                            };
                                                                            match input_iter.next() {
                                                                                Some(change_raw) => {
                                                                                    let change_str = change_raw.replace(&['$', ',', '\n'][..], "");
                                                                                    let mut change_str_err = |message: &str, input|{
                                                                                        type_str_err(&format!("change_raw = {}, change_str = {}, {}", change_raw, change_str, message), input);
                                                                                    };
                                                                                    match change_str.parse::<f64>() {
                                                                                        Ok(change_abs) => {
                                                                                            let mut change_abs_err = |message: &str, input|{
                                                                                                change_str_err(&format!("change_abs = {}, {}", change_abs, message), input);
                                                                                            };
                                                                                            let change_f = match action_str {
                                                                                                "Aporte" | "Aporte por traslado de otro portafolio" => {
                                                                                                    change_abs
                                                                                                }
                                                                                                "Aporte por traslado a otro portafolio"
                                                                                                | "Retiro parcial" => -change_abs,
                                                                                                _ => {
                                                                                                    change_abs_err(&format!("error code KevkgKt9: Action '{}' not recognized", action_str), input.clone());
                                                                                                    continue;
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
                                                                                                        fund_value: Vec::<_>::with_capacity(10),
                                                                                                    });
                                                                                                }
                                                                                            };
                                                                                        }
                                                                                        Err(e) => {
                                                                                            change_str_err(&format!("error code G9tv9Suj: Could not parse change_str: {}", e), input.clone());
                                                                                        }
                                                                                    };
                                                                                }
                                                                                None => {
                                                                                    type_str_err("error code Vu3W9Eaw: Error parsing change_raw", input.clone());
                                                                                }
                                                                            }
                                                                        }
                                                                        None => {
                                                                            action_str_err("error code T4a6DIuK: Error parsing type_str", input.clone());
                                                                        }
                                                                    }
                                                                }
                                                                None => {
                                                                    fund_str_err("error code 30JGdhKe: Error parsing action_str", input.clone());
                                                                }
                                                            }
                                                        }
                                                        None => {
                                                            date_err("error code W55uVkoa: Error parsing fund_str", input.clone());
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    date_str_err(&format!("error code 2Zuj7zXV: Error parsing date: {}", e), input.clone());
                                                }
                                            }
                                        }
                                        None => {
                                            input_err("Error code 2NlbQ264: Error parsing date_str", input);
                                        }
                                    }
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
                    Err(error) => {
                        input_err(&format!("Error code Ug7eRN7t - Invalid data: {}", error), input);
                    }
                }
            }
            for r in repetitions {
                let action = &mut table.table[r.fund_index].action;
                let a = action[r.action_index].clone();
                let reps = r.repetition
                    - action
                        .iter()
                        .filter(|b| a.date == b.date && a.change == b.change)
                        .count() as i32;
                for _repetition in 0..reps {
                    action.push(a.clone());
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
        println!("Paste the fund and unit values here.\nEnter EOF if you have no data, or Ctrl + C to close this program:");
        let mut mode = Mode1::Header;
        let mut errors = String::new();
        let mut errors_produced = false;
        let mut input_err = |message: &str, input, is_error| {
            if is_error {
                errors = format!(
                    "{}{}\n{}", errors, message, input
                );
                errors_produced = true;
            } else {
                errors = format!(
                    "{}{}", errors, message
                );
            }
        };
        loop {
            use std::io;
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_number_of_bytes_read) => {
                    let input = input; // Turned into read-only
                    match mode {
                        Mode1::Header => {
                            if input.starts_with("PORTAFOLIO 	FECHA DE CORTE DE LA INFORMACI") {
                                mode = Mode1::SkipSubHeader;
                            } else if input == "EOF\n" {
                                break;
                            }
                        }
                        Mode1::SkipSubHeader => {
                            if input == "EOF\n" {
                                break;
                            } else {
                                mode = Mode1::Table;
                            }
                        }
                        Mode1::Table => {
                            if input.starts_with("VALOR TOTAL DEL FONDO ") {
                                println!("Moving to Footer");
                                mode = Mode1::Footer;
                            } else if input == "EOF\n" {
                                break;
                            } else {
                                let mut input_iter = input.split('\t'); // Do not use split_whitespace because funds and actions have spaces
                                match input_iter.next() {
                                    Some(fund_str) => {
                                        let mut fund_str_err = |message: &str, input, is_error| {
                                            input_err(&format!("fund_str = {}, {}", fund_str, message), input, is_error);
                                        };
                                        match input_iter.next() {
                                            Some(date_str) => {
                                                let mut date_str_err = |message: &str, input, is_error| {
                                                    fund_str_err(&format!("date_str = {}, {}", date_str, message), input, is_error);
                                                };
                                                match chrono::NaiveDate::parse_from_str(
                                                    date_str,
                                                    "%d / %m / %y ",
                                                    ) {
                                                    Ok(date) => {
                                                        let mut date_err = |message: &str, input, is_error| {
                                                            date_str_err(&format!("date = {}, {}", date, message), input, is_error);
                                                        };
                                                        match input_iter.next() {
                                                            Some(fund_value_raw) => {
                                                                let fund_value_str = fund_value_raw.replace(&['$', ',', ' '][..], "");
                                                                let mut fund_value_str_err = |message: &str, input, is_error| {
                                                                    date_err(&format!("fund_value_raw = {}, fund_value_str = {}, {}", fund_value_raw, fund_value_str, message), input, is_error);
                                                                };
                                                                match fund_value_str.parse::<f64>() {
                                                                    Ok(fund_value_f) => {
                                                                        let fund_value = (fund_value_f * 100.0) as i64;
                                                                        let mut fund_value_err = |message: &str, input, is_error| {
                                                                            fund_value_str_err(&format!("fund_value_f = {}, fund_value = {}, {}", fund_value_f, fund_value, message), input, is_error);
                                                                        };
                                                                        match input_iter.next() {
                                                                            Some(unit_value_raw) => {
                                                                                let unit_value_str = unit_value_raw.replace(&['$', ',', ' '][..], "");
                                                                                let mut unit_value_str_err = |message: &str, input, is_error| {
                                                                                    fund_value_err(&format!("unit_value_raw = {}, unit_value_str = {}, {}", unit_value_raw, unit_value_str, message), input, is_error);
                                                                                };
                                                                                match unit_value_str.parse::<f64>() {
                                                                                    Ok(unit_value_f) => {
                                                                                        let unit_value = (unit_value_f * 100.0) as i64;
                                                                                        let mut unit_value_err = |message: &str, input, is_error| {
                                                                                            unit_value_str_err(&format!("unit_value_f = {}, unit_value = {}, {}", unit_value_f, unit_value, message), input, is_error);
                                                                                        };
                                                                                        match table
                                                                                            .table
                                                                                            .iter_mut()
                                                                                            .find(|s| s.fund == fund_str)
                                                                                        {
                                                                                            Some(series) => {
                                                                                                match series
                                                                                                    .fund_value
                                                                                                    .iter_mut()
                                                                                                    .find(|u: &&mut FundValue| u.date == date)
                                                                                                {
                                                                                                    Some(x) => {
                                                                                                        if x.fund_value != fund_value {
                                                                                                            unit_value_err(&format!("Warning: Fund changing fund_value from {} to {}", x.fund_value, fund_value), input.clone(), false);
                                                                                                            x.fund_value = fund_value;
                                                                                                        }
                                                                                                        if x.unit_value != unit_value {
                                                                                                            unit_value_err(&format!("Warning: Fund changing unit_value from {} to {}", x.unit_value, unit_value), input.clone(), false);
                                                                                                            x.unit_value = unit_value;
                                                                                                        }
                                                                                                    }
                                                                                                    None => series.fund_value.push(FundValue {
                                                                                                        date,
                                                                                                        fund_value,
                                                                                                        unit_value,
                                                                                                    }),
                                                                                                }
                                                                                            }
                                                                                            None => {
                                                                                                table.table.push(Series {
                                                                                                    fund: String::from(fund_str),
                                                                                                    balance: Vec::<_>::with_capacity(10),
                                                                                                    action: Vec::<_>::with_capacity(10),
                                                                                                    fund_value: vec![FundValue {
                                                                                                        date,
                                                                                                        fund_value,
                                                                                                        unit_value,
                                                                                                    }],
                                                                                                });
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                    Err(e) => {
                                                                                        unit_value_str_err(&format!("Error code 0Rb67Jut: Error parsing unit_value_f: {}", e), input.clone(), true);
                                                                                    }
                                                                                };
                                                                            }
                                                                            None => {
                                                                                fund_value_err("error code 4j1BTEnT: Error parsing unit_value_raw", input.clone(), true);
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        fund_value_str_err(&format!("Error code eA99hBWu: Error parsing fund_value_f: {}", e), input.clone(), true);
                                                                    }
                                                                };
                                                            }
                                                            None => {
                                                                date_err("error code Yh34EUqI: Error parsing fund_value_raw", input.clone(), true);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        date_str_err(&format!("Error code 4BrJKI0R: Error parsing date: {}", e), input.clone(), true);
                                                    }
                                                }
                                            }
                                            None => {
                                                fund_str_err("error code 7v683sZZ: Error parsing date_str", input.clone(), true);
                                            }
                                        }
                                    }
                                    None => {
                                        input_err("Error code ZkX15XLd - Error parsing fund_str", input, true);
                                    }
                                }
                            }
                        }
                        Mode1::Footer => {
                            if input.starts_with("    Estas rentabilidades no son garant")
                                || (input == "EOF\n")
                            {
                                break;
                            }
                        }
                    }
                }
                Err(error) => {
                    input_err(&format!("Error code sEU78wPj - Invalid data: {}", error), input, true);
                }
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
    let table = table; // Read-only
                       // dbg!(&table);
                       // return;
    if false {
        println!("Data remains the same. Files remain unchanged.");
    } else {
        println!("Creating new funds file...");
        let new_file_name = "funds.new";
        let new_path = std::path::Path::new(&new_file_name);
        {
            let new_err = &*format!("Error writing to temporary file {}", funds_file_name);
            let new_file = fs::File::create(new_path).expect(new_err);
            let table_new = TableNew::from(table.clone());
            bincode::serialize_into(new_file, &table_new).expect(new_err);
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
            match std::fs::read_dir(dir) {
                Ok(dir_entries) => {
                    for res in dir_entries {
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
                Err(e) => {
                    panic!("Error f3QW0LUT cleaning up files from previous runs: {}", e);
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
        let color3 = &plotters::style::RGBColor(132, 156, 100);
        let color4 = &plotters::style::RGBColor(255, 231, 146);
        let color5 = &plotters::style::RGBColor(157, 85, 15);
        let color6 = &plotters::style::RGBColor(196, 53, 53);
        let color7 = &plotters::style::RGBColor(158, 138, 227);
        let color8 = &plotters::style::RGBColor(134, 202, 217);
        let color9 = &plotters::style::RGBColor(0, 199, 196);
        let color10 = &plotters::style::RGBColor(128, 128, 128);
        let color11 = &plotters::style::RGBColor(160, 130, 0);
        let color12 = &plotters::style::RGBColor(0, 140, 60);
        let color13 = &plotters::style::RGBColor(80, 103, 67);
        let color14 = &plotters::style::RGBColor(145, 143, 86);
        let color15 = &plotters::style::RGBColor(89, 56, 15);
        let color16 = &plotters::style::RGBColor(100, 53, 53);
        let color17 = &plotters::style::RGBColor(78, 100, 157);
        let color18 = &plotters::style::RGBColor(78, 144, 188);
        let color19 = &plotters::style::RGBColor(0, 110, 140);
        let color_vec = vec![
            color0, color1, color2, color3, color4, color5, color6, color7, color8, color9, color10, color11, color12, color13, color14, color15, color16, color17, color18, color19
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
        let x_label_area_size = 70;
        let y_label_area_size0 = 70;
        let y_label_area_size1 = 40;
        let figure_margin = 10;
        let line_spacing = 30;
        let thick_stroke = 3;
        let date_formatter = |date_label: &Date| format!("{}", date_label.format("%b %d"));
        let text_size0 = 30;
        let text_size1 = 24;
        let text_size2 = 24;
        let _background_text = ("Calibri", 1).into_font().color(background_color);
        let text0 = ("Calibri", text_size0).into_font().color(color0);
        let _text1 = ("Calibri", text_size1).into_font().color(color0);
        let text2 = ("Calibri", text_size2).into_font().color(color0);
        use plotters::style::text_anchor::{HPos, Pos, VPos};
        let _text2c = text2.pos(Pos::new(HPos::Center, VPos::Top));
        let durations = &[7, 15, 30, 70]; // Days
        let max_duration = durations.iter().max().unwrap();
        let minimum_date = date
            .checked_sub_signed(chrono::Duration::days(*max_duration))
            .unwrap();
        let table0 = Table {
            table: table
                .table
                .iter()
                .map(|series| Series {
                    fund: series.fund.clone(),
                    balance: {
                        let mut b: Vec<_> = series
                            .balance
                            .iter()
                            .filter(|balance| balance.date >= minimum_date)
                            .cloned()
                            .collect();
                        b.sort_unstable_by(|b1, b2| b1.date.cmp(&b2.date));
                        b
                    },
                    action: {
                        let mut a: Vec<_> = series
                            .action
                            .iter()
                            .filter(|action| action.date >= minimum_date)
                            .cloned()
                            .collect();
                        a.sort_unstable_by(|a1, a2| a1.date.cmp(&a2.date));
                        a
                    },
                    fund_value: {
                        let mut f: Vec<_> = series
                            .fund_value
                            .iter()
                            .filter(|fund_value| fund_value.date >= minimum_date)
                            .cloned()
                            .collect();
                        f.sort_unstable_by(|f1, f2| f1.date.cmp(&f2.date));
                        f
                    }, //Vec::<_>::new(),
                })
                .collect(),
        };
        // dbg!(&table0);
        {
            let figure_file_name = "fondos00.png";
            let figure_path = std::path::Path::new(&figure_file_name);
            if figure_path.exists() {
                panic!(
                    "This program just tried to rewrite {}; please debug",
                    figure_path.to_str().unwrap()
                );
            }
            let drawing_area0 = BitMapBackend::new(figure_path, (1920, 1080)).into_drawing_area();
            drawing_area0.fill(background_color).unwrap();
            drawing_area0
                .split_evenly((2, columns(durations.len())))
                .iter()
                .zip(durations.iter().enumerate())
                .for_each(|(drawing_area1, (duration_index, duration))| {
                    match date.checked_sub_signed(chrono::Duration::days(*duration)) {
                        Some(start_naive_date) => {
                            let start_date = Date::from_utc(start_naive_date, chrono::Utc);
                            let today_date = Date::from_utc(date, chrono::Utc);
                            let ranged_date =
                                plotters::coord::types::RangedDate::from(start_date..today_date);
                            let (consolidated_balance_i, consolidated_investment_i) = table0
                                .table
                                .iter()
                                .fold((0i64, 0i64), |(accum_balance, accum_investment), series| {
                                    match series.balance.iter().find(|b| b.date >= start_naive_date)
                                    {
                                        Some(initial_balance) => (
                                            accum_balance + series.balance.last().unwrap().balance,
                                            accum_investment
                                                + initial_balance.balance
                                                + series
                                                    .action
                                                    .iter()
                                                    .skip_while(|a| a.date < initial_balance.date)
                                                    .map(|a| a.change)
                                                    .sum::<i64>(),
                                        ),
                                        None => (accum_balance, accum_investment),
                                    }
                                });
                            let consolidated_investment_f64 = consolidated_investment_i as f64;
                            let consolidated_investment = consolidated_investment_f64 / 100.0;
                            let consolidated_variation =
                                consolidated_balance_i as f64 / 100.0 - consolidated_investment;
                            let consolidated_variation_percent =
                                100.0 * consolidated_variation / consolidated_investment;
                            let series_vec: Vec<_> = table0
                                .table
                                .iter()
                                .filter(|series: &&Series| {
                                    series.balance.iter().any(|b| b.date >= start_naive_date)
                                })
                                .map(|series: &Series| PlotSeries {
                                    fund: series.fund.to_lowercase(),
                                    variation: {
                                        let balance_iter = series
                                            .balance
                                            .iter()
                                            .skip_while(|b| b.date < start_naive_date);
                                        let initial_balance = balance_iter.clone().next().unwrap();
                                        let initial_balance_f64 = initial_balance.balance as f64;
                                        let mut action_iter = series
                                            .action
                                            .iter()
                                            .skip_while(|a| a.date < initial_balance.date) // skip_while() creates a new iter.
                                            .peekable();
                                        balance_iter
                                            .scan(initial_balance_f64, |running_balance, b| {
                                                let mut adjusted_current_balance = b.balance;
                                                let unadjusted_running_balance_f64 =
                                                    *running_balance as f64;
                                                #[allow(clippy::while_let_on_iterator)]
                                                while let Some(action) = action_iter.peek() {
                                                    // skip_while() creates a new iter; do not use in this loop.
                                                    if action.date >= b.date {
                                                        break;
                                                    }
                                                    *running_balance += action.change as f64;
                                                    adjusted_current_balance -= action.change;
                                                    action_iter.next();
                                                }
                                                let variation1 = 100.0
                                                    * adjusted_current_balance as f64
                                                    / unadjusted_running_balance_f64
                                                    - 100.0;
                                                let variation2 = 100.0 * b.balance as f64
                                                    / *running_balance as f64
                                                    - 100.0;
                                                Some((
                                                    Date::from_utc(b.date, chrono::Utc),
                                                    if variation1.abs() > variation2.abs() {
                                                        variation2
                                                    } else {
                                                        variation1
                                                    },
                                                ))
                                            })
                                            .collect()
                                    },
                                })
                                .collect();
                            let min_variation = match series_vec
                                .iter()
                                .map(|series| series.variation.iter().map(|a| a.1))
                                .flatten()
                                .min_by(|a, b| a.partial_cmp(&b).unwrap())
                            {
                                Some(v) => v,
                                None => 100.0,
                            };
                            let max_variation = match series_vec
                                .iter()
                                .map(|series| series.variation.iter().map(|a| a.1))
                                .flatten()
                                .max_by(|a, b| a.partial_cmp(&b).unwrap())
                            {
                                Some(v) => v,
                                None => 100.0,
                            };
                            let variation_expansion = {
                                let variation_expansion = 0.02 * (max_variation - min_variation);
                                if variation_expansion > 0. {
                                    variation_expansion
                                } else {
                                    1.
                                }
                            };
                            let variation_range = (min_variation - variation_expansion)
                                ..(max_variation + variation_expansion);
                            let mut chart = ChartBuilder::on(&drawing_area1)
                                .x_label_area_size(x_label_area_size)
                                .y_label_area_size(if duration_index == 0 {
                                    y_label_area_size0
                                } else {
                                    y_label_area_size1
                                })
                                .margin(figure_margin)
                                .caption(
                                    format!(
                                        "{} días (inversión ${:.2}, rendimiento ${:.2} ({:.2}%))",
                                        duration,
                                        consolidated_investment,
                                        consolidated_variation,
                                        consolidated_variation_percent,
                                    ),
                                    text0.clone(),
                                )
                                .build_cartesian_2d(ranged_date, variation_range)
                                .unwrap();
                            chart
                                .configure_mesh()
                                .bold_line_style(&color02)
                                .light_line_style(&color01)
                                .x_desc("Fecha")
                                .y_desc(if duration_index == 0 {
                                    "Variación respecto al portafolio inicial (%)"
                                } else {
                                    ""
                                })
                                .x_label_formatter(&date_formatter)
                                .axis_style(color0)
                                .axis_desc_style(text2.clone())
                                .label_style(text2.clone())
                                .draw()
                                .unwrap();
                            for (index, series) in series_vec.iter().enumerate() {
                                chart
                                    .draw_series(LineSeries::new(
                                        series.variation.clone(),
                                        color_vec[index].stroke_width(thick_stroke),
                                    ))
                                    .unwrap();
                            }
                            let mut labels: Vec<_> = series_vec
                                .iter()
                                .enumerate()
                                .map(|(index, series)| Label {
                                    index,
                                    fund: &series.fund,
                                    variation: series.variation.last().unwrap().1,
                                    backend_coord: {
                                        let mut bc = chart.backend_coord(&(
                                            start_date,
                                            series.variation.last().unwrap().1,
                                        ));
                                        bc.0 += 20;
                                        bc
                                    },
                                })
                                .collect();
                            labels.sort_unstable_by(|p1, p2| {
                                p1.backend_coord.1.cmp(&p2.backend_coord.1)
                            });
                            let backend_y_range = (
                                chart.backend_coord(&(start_date, max_variation)).1,
                                chart.backend_coord(&(start_date, min_variation)).1
                                    - line_spacing * labels.len() as i32,
                            );
                            labels
                                .iter()
                                .fold(backend_y_range, |(min_y, max_y), label| {
                                    let mut coord = label.backend_coord;
                                    if coord.1 < min_y {
                                        coord.1 = min_y;
                                    }
                                    if coord.1 > max_y {
                                        coord.1 = max_y;
                                    }
                                    drawing_area0
                                        .draw_text(
                                            &format!("{} {:.2}%", label.fund, label.variation),
                                            &("Calibri", text_size1)
                                                .into_font()
                                                .color(color_vec[label.index]),
                                            coord,
                                        )
                                        .unwrap();
                                    (coord.1 + line_spacing, max_y + line_spacing)
                                });
                        }
                        None => eprintln!(
                            "Error subtracting duration {} from date {}. Please review the code.",
                            *duration, date
                        ),
                    }
                });
        }
        // Total fund value
        {
            let figure_file_name = "fondos01.png";
            let figure_path = std::path::Path::new(&figure_file_name);
            if figure_path.exists() {
                panic!(
                    "This program just tried to rewrite {}; please debug",
                    figure_path.to_str().unwrap()
                );
            }
            let drawing_area0 = BitMapBackend::new(figure_path, (1920, 1080)).into_drawing_area();
            drawing_area0.fill(background_color).unwrap();
            drawing_area0
                .split_evenly((2, columns(durations.len())))
                .iter()
                .zip(durations.iter().enumerate())
                .for_each(|(drawing_area1, (duration_index, duration))| {
                    match date.checked_sub_signed(chrono::Duration::days(*duration)) {
                        Some(start_naive_date) => {
                            let start_date = Date::from_utc(start_naive_date, chrono::Utc);
                            let today_date = Date::from_utc(date, chrono::Utc);
                            let ranged_date =
                                plotters::coord::types::RangedDate::from(start_date..today_date);
                            let series_vec: Vec<_> = table0
                                .table
                                .iter()
                                .filter(|series: &&Series| {
                                    series.fund_value.iter().any(|b| b.date >= start_naive_date)
                                        && series.balance.iter().any(|b| b.date >= start_naive_date)
                                })
                                .map(|series: &Series| PlotSeries {
                                    fund: series.fund.to_lowercase(),
                                    variation: {
                                        let balance_iter = series
                                            .fund_value
                                            .iter()
                                            .skip_while(|b| b.date < start_naive_date);
                                        let initial_balance_f64 =
                                            balance_iter.clone().next().unwrap().unit_value as f64;
                                        balance_iter
                                            .map(|b| {
                                                let variation = 100.0 * b.unit_value as f64
                                                    / initial_balance_f64
                                                    - 100.0;
                                                (Date::from_utc(b.date, chrono::Utc), variation)
                                            })
                                            .collect()
                                    },
                                })
                                .collect();
                            let min_variation = match series_vec
                                .iter()
                                .map(|series| series.variation.iter().map(|a| a.1))
                                .flatten()
                                .min_by(|a, b| a.partial_cmp(&b).unwrap())
                            {
                                Some(v) => v,
                                None => 100.0,
                            };
                            let max_variation = match series_vec
                                .iter()
                                .map(|series| series.variation.iter().map(|a| a.1))
                                .flatten()
                                .max_by(|a, b| a.partial_cmp(&b).unwrap())
                            {
                                Some(v) => v,
                                None => 100.0,
                            };
                            let variation_expansion = {
                                let variation_expansion = 0.02 * (max_variation - min_variation);
                                if variation_expansion > 0. {
                                    variation_expansion
                                } else {
                                    1.
                                }
                            };
                            let variation_range = (min_variation - variation_expansion)
                                ..(max_variation + variation_expansion);
                            let mut chart = ChartBuilder::on(&drawing_area1)
                                .x_label_area_size(x_label_area_size)
                                .y_label_area_size(if duration_index == 0 {
                                    y_label_area_size0
                                } else {
                                    y_label_area_size1
                                })
                                .margin(figure_margin)
                                .caption(format!("Valor unidad {} días", duration,), text0.clone())
                                .build_cartesian_2d(ranged_date, variation_range)
                                .unwrap();
                            chart
                                .configure_mesh()
                                .bold_line_style(&color02)
                                .light_line_style(&color01)
                                .x_desc("Fecha")
                                .y_desc(if duration_index == 0 {
                                    "Variación diaria unidad (%)"
                                } else {
                                    ""
                                })
                                .x_label_formatter(&date_formatter)
                                .axis_style(color0)
                                .axis_desc_style(text2.clone())
                                .label_style(text2.clone())
                                .draw()
                                .unwrap();
                            for (index, series) in series_vec.iter().enumerate() {
                                chart
                                    .draw_series(LineSeries::new(
                                        series.variation.clone(),
                                        color_vec[index].stroke_width(thick_stroke),
                                    ))
                                    .unwrap();
                            }
                            let mut labels: Vec<_> = series_vec
                                .iter()
                                .enumerate()
                                .map(|(index, series)| Label {
                                    index,
                                    fund: &series.fund,
                                    variation: series.variation.last().unwrap().1,
                                    backend_coord: {
                                        let mut bc = chart.backend_coord(&(
                                            start_date,
                                            series.variation.last().unwrap().1,
                                        ));
                                        bc.0 += 20;
                                        bc
                                    },
                                })
                                .collect();
                            labels.sort_unstable_by(|p1, p2| {
                                p1.backend_coord.1.cmp(&p2.backend_coord.1)
                            });
                            let backend_y_range = (
                                chart.backend_coord(&(start_date, max_variation)).1,
                                chart.backend_coord(&(start_date, min_variation)).1
                                    - line_spacing * labels.len() as i32,
                            );
                            labels
                                .iter()
                                .fold(backend_y_range, |(min_y, max_y), label| {
                                    let mut coord = label.backend_coord;
                                    if coord.1 < min_y {
                                        coord.1 = min_y;
                                    }
                                    if coord.1 > max_y {
                                        coord.1 = max_y;
                                    }
                                    drawing_area0
                                        .draw_text(
                                            &format!("{} {:.2}%", label.fund, label.variation),
                                            &("Calibri", text_size1)
                                                .into_font()
                                                .color(color_vec[label.index]),
                                            coord,
                                        )
                                        .unwrap();
                                    (coord.1 + line_spacing, max_y + line_spacing)
                                });
                        }
                        None => eprintln!(
                            "Error subtracting duration {} from date {}. Please review the code.",
                            *duration, date
                        ),
                    }
                });
        }
    }
    println!("Figures are ready.");
}
