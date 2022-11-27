#![allow(unused_variables)] // TODO: Remove when stable

use calamine::{open_workbook, DataType, Range, Reader, Xlsx};
use chrono::format::ParseError;
use chrono::{NaiveDate, NaiveTime};
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct ExcelParseError {
    file: String,
    sheet: usize,
    row: usize,
    column: usize,
    token: String,
    expected: String,
}
impl fmt::Display for ExcelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parsing error for file '{}' in sheet number '{}' (row {}, column {}). {}. Unexpected token '{}'.", self.file, self.sheet, self.row, self.column, self.expected,self.token)
    }
}

impl Error for ExcelParseError {}

#[derive(Debug)]
pub struct ScheduleEntry {
    pub date: Option<NaiveDate>,
    pub start_stop_time: (NaiveTime, Option<NaiveTime>),
    pub scenes: Vec<String>,
}

pub fn get_scedule_entry(date: &String, time: &String, scenes: &String) -> ScheduleEntry {
    dbg!(date, time, scenes);
    ScheduleEntry {
        date: parse_date(date),
        start_stop_time: parse_time(time),
        scenes: parse_scenes(scenes),
    }
}
fn parse_date(date: &String) -> Option<NaiveDate> {
    //TODO:

    Some(chrono::offset::Local::now().date_naive())
}

fn parse_time(time: &String) -> (NaiveTime, Option<NaiveTime>) {
    if time.contains("-") {
        if let [start, stop] = time.split('-').collect::<Vec<&str>>().as_slice() {
            (
                NaiveTime::parse_from_str(start.trim(), "%H:%M").unwrap(), // TODO: Add error handling
                Some(NaiveTime::parse_from_str(stop.trim(), "%H:%M").unwrap()), // TODO: Add error handling
            )
        } else {
            panic!() // TODO: Add error handling
        }
    } else {
        (
            NaiveTime::parse_from_str(time.trim(), "%H:%M").unwrap(), // TODO: Add error handling
            None,
        )
    }
}

fn add_corresponding_stop_time(scedule_entries: Vec<ScheduleEntry>) -> Vec<ScheduleEntry> {
    if scedule_entries.len() <= 1 {
        return scedule_entries;
    }

    dbg!(scedule_entries.len());
    let mut previous_start_time = scedule_entries.last().unwrap().start_stop_time.0.clone();
    let mut previous_date = scedule_entries.last().unwrap().date.clone();
    let mut new_scedule_entries = vec![];

    for (i, entry) in scedule_entries.into_iter().rev().enumerate() {
        let previous_date_tmp = entry.date.clone();
        let previous_start_time_tmp = entry.start_stop_time.0.clone();
        if !previous_date.eq(&entry.date) || i == 0 {
            new_scedule_entries.push(entry);
        } else {
            if entry.start_stop_time.1.is_none() {
                new_scedule_entries.push(ScheduleEntry {
                    start_stop_time: (entry.start_stop_time.0, Some(previous_start_time)),
                    ..entry
                })
            } else {
                new_scedule_entries.push(entry);
            }
        }
        previous_date = previous_date_tmp;
        previous_start_time = previous_start_time_tmp;
    }
    dbg!(new_scedule_entries.len());

    new_scedule_entries.reverse();
    new_scedule_entries
}

fn parse_scenes(scenes: &String) -> Vec<String> {
    scenes.split("/").map(|s| s.trim().to_owned()).collect()
}

pub fn read_excel(path: &str, sheet_num: usize) -> Result<Range<DataType>, calamine::Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range_at(sheet_num)
        .ok_or(calamine::Error::Msg("Cannot find sheet."))??;
    Ok(range)
}

pub fn parse_schedule_plan_content(
    excel_range: &Range<DataType>,
) -> Result<Vec<ScheduleEntry>, ExcelParseError> {
    let mut start_parsing = false;
    let mut previous_date: Option<String> = None;
    let mut scedule_entries = vec![];
    for row in excel_range.rows() {
        if row.len() != 3 {
            todo!("Add parser error when more than 3 rows."); // TODO:
        }
        let (opt_date, times, scenes) = match row {
            [DataType::String(x), DataType::String(y), DataType::String(z)] => {
                (Some(x.to_owned()), y.to_owned(), z.to_owned())
            }
            [DataType::String(x), DataType::String(y), DataType::Float(z)] => {
                (Some(x.to_owned()), y.to_owned(), z.to_string())
            }
            [DataType::Empty, DataType::String(y), DataType::Float(z)] => {
                (None, y.to_owned(), z.to_string())
            }
            [DataType::Empty, DataType::String(y), DataType::String(z)] => {
                (None, y.to_owned(), z.to_owned())
            }
            _ => {
                todo!("Add parser error when not only strings. Row: {row:?}")
            }
        };

        if !start_parsing {
            if !times.eq(&String::from("Zeit")) {
                continue;
            } else {
                start_parsing = true;
                continue;
            }
        }
        if let Some(date) = &opt_date {
            previous_date = Some(date.clone());
            scedule_entries.push(get_scedule_entry(&date, &times, &scenes));
        } else if let Some(date) = &previous_date {
            scedule_entries.push(get_scedule_entry(&date, &times, &scenes));
        } else {
            panic!("No date found")
        }
    }
    Ok(add_corresponding_stop_time(scedule_entries))
}

fn main() -> Result<(), Box<dyn Error>> {
    let root_dir = Path::new(file!()).parent().and_then(|p| p.parent()).expect(
        format!(
            "Root file path not found. File '{}' has probably moved.",
            file!()
        )
        .as_str(),
    );
    let test_file_path = root_dir.join("tests/data/test_scedule.xlsx");
    let sheet_num = 0;
    let excel_range = read_excel(
        test_file_path
            .to_str()
            .expect("Check file name, wrong UTF-8 encoding for this os."),
        sheet_num,
    )?;
    let scedule_entries = parse_schedule_plan_content(&excel_range)?;
    dbg!(scedule_entries);

    Ok(())
}
