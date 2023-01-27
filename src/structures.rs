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

impl ScheduleEntry {
  pub fn new(
    date: Option<NaiveDate>,
    start_stop_time: (NaiveTime, Option<NaiveTime>),
    scenes: Vec<String>,
  ) -> Self {
    let uuid = Self::get_uuid(&scenes, &date, &start_stop_time);
    Self {
      date,
      start_stop_time,
      scenes,
      uuid,
    }
  }

  fn get_uuid(
    scenes: &Vec<String>,
    date: &Option<NaiveDate>,
    start_stop_time: &(NaiveTime, Option<NaiveTime>),
  ) -> md5::Digest {
    let mut scenes = scenes.to_owned();
    scenes.sort_unstable();
    let date_str = match date {
      Some(d) => d.format("%Y-%m-%d").to_string(),
      None => "None".to_owned(),
    };
    fn time_str(time: NaiveTime) -> String {
      time.format("%H:%M").to_string()
    }
    let start_stop_time_str = match start_stop_time.1 {
      Some(stop) => format!("{}-{}", time_str(start_stop_time.0), time_str(stop)),
      None => time_str(start_stop_time.0),
    };
    let uuid = md5::compute(format!(
      "{}{}{}",
      date_str,
      start_stop_time_str,
      scenes.join("")
    ));
    uuid
  }
}
