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

pub struct ScheduleEntry {
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub scenes: Vec<String>,
}

pub fn read_excel(path: &str, sheet_num: usize) -> Result<Range<DataType>, calamine::Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range_at(sheet_num)
        .ok_or(calamine::Error::Msg("Cannot find sheet."))??;
    Ok(range)
}

pub fn parse_schedule_plan_content(
    excel_range: Range<DataType>,
) -> Result<Vec<ScheduleEntry>, ExcelParseError> {
    for row in excel_range.rows() {
        if row.len() != 3 {
            todo!("Add parser error"); // TODO:
        }
        // TODO:
    }
    // TODO:
    Ok(vec![])
}

fn main() {
    let root_dir = Path::new(file!()).parent().and_then(|p| p.parent()).expect(
        format!(
            "Root file path not found. File '{}' has probably moved.",
            file!()
        )
        .as_str(),
    );
    let test_file_path = root_dir.join("tests/data/test_scedule.xlsx");
    let sheet_num = 0;
    if let Err(e) = read_excel(
        test_file_path
            .to_str()
            .expect("Check file name, wrong UTF-8 encoding for this os."),
        sheet_num,
    ) {
        println!("Error while reading excel: {e}");
        std::process::exit(1);
    }
}
