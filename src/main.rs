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
    pub date: NaiveDate,
    pub start_stop_time: (NaiveTime, Option<NaiveTime>),
    pub scenes: Vec<String>,
}

impl ScheduleEntry {
    pub fn new(date: &String, time: &String, scenes: &String) -> ScheduleEntry {
        dbg!(date, time, scenes);
        ScheduleEntry {
            date: ScheduleEntry::parse_date(date),
            start_stop_time: ScheduleEntry::parse_time(time),
            scenes: ScheduleEntry::parse_scenes(scenes),
        }
    }
    fn parse_date(date: &String) -> NaiveDate {
        //TODO:
        chrono::offset::Local::now().date_naive()
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
    fn parse_scenes(scenes: &String) -> Vec<String> {
        // TODO:
        vec![]
    }
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
            scedule_entries.push(ScheduleEntry::new(&date, &times, &scenes));
        } else if let Some(date) = &previous_date {
            scedule_entries.push(ScheduleEntry::new(&date, &times, &scenes));
        } else {
            panic!("No date found")
        }
    }
    Ok(scedule_entries)
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
