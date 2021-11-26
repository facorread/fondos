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
}

// Status of the data in file balances.txt
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
enum BalancesTxtStatus {
    ReadButUnprocessed,
    Processed,
    NoData,
}

/// Balances and actions expressed in cents. u32 is insufficient to represent large amounts. f64 cannot be hashed. u64 is a hassle for working with Actions.
type Cents = i64;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
/// Represents a record of the money balance in a fund.
struct Balance {
    date: chrono::NaiveDate,
    /// Balance in the fund
    /// 
    balance: Cents,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
/// Represents a record of whole fund value, unit value, and returns on equity.
struct FundValue {
    date: chrono::NaiveDate,
    /// Value of the whole fund
    fund_value: Cents,
    /// Value of a fund unit
    unit_value: Cents,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
/// Represents a record of an action in a fund.
struct Action {
    date: chrono::NaiveDate,
    /// Amount of the action
    change: Cents,
}

type Date = chrono::Date<chrono::Utc>;

/// Represents variation of a fund.
type Variation = (
    Date,
    // Variation in COP.
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
struct FundAggregate {
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

fn create_file(file_name: &str) -> Result<std::fs::File, String> {
    use {std::fs::File, std::path::Path};
    let path = Path::new(&file_name);
    File::create(path).or_else(|err| Err(format!("Error creating file {}: {}", file_name, err)))
}

fn file_lines(file_name: &str) -> Result<std::io::Lines<std::io::BufReader<std::fs::File>>, String> {
    use std::{fs::File, io::BufRead, path::Path};
    let input_path = Path::new(&file_name);
    let file = File::open(input_path).or_else(|err| Err(format!("Error reading file {}: {}", file_name, err)))?;
    Ok(std::io::BufReader::new(file).lines())
}

fn parse_name<F>(name_opt: Option<&str>, error_prefix: F) -> Result<&str, String>
where
F: Fn() -> String
{
    let trimmed_name = name_opt.ok_or_else(|| format!("{}No name", error_prefix()))?.trim();
    if trimmed_name.is_empty() {
        Err(format!("{}Empty name", error_prefix()))
    } else {
        Ok(&trimmed_name)
    }
}

fn parse_date<F>(date: &str, error_prefix: F) -> Result<chrono::NaiveDate, String>
where
F: Fn() -> String
{
    let trimmed_date = date.replace(&['$', ',', ' '][..], "");
    let err = |msg: String| Err(format!("{}{}. Is the value {} correctly formatted as a d/m/y date?", error_prefix(), msg, trimmed_date));
    if trimmed_date.is_empty() {
        return err("{}Empty date".to_string())
    }
    use chrono::Datelike;
    let parsed_date = chrono::NaiveDate::parse_from_str(&trimmed_date, "%d/%m/%Y").or_else(|e| err(e.to_string()))?;
    if parsed_date.year() < 100 {
        chrono::NaiveDate::from_ymd_opt(parsed_date.year() + 2000, parsed_date.month(), parsed_date.day()).ok_or("Transforming year from 2 to 4 digits".to_string())
    } else {
        Ok(parsed_date)
    }
}

fn parse_date_opt<F>(date_opt: Option<&str>, error_prefix: F) -> Result<chrono::NaiveDate, String>
where
F: Fn() -> String
{
    let ok_date = date_opt.ok_or_else(|| format!("{}No valid date", error_prefix()))?;
    parse_date(ok_date, error_prefix)
}

fn parse_cents<F>(pesos_opt: Option<&str>, error_prefix: F) -> Result<Cents, String>
where
F: Fn() -> String
{
    let trimmed_pesos = pesos_opt.ok_or_else(|| format!("{}No valid amount", error_prefix()))?.trim();
    let err = |msg| Err(format!("{}{}. Is the value {} correctly formatted as pesos?", error_prefix(), msg, trimmed_pesos));
    let len = trimmed_pesos.len(); // Shortest value: "$.00" "$000.00"
    if len < 4 {
        return err("Pesos value too short");
    }
    let mut cents_str = String::new();
    let mut pesos_it = trimmed_pesos.chars();
    if pesos_it.next() != Some('$') {
        return err("Value has no $ sign");
    }
    let mut comma_digits = -1;
    let mut decimal_digits = -1;
    let mut largest_digits = 0;
    match pesos_it.next() {
        Some('.') => {
            decimal_digits = 0;
        },
        Some(c) if c.is_digit(10) => {
            cents_str.push(c);
            largest_digits += 1;
        },
        Some(c) => {
            return err(&format!("Pesos value starts with ${} instead of a number", c));
        }
        None => {
            return err("Pesos value is cut short");
        }
    };

    while let Some(c) = pesos_it.next() {
        if decimal_digits < 0 {
            if comma_digits < 0 {
                match c {
                    '.' => decimal_digits = 0,
                    ',' => comma_digits = 0,
                    '0'..='9' => {
                        cents_str.push(c);
                        largest_digits += 1;
                        if largest_digits > 3 {
                            return err("Pesos value has too many digits before the first ','");
                        }
                    },
                    _ => return err(&format!("Pesos value has invalid character '{}'", c)),
                }
            } else {
                comma_digits += 1;
                comma_digits %= 4;
                if comma_digits == 0 {
                    if c == '.' {
                        decimal_digits = 0;
                    } else if c != ',' {
                        return err(&format!("Pesos value has '{}' instead of the ',' thousands separator", c));
                    }
                } else {
                    if c.is_digit(10) {
                        cents_str.push(c);
                    } else {
                        return err(&format!("Pesos value has '{}' instead of a digit", c));
                    }
                }
            }
        } else {
            if c.is_digit(10) {
                cents_str.push(c);
                decimal_digits += 1;
            } else {
                return err(&format!("Pesos value has '{}' instead of a decimal digit", c));
            }
        }
    }
    if decimal_digits == 1 {
        return err("Pesos value has 1 decimal digit instead of 2");
    }
    if decimal_digits != 2 {
        return err(&format!("Pesos value has {} decimal digits instead of 2", decimal_digits));
    }
    cents_str.parse::<i64>().map_err(|e| e.to_string())
}

fn parse_percent<F>(percent_opt: Option<&str>, error_prefix: F) -> Result<f64, String>
where
F: Fn() -> String
{
    let trimmed_percent = percent_opt.ok_or_else(|| format!("{}No valid amount", error_prefix()))?.trim();
    let err = |msg| Err(format!("{}{}. Is the value {} correctly formatted as a percentage?", error_prefix(), msg, trimmed_percent));
    if trimmed_percent == "NA" {
        return Ok(f64::NAN);
    }
    let len = trimmed_percent.len(); // Shortest value: "$.00" "$000.00"
    if len < 3 {
        return err("Percent value too short");
    }
    let mut percent_str = String::new();
    let mut percent_it = trimmed_percent.chars();
    let mut comma_digits = -1;
    let mut decimal_digits = -1;
    let mut largest_digits = 0;
    match percent_it.next() {
        Some('.') => {
            percent_str.push('.');
            decimal_digits = 0;
        },
        Some('-') => {
            percent_str.push('-');
        },
        Some(c) if c.is_digit(10) => {
            percent_str.push(c);
            largest_digits += 1;
        },
        Some(c) => {
            return err(&format!("Percentage starts with {} instead of a number", c));
        }
        None => {
            return err("Percentage is cut short");
        }
    };

    while let Some(c) = percent_it.next() {
        if decimal_digits < 0 {
            if comma_digits < 0 {
                match c {
                    '.' => {
                        percent_str.push('.');
                        decimal_digits = 0
                    },
                    ',' => comma_digits = 0,
                    ' ' | '%' => {
                        break;
                    }
                    '0'..='9' => {
                        percent_str.push(c);
                        largest_digits += 1;
                        if largest_digits > 3 {
                            return err("Percentage has too many digits before the first ','");
                        }
                    },
                    _ => return err(&format!("Percentage has invalid character '{}'", c)),
                }
            } else {
                comma_digits += 1;
                comma_digits %= 4;
                if comma_digits == 0 {
                    match c {
                        '.' => {
                            percent_str.push('.');
                            decimal_digits = 0;
                        }
                        ',' => {}
                        ' ' | '%' => {
                            break;
                        }
                        _ => {
                            return err(&format!("Percentage has '{}' instead of a decimal point, thousands separator, space, or percent sign", c));
                        }
                    }
                } else {
                    if c.is_digit(10) {
                        percent_str.push(c);
                    } else {
                        return err(&format!("Percentage has '{}' instead of a digit", c));
                    }
                }
            }
        } else {
            match c {
                c if c.is_digit(10) => {
                    percent_str.push(c);
                }
                ' ' | '%' => {
                    break;
                }
                _ => {
                    return err(&format!("Percentage has '{}' instead of a decimal digit", c));
                }
            }
        }
    }
    percent_str.parse::<f64>().map_err(|e| e.to_string())
}

fn columns(n_durations: usize) -> usize {
    n_durations / 2 + n_durations % 2
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;
    use std::fs;
    let date = chrono::Local::today().naive_local();
    let funds_file_name = "data/funds.dat";
    let r_err0 = |e| Err(format!("Error reading the file {}: {}", funds_file_name, e));
    let r_err1 = |e| Err(format!("Error reading the file {}: {}", funds_file_name, e)); // Two closures with similar name; they differ in the type of e. Reminder: Rust does not define generic closures.
    let w_err0 = |e| Err(format!("Error writing to file {}: {}", funds_file_name, e));
    let db_path = std::path::Path::new(&funds_file_name);
    let mut table: Table = if db_path.exists() {
        let db_file = fs::File::open(db_path).or_else(r_err0)?;
        bincode::deserialize_from(db_file).or_else(r_err1)?
    } else {
        println!("Starting a new file. Ctrl + C if this is a mistake.");
        Table {
            table: Vec::<_>::with_capacity(10),
        }
    };
    let original_hash = calculate_hash(&table);
    table.table.iter_mut().for_each(|s| s.fund = s.fund.trim().to_lowercase());

    // A few examples useful for debugging
    // table.table.iter().find(|s| s.fund == "capital").unwrap().action.iter().enumerate().for_each(|r| println!("{:?}", r));
    // table.table.iter().find(|s| s.fund == "consumo global").unwrap().balance.iter().enumerate().for_each(|r| println!("{:?}", r));

    // It is usual to transfer all money from one fund to another.
    // In those cases, the emptied fund disappears from balances.txt and history.txt.
    // We must manually account for this disappearance.
    // {
    //     let emptied_fund = &mut table.table.iter_mut().find(|s| s.fund == "consumo global").unwrap();
    //     let date = chrono::NaiveDate::from_ymd(2021, 11, 19);
    //     emptied_fund.action.push(Action {
    //         date,
    //         change: -321686700,
    //     });
    //     emptied_fund.action.push(Action {
    //         date,
    //         change: -1209632,
    //     });
    //     let date = date.succ();
    //     match emptied_fund.balance.iter_mut().find(|b| b.date == date) {
    //         Some(b) => {
    //             b.balance = 0;
    //         }
    //         None => {
    //             emptied_fund.balance.push(Balance {
    //                 date,
    //                 balance: 0,
    //             });
    //         }
    //     }
    // }
    let mut table_aggregate: Vec<FundAggregate> = Vec::new();
    // Process balances.txt
    {
        let mut mode = Mode::Header;
        let mut input_lines = Vec::new();
        let mut fund_data_status = BalancesTxtStatus::NoData;
        for (line_index, input_res) in file_lines("balances.txt")?.enumerate() {
            let input = input_res?;
            match mode {
                Mode::Header => {
                    if input == "Anual**" {
                        mode = Mode::Table;
                    }
                }
                Mode::Table => {
                    if input.starts_with("Total	") {
                        mode = Mode::Footer;
                    } else {
                        fund_data_status = BalancesTxtStatus::ReadButUnprocessed;
                        input_lines.push((line_index, input));
                    }
                }
                Mode::Footer => {
                    assert_eq!(fund_data_status, BalancesTxtStatus::ReadButUnprocessed);
                    match input.strip_prefix("*Los valores presentados están a la fecha de cierre") {
                        Some(date_str) => {
                            let date = parse_date(date_str, || format!("Parsing date at balances.txt line {}: ", line_index + 1))?;
                            fund_data_status = BalancesTxtStatus::Processed;
                            for (line_index, input) in input_lines.into_iter() {
                                let mut fields = input.split('\t');
                                let fund_name = parse_name(fields.next(), || format!("Parsing fund name at balances.txt line {} field 1: ", line_index + 1))?.to_lowercase();
                                let balance = parse_cents(fields.next(), || format!("Parsing {} fund balance at balances.txt line {} field 2: ", fund_name, line_index + 1))?;
                                assert_eq!(fields.count(), 4); // 4 remaining fields, to be left unused
                                match table.table.iter_mut().find(|s| s.fund == fund_name) {
                                    Some(series) => {
                                        match series
                                            .balance
                                            .iter_mut()
                                            .find(|b: &&mut Balance| b.date == date)
                                        {
                                            Some(b) => {
                                                if b.balance != balance {
                                                    println!("Warning: Fund changing balance from {} to {}", b.balance, balance);
                                                    b.balance = balance;
                                                }
                                            }
                                            None => {
                                                series.balance.push(Balance { date, balance })
                                            }
                                        }
                                    }
                                    None => {
                                        table.table.push(Series {
                                            fund: String::from(fund_name),
                                            balance: vec![Balance { date, balance }],
                                            action: Vec::<_>::with_capacity(10),
                                            fund_value: Vec::<_>::with_capacity(10),
                                        });
                                    }
                                }
                            }
                            break; // Stop reading the file
                        },
                        None => {},
                    }
                }
            }
        }
        assert_ne!(fund_data_status, BalancesTxtStatus::ReadButUnprocessed);
    }
    // Process history.txt
    {
        let mut repetitions = Vec::<Repetition>::with_capacity(10);
        let mut skip_header = true;
        for (line_index, input_res) in file_lines("history.txt")?.enumerate() {
            let input = input_res?;
            if skip_header {
                if input.starts_with("Fecha	Nombre del ") {
                    skip_header = false;
                }
            } else if input.is_empty() {
                skip_header = true; // Waiting to start processing the history of the next fund
            } else {
                let mut fields = input.split('\t');
                let date = parse_date_opt(fields.next(), || format!("Parsing date at balances.txt line {} field 1: ", line_index + 1))?;
                let fund_name = parse_name(fields.next(), || format!("Parsing fund name at balances.txt line {} field 2: ", line_index + 1))?.to_lowercase();
                let action_str = parse_name(fields.next(), || format!("Parsing event description at balances.txt line {} field 3: ", line_index + 1))?;
                let _unused_str = parse_name(fields.next(), || format!("Parsing event type at balances.txt line {} field 4: ", line_index + 1))?;
                let change_abs = parse_cents(fields.next(), || format!("Parsing {} fund balance at balances.txt line {} field 5: ", fund_name, line_index + 1))?;
                assert_eq!(fields.count(), 0); // 0 remaining fields
                let change = match action_str {
                    "Aporte" | "Aporte por traslado de otro portafolio" => {
                        change_abs
                    }
                    "Aporte por traslado a otro portafolio"
                    | "Retiro parcial" => -change_abs,
                    _ => {
                        use std::io::{Error, ErrorKind};
                        return Err(Box::new(Error::new(ErrorKind::Other, format!("error code KevkgKt9: Action '{}' not recognized", action_str))));
                    }
                };
                match table.table.iter().position(|s| s.fund == fund_name) {
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
                            fund: String::from(fund_name),
                            balance: vec![],
                            action: vec![Action { date, change }],
                            fund_value: Vec::<_>::with_capacity(10),
                        });
                    }
                };
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
    }
    // Process profit.txt
    {
        let mut mode = Mode1::Header;
        for (line_index, input_res) in file_lines("profit.txt")?.enumerate() {
            let input = input_res?;
            match mode {
                Mode1::Header => {
                    if input.starts_with("PORTAFOLIO 	FECHA DE CORTE DE LA INFORMACI") {
                        mode = Mode1::SkipSubHeader;
                    }
                }
                Mode1::SkipSubHeader => {
                    mode = Mode1::Table;
                }
                Mode1::Table => {
                    if input.starts_with("VALOR TOTAL DEL FONDO ") {
                        mode = Mode1::Intermission;
                        continue;
                    }
                    let mut fields = input.split('\t');
                    let fund_name = parse_name(fields.next(), || format!("Parsing fund name at profit.txt line {} field 1: ", line_index + 1))?.to_lowercase();
                    let date = parse_date_opt(fields.next(), || format!("Parsing date at profit.txt line {} field 2: ", line_index + 1))?;
                    let fund_value = parse_cents(fields.next(), || format!("Parsing {} fund value at profit.txt line {} field 3: ", fund_name, line_index + 1))?;
                    let unit_value = parse_cents(fields.next(), || format!("Parsing {} unit value at profit.txt line {} field 4: ", fund_name, line_index + 1))?;
                    let zero_value = parse_name(fields.next(), || format!("Parsing {} a zero value at profit.txt line {} field 5: ", fund_name, line_index + 1))?;
                    assert_eq!(zero_value, "$.00");
                    assert_eq!(parse_name(fields.next(), || format!("Parsing a null value for {} at profit.txt line {} field 6: ", fund_name, line_index + 1)), Err(format!("Parsing a null value for {} at profit.txt line {} field 6: Empty name", fund_name, line_index + 1)));
                    let roe_next_to_last_year = parse_percent(fields.next(), || format!("Parsing {} returns from 2 years ago at profit.txt line {} field 7: ", fund_name, line_index + 1))?;
                    let roe_last_year = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 8: ", fund_name, line_index + 1))?;
                    let roe_year_to_date = parse_percent(fields.next(), || format!("Parsing {} returns from year to date at profit.txt line {} field 9: ", fund_name, line_index + 1))?;
                    assert_eq!(fields.count(), 0); // 0 remaining fields
                    match table.table.iter_mut().find(|s| s.fund == fund_name) {
                        Some(series) => {
                            match series
                                .fund_value
                                .iter_mut()
                                .find(|u: &&mut FundValue| u.date == date)
                            {
                                Some(x) => {
                                    if x.fund_value != fund_value {
                                        println!("Warning nwSSqjjY: Fund {} changing fund_value from {} to {}", fund_name, x.fund_value, fund_value);
                                        x.fund_value = fund_value;
                                    }
                                    if x.unit_value != unit_value {
                                        println!("Warning bxZohaYm: Fund {} changing unit_value from {} to {}", fund_name, x.unit_value, unit_value);
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
                                fund: String::from(&fund_name),
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
                    table_aggregate.push(FundAggregate {
                        fund: String::from(fund_name),
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
                }
                Mode1::Intermission => {
                    if input.starts_with("Diaria 	") {
                        mode = Mode1::Table1;
                    }
                }
                Mode1::Table1 => {
                    if input == "" {
                        break;
                    }
                    let mut fields = input.split('\t');
                    let fund_name = parse_name(fields.next(), || format!("Parsing fund name at profit.txt line {} field 1: ", line_index + 1))?;
                    let roe_day = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 2: ", fund_name, line_index + 1))?;
                    let roe_day_annualized = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 3: ", fund_name, line_index + 1))?;
                    let roe_month = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 4: ", fund_name, line_index + 1))?;
                    let roe_trimester = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 5: ", fund_name, line_index + 1))?;
                    let roe_semester = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 6: ", fund_name, line_index + 1))?;
                    let roe_year = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 7: ", fund_name, line_index + 1))?;
                    let roe_2_years = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 8: ", fund_name, line_index + 1))?;
                    let roe_total = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 9: ", fund_name, line_index + 1))?;
                    let _roe_year_to_date = parse_percent(fields.next(), || format!("Parsing {} returns from last year at profit.txt line {} field 10: ", fund_name, line_index + 1))?;
                    assert_eq!(fields.count(), 0); // 0 remaining fields
                    match table_aggregate.iter_mut().find(|u| u.fund == fund_name) {
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
                }
            }
        }
    }
    // Sort table
    {
        table.table.iter_mut().for_each(|series| {
            series.balance.sort_unstable();
            series.action.sort_unstable();
            series.fund_value.sort_unstable();
        });
    }
    // Check fund transfer consistency: Check for that every withdrawal from a fund has a corresponding deposit into another.
    {
        let non_empty = |s: &&Series| !s.balance.is_empty() && s.balance.last().unwrap().balance != 0;
        let recent = |a: &&Action| a.date > chrono::NaiveDate::from_ymd(2021, 11, 13);
        let fund_selection: Vec<_> = table.table.iter().filter(non_empty)
        .map(|s| (s.fund.clone(), s.balance.last().unwrap())).collect();
        for s1 in table.table.iter().filter(non_empty) {
            for a1 in s1.action.iter().filter(recent) {
                let mut expected_action = a1.clone();
                expected_action.change = -expected_action.change;
                let mut match_found = false;
                for s2 in table.table.iter() {
                    for a2 in s2.action.iter() {
                        if *a2 == expected_action {
                            match_found = true;
                        }
                    }
                }
                if !match_found {
                    let best_matching_fund = fund_selection.iter().map(|(n, b)| (n, b, b.balance - a1.change))
                    .min_by(|a, b| a.2.abs().cmp(&b.2.abs())).unwrap();
                    println!("{}: no match: {:?}; nearest fund {:?}", s1.fund, a1, best_matching_fund);
                }
            }
        }
    }
    // println!("Data is not saved to disk. Data {}", if calculate_hash(&table) == original_hash { "remains unchanged." } else { "has changed." }); return Ok(());
    // Save the table to funds.dat
    if calculate_hash(&table) == original_hash {
        println!("Data remains the same. Files remain unchanged.");
    } else {
        println!("Creating new funds file...");
        let new_file_name = "data/funds.new";
        let new_path = std::path::Path::new(&new_file_name);
        {
            let new_file = fs::File::create(new_path).or_else(|e| Err(format!("Error writing to temporary file {}: {}", new_file_name, e)))?;
            bincode::serialize_into(new_file, &table).or_else(|e| Err(format!("Error writing to temporary file {}: {}", new_file_name, e)))?;
        }
        if db_path.exists() {
            let backup_file_name = format!(
                "data/funds_backup{}.dat",
                chrono::Local::now().format("%Y%m%dT%H%M%S")
            );
            let to = std::path::Path::new(&backup_file_name);
            fs::rename(db_path, to).or_else(|e| Err(format!("Error creating backup {}: {}", backup_file_name, e)))?;
        }
        fs::rename(new_path, db_path).or_else(w_err0)?;
    }
    {
        // Delete any png and csv files from previous runs.
        for res in std::fs::read_dir(".")? {
            if let Ok(entry) = res {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if (extension == "png") || (extension == "csv") {
                        if let Some(file_name_os_str) = path.file_name() {
                            if let Some(file_name) = file_name_os_str.to_str() {
                                if let Err(e) = fs::remove_file(&path) {
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
    {
        // Save fund information to funds.csv
        let csv_file_name = "funds.csv";
        let csv_err = |e| Err(format!("Error writing to {}: {}", csv_file_name, e));
        let csv_file = create_file(csv_file_name)?;
        writeln!(&csv_file, "Portafolio,Dia %,Dia %EA,Mes %,3 Meses,6 Meses,Ano corrido,Ano,Ano pasado,Hace 2 anos,Ultimos 2 anos,Desde el inicio").or_else(csv_err)?;
        for f in table_aggregate {
            writeln!(&csv_file, "{},{},{},{},{},{},{},{},{},{},{},{}", f.fund, f.roe_day, f.roe_day_annualized, f.roe_month, f.roe_trimester, f.roe_semester, f.roe_year_to_date, f.roe_year, f.roe_last_year, f.roe_next_to_last_year, f.roe_2_years, f.roe_total).or_else(csv_err)?;
        }
    }
    // Save latest movements to file comparison.csv
    {
        let csv_file_name = "comparison.csv";
        let csv_file = create_file(csv_file_name)?;
        let csv_err = |e| Err(format!("Error writing to {}: {}", csv_file_name, e));
        writeln!(&csv_file, "Fund,Previous date,Previous $,Change,Last date,Last $").or_else(csv_err)?;
        for series in table.table.iter() {
            let mut it = series.balance.iter().rev();
            if let Some(last_record) = it.next() {
                write!(&csv_file, "{}", &series.fund).or_else(csv_err)?;
                let last_record_balance = last_record.balance as f64 / 100.0;
                if let Some(next_to_last_record) = it.next() {
                    let next_to_last_record_balance = next_to_last_record.balance as f64 / 100.0;
                    write!(&csv_file, ",{},{},{}", next_to_last_record.date, next_to_last_record_balance, last_record_balance - next_to_last_record_balance).or_else(csv_err)?;
                } else {
                    write!(&csv_file, ",,,").or_else(csv_err)?;
                }
                writeln!(&csv_file, ",{},{}", last_record.date, last_record_balance).or_else(csv_err)?;
            }
        };
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
        let y_label_area_size0 = 140;
        let y_label_area_size1 = 120;
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
        // Retain recent records for plotting
        {
            let max_duration = durations.iter().max().unwrap();
            let minimum_date = date
                .checked_sub_signed(chrono::Duration::days(*max_duration))
                .unwrap();
            table.table.iter_mut().for_each(|series| {
                series.balance.retain(|r| r.date >= minimum_date);
                series.action.retain(|r| r.date >= minimum_date);
                series.fund_value.retain(|r| r.date >= minimum_date);
            });
        }
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
                            // Calculate consolidated balances across funds
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
                                    fund: series.fund.clone(),
                                    variation: {
                                        let balance_iter = series
                                            .balance
                                            .iter()
                                            .skip_while(|b| b.date < start_naive_date);
                                        let initial_balance = balance_iter.clone().next().unwrap();
                                        let mut action_iter = series
                                            .action
                                            .iter()
                                            .skip_while(|a| a.date < initial_balance.date) // skip_while() creates a new iter.
                                            .peekable();
                                        balance_iter
                                            .scan(initial_balance.balance, |running_balance, b| {
                                                let mut adjusted_current_balance = b.balance;
                                                let unadjusted_running_balance = *running_balance;
                                                #[allow(clippy::while_let_on_iterator)]
                                                while let Some(action) = action_iter.peek() {
                                                    // skip_while() creates a new iter; do not use in this loop.
                                                    if action.date >= b.date {
                                                        break;
                                                    }
                                                    *running_balance += action.change;
                                                    adjusted_current_balance -= action.change;
                                                    action_iter.next();
                                                }
                                                let variation1 = adjusted_current_balance - unadjusted_running_balance;
                                                let variation2 = b.balance - *running_balance;
                                                Some((
                                                    Date::from_utc(b.date, chrono::Utc),
                                                    if variation1.abs() > variation2.abs() {
                                                        variation2 as f64 / 100.0
                                                    } else {
                                                        variation1 as f64 / 100.0
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
                                    "Variación respecto al portafolio inicial ($)"
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
                                            &format!("{} {:.2}", label.fund, label.variation),
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
            let accessible_funds = vec!["acciones colombia",  "acciones global",  "capital",  "consumo global",  "diver dinamico",  "diver moderado",  "diver. conservador",  "estable", "preserva",  "renta fija global",  "renta fija pesos",  "sostenible global"];
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
                                        && accessible_funds.contains(&series.fund.as_str())
                                })
                                .map(|series: &Series| PlotSeries {
                                    fund: series.fund.clone(),
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
    println!("Figures and data files are ready. Please run the following:\n    start *.png\n    start *.csv");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn cents0() {
        assert_eq!(super::parse_cents(Some("$.00"), || "Test: ".to_string()), Ok(0));
    }
    #[test]
    fn cents1() {
        assert_eq!(super::parse_cents(Some("$1.74"), || "Test: ".to_string()), Ok(174));
    }
    #[test]
    fn cents2() {
        assert_eq!(super::parse_cents(Some("$1,231.74"), || "Test: ".to_string()), Ok(123174));
    }
    #[test]
    fn cents3() {
        assert_eq!(super::parse_cents(Some("$211,231.74"), || "Test: ".to_string()), Ok(21123174));
    }
    #[test]
    fn cents4() {
        assert_eq!(super::parse_cents(Some("$9,211,231.74"), || "Test: ".to_string()), Ok(921123174));
    }
    #[test]
    fn cents5() {
        assert_eq!(super::parse_cents(Some("$,211,231.74"), || "Test: ".to_string()), Err("Test: Pesos value starts with $, instead of a number. Is the value $,211,231.74 correctly formatted as pesos?".to_string()));
    }
    #[test]
    fn cents6() {
        assert_eq!(super::parse_cents(Some("$1,2,1,231.74"), || "Test: ".to_string()), Err("Test: Pesos value has ',' instead of a digit. Is the value $1,2,1,231.74 correctly formatted as pesos?".to_string()));
    }
    #[test]
    fn cents7() {
        assert_eq!(super::parse_cents(Some(" $1,211,231.7 "), || "Test: ".to_string()), Err("Test: Pesos value has 1 decimal digit instead of 2. Is the value $1,211,231.7 correctly formatted as pesos?".to_string()));
    }
    #[test]
    fn cents8() {
        assert_eq!(super::parse_cents(Some(" $1,211,231.740 "), || "Test: ".to_string()), Err("Test: Pesos value has 3 decimal digits instead of 2. Is the value $1,211,231.740 correctly formatted as pesos?".to_string()));
    }
    #[test]
    fn cents9() {
        assert_eq!(super::parse_cents(Some(" $1211,231.74 "), || "Test: ".to_string()), Err("Test: Pesos value has too many digits before the first ','. Is the value $1211,231.74 correctly formatted as pesos?".to_string()));
    }
    #[test]
    fn date0() {
        assert_eq!(super::parse_date_opt(None, || "Test: ".to_string()), Err("Test: No valid date".to_string()));
    }
    #[test]
    fn date1() {
        assert_eq!(super::parse_date_opt(Some(" 31/12/2021 "), || "Test: ".to_string()), Ok(chrono::NaiveDate::from_ymd(2021, 12, 31)));
    }
    #[test]
    fn date2() {
        assert_eq!(super::parse_date_opt(Some(" 31/13/2021 "), || "Test: ".to_string()), Err("Test: input is out of range. Is the value 31/13/2021 correctly formatted as a d/m/y date?".to_string()));
    }
    #[test]
    fn date3() {
        assert_eq!(super::parse_date_opt(Some(" 31 / 12 / 21 "), || "Test: ".to_string()), Ok(chrono::NaiveDate::from_ymd(2021, 12, 31)));
    }
    #[test]
    fn percent0() {
        assert_eq!(super::parse_percent(None, || "Test: ".to_string()), Err("Test: No valid amount".to_string()));
    }
    #[test]
    fn percent1() {
        assert_eq!(super::parse_percent(Some("0 %"), || "Test: ".to_string()), Ok(0.0));
    }
    #[test]
    fn percent2() {
        assert_eq!(super::parse_percent(Some("c %EA"), || "Test: ".to_string()), Err("Test: Percentage starts with c instead of a number. Is the value c %EA correctly formatted as a percentage?".to_string()));
    }
    #[test]
    fn percent3() {
        assert_eq!(super::parse_percent(Some("1000 %EA"), || "Test: ".to_string()), Err("Test: Percentage has too many digits before the first ','. Is the value 1000 %EA correctly formatted as a percentage?".to_string()));
    }
    #[test]
    fn percent4() {
        assert_eq!(super::parse_percent(Some("10f %EA"), || "Test: ".to_string()), Err("Test: Percentage has invalid character 'f'. Is the value 10f %EA correctly formatted as a percentage?".to_string()));
    }
    #[test]
    fn percent5() {
        assert_eq!(super::parse_percent(Some("1,000p.00 %EA"), || "Test: ".to_string()), Err("Test: Percentage has 'p' instead of a decimal point, thousands separator, space, or percent sign. Is the value 1,000p.00 %EA correctly formatted as a percentage?".to_string()));
    }
    #[test]
    fn percent6() {
        assert_eq!(super::parse_percent(Some("1,00q.00 %EA"), || "Test: ".to_string()), Err("Test: Percentage has 'q' instead of a digit. Is the value 1,00q.00 %EA correctly formatted as a percentage?".to_string()));
    }
    #[test]
    fn percent7() {
        assert_eq!(super::parse_percent(Some("1,000.0r %EA"), || "Test: ".to_string()), Err("Test: Percentage has 'r' instead of a decimal digit. Is the value 1,000.0r %EA correctly formatted as a percentage?".to_string()));
    }
    #[test]
    fn percent8() {
        assert_eq!(super::parse_percent(Some("3 %EA"), || "Test: ".to_string()), Ok(3.));
    }
    #[test]
    fn percent9() {
        assert_eq!(super::parse_percent(Some("-3,324,122,432.643 %EA"), || "Test: ".to_string()), Ok(-3_324_122_432.643));
    }
    #[test]
    fn percent10() {
        assert_eq!(super::parse_percent(Some("-0.003 %EA"), || "Test: ".to_string()), Ok(-0.003));
    }
    #[test]
    fn percent11() {
        assert_eq!(super::parse_percent(Some("-.003 %EA"), || "Test: ".to_string()), Ok(-0.003));
    }
    #[test]
    fn percent12() {
        assert_eq!(super::parse_percent(Some(".003 %EA"), || "Test: ".to_string()), Ok(0.003));
    }
}
