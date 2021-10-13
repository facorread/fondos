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
use std::io::Write as IoWrite;

enum Mode {
    Header,
    Table,
    Footer,
}

enum Mode1 {
    Header,
    SkipSubHeader,
    Table,
    Intermission,
    Table1,
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
/// Represents a record of whole fund value, unit value, and returns on equity.
struct FundValue {
    date: chrono::NaiveDate,
    /// Value of the whole fund, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    fund_value: i64,
    /// Value of a fund unit, expressed in cents. u32 is insufficient to represent large sums. f64 cannot be hashed. u64 is a hassle for working with Actions.
    unit_value: i64,
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

#[derive(Clone, Debug)]
/// A cumulative record of fund performance, to be stored in funds.csv.
struct FundCuml {
    fund: String,
    /// Return on equity for next-to-last year, expressed in percentage.
    roe_next_to_last_year: f64,
    /// Return on equity for last year, expressed in percentage.
    roe_last_year: f64,
    /// Return on equity for year to date, expressed in percentage.
    roe_year_to_date: f64,
    /// Return on equity for today, expressed in percentage.
    roe_day: f64,
    /// Return on equity for today, annualized, expressed in percentage.
    roe_day_annualized: f64,
    /// Return on equity for the last month, expressed in percentage.
    roe_month: f64,
    /// Return on equity for the last trimester, expressed in percentage.
    roe_trimester: f64,
    /// Return on equity for the last semester, expressed in percentage.
    roe_semester: f64,
    /// Return on equity for the last year, expressed in percentage.
    roe_year: f64,
    /// Return on equity for the last 2 years, expressed in percentage.
    roe_2_years: f64,
    /// Return on equity from the beginning of the fund, expressed in percentage.
    roe_total: f64,
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
    let interactive_run = true;
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
    let mut table_cuml: Vec<FundCuml> = Vec::new();
    if interactive_run {
        println!("Paste the account status here.\nEnter EOF if you have no data, or Ctrl + C to close this program:");
        let mut mode = Mode::Header;
        let mut errors = String::new();
        let mut errors_produced = false;
        let mut input_err = |message: &str, input, is_error| {
            if is_error {
                errors = format!(
                    "{}{}\n{}\n", errors, message, input
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
                    "{}{}\n{}\n", errors, message, input
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
                    "{}{}\n{}\n", errors, message, input
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
                                mode = Mode1::Intermission;
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
                                                                                        match input_iter.next() {
                                                                                            Some("$.00 ") => {
                                                                                                match input_iter.next() {
                                                                                                    Some("") => {
                                                                                                        match input_iter.next() {
                                                                                                            Some(mut roe_next_to_last_year_raw) => {
                                                                                                                if roe_next_to_last_year_raw == "NA " {
                                                                                                                    roe_next_to_last_year_raw = "0 % EA "
                                                                                                                }
                                                                                                                let mut roe_next_to_last_year_raw_err = |message: &str, input, is_error| {
                                                                                                                    unit_value_err(&format!("roe_next_to_last_year_raw = {}, {}", roe_next_to_last_year_raw, message), input, is_error);
                                                                                                                };
                                                                                                                match roe_next_to_last_year_raw.split_once(' ') {
                                                                                                                    Some((roe_next_to_last_year_str, "% EA ")) => {
                                                                                                                        let mut roe_next_to_last_year_str_err = |message: &str, input, is_error| {
                                                                                                                            roe_next_to_last_year_raw_err(&format!("roe_next_to_last_year_str = {}, {}", roe_next_to_last_year_str, message), input, is_error);
                                                                                                                        };
                                                                                                                        match roe_next_to_last_year_str.parse::<f64>() {
                                                                                                                            Ok(roe_next_to_last_year) => {
                                                                                                                                let mut roe_next_to_last_year_err = |message: &str, input, is_error| {
                                                                                                                                    roe_next_to_last_year_str_err(&format!("roe_next_to_last_year = {}, {}", roe_next_to_last_year, message), input, is_error);
                                                                                                                                };
                                                                                                                                match input_iter.next() {
                                                                                                                                    Some(mut roe_last_year_raw) => {
                                                                                                                                        if roe_last_year_raw == "NA " {
                                                                                                                                            roe_last_year_raw = "0 % EA "
                                                                                                                                        }
                                                                                                                                        let mut roe_last_year_raw_err = |message: &str, input, is_error| {
                                                                                                                                            roe_next_to_last_year_err(&format!("roe_last_year_raw = {}, {}", roe_last_year_raw, message), input, is_error);
                                                                                                                                        };
                                                                                                                                        match roe_last_year_raw.split_once(' ') {
                                                                                                                                            Some((roe_last_year_str, "% EA ")) => {
                                                                                                                                                let mut roe_last_year_str_err = |message: &str, input, is_error| {
                                                                                                                                                    roe_last_year_raw_err(&format!("roe_last_year_str = {}, {}", roe_last_year_str, message), input, is_error);
                                                                                                                                                };
                                                                                                                                                match roe_last_year_str.parse::<f64>() {
                                                                                                                                                    Ok(roe_last_year) => {
                                                                                                                                                        let mut roe_last_year_err = |message: &str, input, is_error| {
                                                                                                                                                            roe_last_year_str_err(&format!("roe_last_year = {}, {}", roe_last_year, message), input, is_error);
                                                                                                                                                        };
                                                                                                                                                        match input_iter.next() {
                                                                                                                                                            Some(mut roe_year_to_date_raw) => {
                                                                                                                                                                if roe_year_to_date_raw == "NA\n" {
                                                                                                                                                                    roe_year_to_date_raw = "0 % EA\n"
                                                                                                                                                                }
                                                                                                                                                                let mut roe_year_to_date_raw_err = |message: &str, input, is_error| {
                                                                                                                                                                    roe_last_year_err(&format!("roe_year_to_date_raw = {}, {}", roe_year_to_date_raw, message), input, is_error);
                                                                                                                                                                };
                                                                                                                                                                match roe_year_to_date_raw.split_once(' ') {
                                                                                                                                                                    Some((roe_year_to_date_str, "% EA\n")) => {
                                                                                                                                                                        let mut roe_year_to_date_str_err = |message: &str, input, is_error| {
                                                                                                                                                                            roe_year_to_date_raw_err(&format!("roe_year_to_date_str = {}, {}", roe_year_to_date_str, message), input, is_error);
                                                                                                                                                                        };
                                                                                                                                                                        match roe_year_to_date_str.parse::<f64>() {
                                                                                                                                                                            Ok(roe_year_to_date) => {
                                                                                                                                                                                let mut roe_year_to_date_err = |message: &str, input, is_error| {
                                                                                                                                                                                    roe_year_to_date_str_err(&format!("roe_year_to_date = {}, {}", roe_year_to_date, message), input, is_error);
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
                                                                                                                                                                                                    roe_year_to_date_err(&format!("Warning nwSSqjjY: Fund changing fund_value from {} to {}", x.fund_value, fund_value), input.clone(), false);
                                                                                                                                                                                                    x.fund_value = fund_value;
                                                                                                                                                                                                }
                                                                                                                                                                                                if x.unit_value != unit_value {
                                                                                                                                                                                                    roe_year_to_date_err(&format!("Warning bxZohaYm: Fund changing unit_value from {} to {}", x.unit_value, unit_value), input.clone(), false);
                                                                                                                                                                                                    x.unit_value = unit_value;
                                                                                                                                                                                                }
                                                                                                                                                                                            }
                                                                                                                                                                                            None => {
                                                                                                                                                                                                series.fund_value.push(FundValue {
                                                                                                                                                                                                    date,
                                                                                                                                                                                                    fund_value,
                                                                                                                                                                                                    unit_value,
                                                                                                                                                                                                })
                                                                                                                                                                                            },
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
                                                                                                                                                                                table_cuml.push(FundCuml {
                                                                                                                                                                                    fund: String::from(fund_str),
                                                                                                                                                                                    roe_next_to_last_year,
                                                                                                                                                                                    roe_last_year,
                                                                                                                                                                                    roe_year_to_date,
                                                                                                                                                                                    roe_day: 0.,
                                                                                                                                                                                    roe_day_annualized: 0.,
                                                                                                                                                                                    roe_month: 0.,
                                                                                                                                                                                    roe_trimester: 0.,
                                                                                                                                                                                    roe_semester: 0.,
                                                                                                                                                                                    roe_year: 0.,
                                                                                                                                                                                    roe_2_years: 0.,
                                                                                                                                                                                    roe_total: 0.,
                                                                                                                                                                                });
                                                                                                                                                                                
                                                                                                                                                                            },
                                                                                                                                                                            Err(e) => {
                                                                                                                                                                                roe_year_to_date_str_err(&format!("Error code dAEaxMDB: Error parsing roe_year_to_date: {}", e), input.clone(), true);
                                                                                                                                                                            },
                                                                                                                                                                        };
                                                                                                                                                                    },
                                                                                                                                                                    Some((erroneous1, erroneous2)) => {
                                                                                                                                                                        roe_year_to_date_raw_err(&format!("Error code LQtGqpXK: roe_year_to_date_raw should have a value 2.324 % EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                                                                    },
                                                                                                                                                                    None => {
                                                                                                                                                                        roe_year_to_date_raw_err("Error code KdXVlabV: roe_year_to_date_raw should have a value 2.324 % EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                                                                    },
                                                                                                                                                                }
                                                                                                                                                            },
                                                                                                                                                            None => {
                                                                                                                                                                roe_last_year_err("Error code FnenKjVU: Error parsing roe_year_to_date_raw", input.clone(), true);
                                                                                                                                                            }
                                                                                                                                                        };
                                                                                                                                                    },
                                                                                                                                                    Err(e) => {
                                                                                                                                                        roe_last_year_str_err(&format!("Error code HEuhdJrd: Error parsing roe_last_year: {}", e), input.clone(), true);
                                                                                                                                                    },
                                                                                                                                                };
                                                                                                                                            },
                                                                                                                                            Some((erroneous1, erroneous2)) => {
                                                                                                                                                roe_last_year_raw_err(&format!("Error code 5Hg28WR8: roe_last_year_raw should have a value 2.324 % EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                                            },
                                                                                                                                            None => {
                                                                                                                                                roe_last_year_raw_err("Error code A4vJN7cV: roe_last_year_raw should have a value 2.324 % EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                                            },
                                                                                                                                        }
                                                                                                                                    },
                                                                                                                                    None => {
                                                                                                                                        roe_next_to_last_year_err("Error code Qs99xDNc: Error parsing roe_last_year_raw", input.clone(), true);
                                                                                                                                    }
                                                                                                                                };
                                                                                                                            },
                                                                                                                            Err(e) => {
                                                                                                                                roe_next_to_last_year_str_err(&format!("Error code 0e0nPCTC: Error parsing roe_next_to_last_year: {}", e), input.clone(), true);
                                                                                                                            },
                                                                                                                        }
                                                                                                                    },
                                                                                                                    Some((erroneous1, erroneous2)) => {
                                                                                                                        roe_next_to_last_year_raw_err(&format!("Error code wLt82euc: roe_next_to_last_year_raw should have a value 2.324 % EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                    },
                                                                                                                    None => {
                                                                                                                        roe_next_to_last_year_raw_err("Error code 4F55rnmP: roe_next_to_last_year_raw should have a value 2.324 % EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                    },
                                                                                                                }
                                                                                                            },
                                                                                                            None => {
                                                                                                                unit_value_err("Error code fOh75gXn: Error parsing roe_next_to_last_year_str", input.clone(), true);
                                                                                                            }
                                                                                                        }
                                                                                                    },
                                                                                                    Some(erroneous_value) => {
                                                                                                        unit_value_err(&format!("Error code OjPUwScT: Column ** should have an empty value; instead, it has an erroneous value {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous_value), input.clone(), true);
                                                                                                    },
                                                                                                    None => {
                                                                                                        unit_value_err("Error code Y0X6wn8Q: Column ** should have an empty value; instead, it has an erroneous value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                    },
                                                                                                }
                                                                                            },
                                                                                            Some(erroneous_value) => {
                                                                                                unit_value_err(&format!("Error code ZgP73cLu: unit_change should have the value $.00; instead, it has an erroneous value {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous_value), input.clone(), true);
                                                                                            },
                                                                                            None => {
                                                                                                unit_value_err("Error code Uk94XWcH: unit_change should have the value $.00; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                            },
                                                                                        }
                                                                                    },
                                                                                    Err(e) => {
                                                                                        unit_value_str_err(&format!("Error code 0Rb67Jut: Error parsing unit_value_f: {}", e), input.clone(), true);
                                                                                    },
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
                        Mode1::Intermission => {
                            if input.starts_with("Diaria 	") {
                                mode = Mode1::Table1;
                            } else if input == "EOF\n" {
                                break;
                            }
                        }
                        Mode1::Table1 => {
                            if input == "\n" {
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
                                            Some(roe_day_raw) => {
                                                let mut roe_day_raw_err = |message: &str, input, is_error| {
                                                    fund_str_err(&format!("roe_day_raw = {}, {}", roe_day_raw, message), input, is_error);
                                                };
                                                match roe_day_raw.split_once(' ') {
                                                    Some((roe_day_str, "% ")) => {
                                                        let mut roe_day_str_err = |message: &str, input, is_error| {
                                                            roe_day_raw_err(&format!("roe_day_str = {}, {}", roe_day_str, message), input, is_error);
                                                        };
                                                        match roe_day_str.parse::<f64>() {
                                                            Ok(roe_day) => {
                                                                let mut roe_day_err = |message: &str, input, is_error| {
                                                                    roe_day_str_err(&format!("roe_day = {}, {}", roe_day, message), input, is_error);
                                                                };
                                                                match input_iter.next() {
                                                                    Some(roe_day_annualized_raw) => {
                                                                        let mut roe_day_annualized_raw_err = |message: &str, input, is_error| {
                                                                            roe_day_err(&format!("roe_day_annualized_raw = {}, {}", roe_day_annualized_raw, message), input, is_error);
                                                                        };
                                                                        match roe_day_annualized_raw.split_once(' ') {
                                                                            Some((roe_day_annualized_str, "%EA ")) => {
                                                                                let mut roe_day_annualized_str_err = |message: &str, input, is_error| {
                                                                                    roe_day_annualized_raw_err(&format!("roe_day_annualized_str = {}, {}", roe_day_annualized_str, message), input, is_error);
                                                                                };
                                                                                match roe_day_annualized_str.replace(&['$', ',', ' '][..], "").parse::<f64>() {
                                                                                    Ok(roe_day_annualized) => {
                                                                                        let mut roe_day_annualized_err = |message: &str, input, is_error| {
                                                                                            roe_day_annualized_str_err(&format!("roe_day_annualized = {}, {}", roe_day_annualized, message), input, is_error);
                                                                                        };
                                                                                        match input_iter.next() {
                                                                                            Some(roe_month_raw) => {
                                                                                                let mut roe_month_raw_err = |message: &str, input, is_error| {
                                                                                                    roe_day_annualized_err(&format!("roe_month_raw = {}, {}", roe_month_raw, message), input, is_error);
                                                                                                };
                                                                                                match roe_month_raw.split_once(' ') {
                                                                                                    Some((roe_month_str, "%EA ")) => {
                                                                                                        let mut roe_month_str_err = |message: &str, input, is_error| {
                                                                                                            roe_month_raw_err(&format!("roe_month_str = {}, {}", roe_month_str, message), input, is_error);
                                                                                                        };
                                                                                                        match roe_month_str.parse::<f64>() {
                                                                                                            Ok(roe_month) => {
                                                                                                                let mut roe_month_err = |message: &str, input, is_error| {
                                                                                                                    roe_month_str_err(&format!("roe_month = {}, {}", roe_month, message), input, is_error);
                                                                                                                };
                                                                                                                match input_iter.next() {
                                                                                                                    Some(roe_trimester_raw) => {
                                                                                                                        let mut roe_trimester_raw_err = |message: &str, input, is_error| {
                                                                                                                            roe_month_err(&format!("roe_trimester_raw = {}, {}", roe_trimester_raw, message), input, is_error);
                                                                                                                        };
                                                                                                                        match roe_trimester_raw.split_once(' ') {
                                                                                                                            Some((roe_trimester_str, "%EA ")) => {
                                                                                                                                let mut roe_trimester_str_err = |message: &str, input, is_error| {
                                                                                                                                    roe_trimester_raw_err(&format!("roe_trimester_str = {}, {}", roe_trimester_str, message), input, is_error);
                                                                                                                                };
                                                                                                                                match roe_trimester_str.parse::<f64>() {
                                                                                                                                    Ok(roe_trimester) => {
                                                                                                                                        let mut roe_trimester_err = |message: &str, input, is_error| {
                                                                                                                                            roe_trimester_str_err(&format!("roe_trimester = {}, {}", roe_trimester, message), input, is_error);
                                                                                                                                        };
                                                                                                                                        match input_iter.next() {
                                                                                                                                            Some(roe_semester_raw) => {
                                                                                                                                                let mut roe_semester_raw_err = |message: &str, input, is_error| {
                                                                                                                                                    roe_trimester_err(&format!("roe_semester_raw = {}, {}", roe_semester_raw, message), input, is_error);
                                                                                                                                                };
                                                                                                                                                match roe_semester_raw.split_once(' ') {
                                                                                                                                                    Some((roe_semester_str, "%EA ")) => {
                                                                                                                                                        let mut roe_semester_str_err = |message: &str, input, is_error| {
                                                                                                                                                            roe_semester_raw_err(&format!("roe_semester_str = {}, {}", roe_semester_str, message), input, is_error);
                                                                                                                                                        };
                                                                                                                                                        match roe_semester_str.parse::<f64>() {
                                                                                                                                                            Ok(roe_semester) => {
                                                                                                                                                                let mut roe_semester_err = |message: &str, input, is_error| {
                                                                                                                                                                    roe_semester_str_err(&format!("roe_semester = {}, {}", roe_semester, message), input, is_error);
                                                                                                                                                                };
                                                                                                                                                                match input_iter.next() {
                                                                                                                                                                    Some(roe_year_raw) => {
                                                                                                                                                                        let mut roe_year_raw_err = |message: &str, input, is_error| {
                                                                                                                                                                            roe_semester_err(&format!("roe_year_raw = {}, {}", roe_year_raw, message), input, is_error);
                                                                                                                                                                        };
                                                                                                                                                                        match roe_year_raw.split_once(' ') {
                                                                                                                                                                            Some((roe_year_str, "%EA ")) => {
                                                                                                                                                                                let mut roe_year_str_err = |message: &str, input, is_error| {
                                                                                                                                                                                    roe_year_raw_err(&format!("roe_year_str = {}, {}", roe_year_str, message), input, is_error);
                                                                                                                                                                                };
                                                                                                                                                                                match roe_year_str.parse::<f64>() {
                                                                                                                                                                                    Ok(roe_year) => {
                                                                                                                                                                                        let mut roe_year_err = |message: &str, input, is_error| {
                                                                                                                                                                                            roe_year_str_err(&format!("roe_year = {}, {}", roe_year, message), input, is_error);
                                                                                                                                                                                        };
                                                                                                                                                                                        match input_iter.next() {
                                                                                                                                                                                            Some(roe_2_years_raw) => {
                                                                                                                                                                                                let mut roe_2_years_raw_err = |message: &str, input, is_error| {
                                                                                                                                                                                                    roe_year_err(&format!("roe_2_years_raw = {}, {}", roe_2_years_raw, message), input, is_error);
                                                                                                                                                                                                };
                                                                                                                                                                                                match roe_2_years_raw.split_once(' ') {
                                                                                                                                                                                                    Some((roe_2_years_str, "%EA ")) => {
                                                                                                                                                                                                        let mut roe_2_years_str_err = |message: &str, input, is_error| {
                                                                                                                                                                                                            roe_2_years_raw_err(&format!("roe_2_years_str = {}, {}", roe_2_years_str, message), input, is_error);
                                                                                                                                                                                                        };
                                                                                                                                                                                                        match roe_2_years_str.parse::<f64>() {
                                                                                                                                                                                                            Ok(roe_2_years) => {
                                                                                                                                                                                                                let mut roe_2_years_err = |message: &str, input, is_error| {
                                                                                                                                                                                                                    roe_2_years_str_err(&format!("roe_2_years = {}, {}", roe_2_years, message), input, is_error);
                                                                                                                                                                                                                };
                                                                                                                                                                                                                match input_iter.next() {
                                                                                                                                                                                                                    Some(roe_total_raw) => {
                                                                                                                                                                                                                        let mut roe_total_raw_err = |message: &str, input, is_error| {
                                                                                                                                                                                                                            roe_2_years_err(&format!("roe_total_raw = {}, {}", roe_total_raw, message), input, is_error);
                                                                                                                                                                                                                        };
                                                                                                                                                                                                                        match roe_total_raw.split_once(' ') {
                                                                                                                                                                                                                            Some((roe_total_str, "%EA ")) => {
                                                                                                                                                                                                                                let mut roe_total_str_err = |message: &str, input, is_error| {
                                                                                                                                                                                                                                    roe_total_raw_err(&format!("roe_total_str = {}, {}", roe_total_str, message), input, is_error);
                                                                                                                                                                                                                                };
                                                                                                                                                                                                                                match roe_total_str.parse::<f64>() {
                                                                                                                                                                                                                                    Ok(roe_total) => {
                                                                                                                                                                                                                                        let mut roe_total_err = |message: &str, input, is_error| {
                                                                                                                                                                                                                                            roe_total_str_err(&format!("roe_total = {}, {}", roe_total, message), input, is_error);
                                                                                                                                                                                                                                        };
                                                                                                                                                                                                                                        match input_iter.next() {
                                                                                                                                                                                                                                            Some(roe_year_to_date_raw) => {
                                                                                                                                                                                                                                                let mut roe_year_to_date_raw_err = |message: &str, input, is_error| {
                                                                                                                                                                                                                                                    roe_total_err(&format!("roe_year_to_date_raw = {}, {}", roe_year_to_date_raw, message), input, is_error);
                                                                                                                                                                                                                                                };
                                                                                                                                                                                                                                                match roe_year_to_date_raw.split_once(' ') {
                                                                                                                                                                                                                                                    Some((roe_year_to_date_str, "%EA\n")) => {
                                                                                                                                                                                                                                                        let mut roe_year_to_date_str_err = |message: &str, input, is_error| {
                                                                                                                                                                                                                                                            roe_year_to_date_raw_err(&format!("roe_year_to_date_str = {}, {}", roe_year_to_date_str, message), input, is_error);
                                                                                                                                                                                                                                                        };
                                                                                                                                                                                                                                                        match roe_year_to_date_str.parse::<f64>() {
                                                                                                                                                                                                                                                            Ok(_roe_year_to_date) => {
                                                                                                                                                                                                                                                                // let mut roe_year_to_date_err = |message: &str, input, is_error| {
                                                                                                                                                                                                                                                                //     roe_year_to_date_str_err(&format!("roe_year_to_date = {}, {}", roe_year_to_date, message), input, is_error);
                                                                                                                                                                                                                                                                // };
                                                                                                                                                                                                                                                                match table_cuml.iter_mut().find(|u| u.fund == fund_str) {
                                                                                                                                                                                                                                                                    Some(x) => {
                                                                                                                                                                                                                                                                        x.roe_day = roe_day;
                                                                                                                                                                                                                                                                        x.roe_day_annualized = roe_day_annualized;
                                                                                                                                                                                                                                                                        x.roe_month = roe_month;
                                                                                                                                                                                                                                                                        x.roe_trimester = roe_trimester;
                                                                                                                                                                                                                                                                        x.roe_semester = roe_semester;
                                                                                                                                                                                                                                                                        x.roe_year = roe_year;
                                                                                                                                                                                                                                                                        x.roe_2_years = roe_2_years;
                                                                                                                                                                                                                                                                        x.roe_total = roe_total;
                                                                                                                                                                                                                                                                    },
                                                                                                                                                                                                                                                                    None => {},
                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                            },
                                                                                                                                                                                                                                                            Err(e) => {
                                                                                                                                                                                                                                                                roe_year_to_date_str_err(&format!("Error code U3e0L5sj: Error parsing roe_year_to_date: {}", e), input.clone(), true);
                                                                                                                                                                                                                                                            },
                                                                                                                                                                                                                                                        };
                                                                                                                                                                                                                                                    },
                                                                                                                                                                                                                                                    Some((erroneous1, erroneous2)) => {
                                                                                                                                                                                                                                                        roe_year_to_date_raw_err(&format!("Error code snYD9Q9f: roe_year_to_date_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                                                                                                                                                    },
                                                                                                                                                                                                                                                    None => {
                                                                                                                                                                                                                                                        roe_year_to_date_raw_err("Error code K7vs50R5: roe_year_to_date_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                                                                                                                                                    },
                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                            },
                                                                                                                                                                                                                                            None => {
                                                                                                                                                                                                                                                roe_total_err("Error code 6pI65S3G: Error parsing roe_year_to_date_raw", input.clone(), true);
                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                        };
                                                                                                                                                                                                                                    },
                                                                                                                                                                                                                                    Err(e) => {
                                                                                                                                                                                                                                        roe_total_str_err(&format!("Error code M3k4mygL: Error parsing roe_total: {}", e), input.clone(), true);
                                                                                                                                                                                                                                    },
                                                                                                                                                                                                                                };
                                                                                                                                                                                                                            },
                                                                                                                                                                                                                            Some((erroneous1, erroneous2)) => {
                                                                                                                                                                                                                                roe_total_raw_err(&format!("Error code 7mV9BlWN: roe_total_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                                                                                                                            },
                                                                                                                                                                                                                            None => {
                                                                                                                                                                                                                                roe_total_raw_err("Error code 8cHQ7NMn: roe_total_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                                                                                                                            },
                                                                                                                                                                                                                        }
                                                                                                                                                                                                                    },
                                                                                                                                                                                                                    None => {
                                                                                                                                                                                                                        roe_2_years_err("Error code H4du68nr: Error parsing roe_total_raw", input.clone(), true);
                                                                                                                                                                                                                    }
                                                                                                                                                                                                                };
                                                                                                                                                                                                            },
                                                                                                                                                                                                            Err(e) => {
                                                                                                                                                                                                                roe_2_years_str_err(&format!("Error code Lo8v1ItQ: Error parsing roe_2_years: {}", e), input.clone(), true);
                                                                                                                                                                                                            },
                                                                                                                                                                                                        }
                                                                                                                                                                                                    },
                                                                                                                                                                                                    Some((erroneous1, erroneous2)) => {
                                                                                                                                                                                                        roe_2_years_raw_err(&format!("Error code aXp585U0: roe_2_years_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                                                                                                    },
                                                                                                                                                                                                    None => {
                                                                                                                                                                                                        roe_2_years_raw_err("Error code A8d0Q0EN: roe_2_years_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                                                                                                    },
                                                                                                                                                                                                }
                                                                                                                                                                                            },
                                                                                                                                                                                            None => {
                                                                                                                                                                                                roe_year_err("Error code PX1d1y6Z: Error parsing roe_2_years_str", input.clone(), true);
                                                                                                                                                                                            }
                                                                                                                                                                                        }
                                                                                                                                                                                    },
                                                                                                                                                                                    Err(e) => {
                                                                                                                                                                                        roe_year_str_err(&format!("Error code 3K50holX: Error parsing roe_year: {}", e), input.clone(), true);
                                                                                                                                                                                    },
                                                                                                                                                                                };
                                                                                                                                                                            },
                                                                                                                                                                            Some((erroneous1, erroneous2)) => {
                                                                                                                                                                                roe_year_raw_err(&format!("Error code nO43crWp: roe_year_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                                                                            },
                                                                                                                                                                            None => {
                                                                                                                                                                                roe_year_raw_err("Error code 6x2UZ58c: roe_year_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                                                                            },
                                                                                                                                                                        }
                                                                                                                                                                    },
                                                                                                                                                                    None => {
                                                                                                                                                                        roe_semester_err("Error code PEpnLnhi: Error parsing roe_year_raw", input.clone(), true);
                                                                                                                                                                    }
                                                                                                                                                                };
                                                                                                                                                            },
                                                                                                                                                            Err(e) => {
                                                                                                                                                                roe_semester_str_err(&format!("Error code ZscOpTQy: Error parsing roe_semester: {}", e), input.clone(), true);
                                                                                                                                                            },
                                                                                                                                                        };
                                                                                                                                                    },
                                                                                                                                                    Some((erroneous1, erroneous2)) => {
                                                                                                                                                        roe_semester_raw_err(&format!("Error code sIkkCffw: roe_semester_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                                                    },
                                                                                                                                                    None => {
                                                                                                                                                        roe_semester_raw_err("Error code BlkcVUyg: roe_semester_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                                                    },
                                                                                                                                                }
                                                                                                                                            },
                                                                                                                                            None => {
                                                                                                                                                roe_trimester_err("Error code qdAPnejd: Error parsing roe_semester_raw", input.clone(), true);
                                                                                                                                            }
                                                                                                                                        };
                                                                                                                                    },
                                                                                                                                    Err(e) => {
                                                                                                                                        roe_trimester_str_err(&format!("Error code bOubaWtc: Error parsing roe_trimester: {}", e), input.clone(), true);
                                                                                                                                    },
                                                                                                                                }
                                                                                                                            },
                                                                                                                            Some((erroneous1, erroneous2)) => {
                                                                                                                                roe_trimester_raw_err(&format!("Error code SLKLyoxW: roe_trimester_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                                            },
                                                                                                                            None => {
                                                                                                                                roe_trimester_raw_err("Error code gXLiMlbA: roe_trimester_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                                            },
                                                                                                                        }
                                                                                                                    },
                                                                                                                    None => {
                                                                                                                        roe_month_err("Error code VjlAwhmH: Error parsing roe_trimester_str", input.clone(), true);
                                                                                                                    }
                                                                                                                }
                                                                                                            },
                                                                                                            Err(e) => {
                                                                                                                roe_month_str_err(&format!("Error code jHNqnLae: Error parsing roe_month: {}", e), input.clone(), true);
                                                                                                            },
                                                                                                        };
                                                                                                    },
                                                                                                    Some((erroneous1, erroneous2)) => {
                                                                                                        roe_month_raw_err(&format!("Error code XcpcYSZK: roe_month_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                                                    },
                                                                                                    None => {
                                                                                                        roe_month_raw_err("Error code jcPjVsko: roe_month_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                                                    },
                                                                                                }
                                                                                            },
                                                                                            None => {
                                                                                                roe_day_annualized_err("Error code elrIXnXB: Error parsing roe_month_raw", input.clone(), true);
                                                                                            }
                                                                                        };
                                                                                    },
                                                                                    Err(e) => {
                                                                                        roe_day_annualized_str_err(&format!("Error code CjlluBXh: Error parsing roe_day_annualized: {}", e), input.clone(), true);
                                                                                    },
                                                                                };
                                                                            },
                                                                            Some((erroneous1, erroneous2)) => {
                                                                                roe_day_annualized_raw_err(&format!("Error code VhhpJPnq: roe_day_annualized_raw should have a value 2.324 %EA or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                                            },
                                                                            None => {
                                                                                roe_day_annualized_raw_err("Error code YStYARdR: roe_day_annualized_raw should have a value 2.324 %EA or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                                            },
                                                                        }
                                                                    },
                                                                    None => {
                                                                        roe_day_err("Error code fSotfHSo: Error parsing roe_day_annualized_raw", input.clone(), true);
                                                                    }
                                                                };
                                                            },
                                                            Err(e) => {
                                                                roe_day_str_err(&format!("Error code GTGsBwGx: Error parsing roe_day: {}", e), input.clone(), true);
                                                            },
                                                        }
                                                    },
                                                    Some((erroneous1, erroneous2)) => {
                                                        roe_day_raw_err(&format!("Error code 4Xd090yi: roe_day_raw should have a value 2.324 % or similar; instead, it has an erroneous value {} {}; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", erroneous1, erroneous2), input.clone(), true);
                                                    },
                                                    None => {
                                                        roe_day_raw_err("Error code nCcfThLw: roe_day_raw should have a value 2.324 % or similar; instead, it has no value; this might indicate that the bank has updated the fund values page. Please review the code accordingly.", input.clone(), true);
                                                    },
                                                }
                                            },
                                            None => {
                                                fund_str_err("Error code zuZIYGcd: Error parsing roe_day_str", input.clone(), true);
                                            }
                                        }
                                    }
                                    None => {
                                        input_err("Error code wKHKaGAg - Error parsing fund_str", input, true);
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

    // Save the table to funds.dat
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
        // Delete any png and csv files from previous runs.
        for dir in &["."] {
            match std::fs::read_dir(dir) {
                Ok(dir_entries) => {
                    for res in dir_entries {
                        if let Ok(entry) = res {
                            let path = entry.path();
                            if let Some(extension) = path.extension() {
                                if (extension == "png") || (extension == "csv") {
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
        // Save fund information to funds.csv
        let csv_file_name = "funds.csv";
        let csv_path = std::path::Path::new(&csv_file_name);
        {
            let csv_err = &*format!("Error writing to CSV file {}", csv_file_name);
            let csv_file = fs::File::create(csv_path).expect(csv_err);
            writeln!(&csv_file, "Portafolio,Dia %,Dia %EA,Mes %,3 Meses,6 Meses,Ano corrido,Ano,Ano pasado,Hace 2 anos,Ultimos 2 anos,Desde el inicio").expect("Error writing CSV file header");
            for f in table_cuml {
                writeln!(&csv_file, "{},{},{},{},{},{},{},{},{},{},{},{}", f.fund, f.roe_day, f.roe_day_annualized, f.roe_month, f.roe_trimester, f.roe_semester, f.roe_year_to_date, f.roe_year, f.roe_last_year, f.roe_next_to_last_year, f.roe_2_years, f.roe_total).expect("Writing CSV file header");
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
        // Filtering and sorting table records to speed up figure production
        let table = Table {
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
                            let (consolidated_balance_i, consolidated_investment_i) = table
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
                            let series_vec: Vec<_> = table
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
        // Unit value as a proportion of the initial value
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
                            let series_vec: Vec<_> = table
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
