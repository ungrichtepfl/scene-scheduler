use chrono::{NaiveDate, NaiveTime};
use std::error::Error;
use std::fmt;

pub type Person = String;

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
pub struct SceneEntry {
  pub role: String,
  pub who: Person,
  pub scenes: Vec<String>,
  pub silent_play: Vec<bool>,
}

#[derive(Debug)]
pub struct ScheduleEntry {
  pub date: Option<NaiveDate>,
  pub start_stop_time: (NaiveTime, Option<NaiveTime>),
  pub scenes: Vec<String>,
  pub uuid: md5::Digest,
}
