use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub type Person = String;
pub type Scene = String;
pub type Room = String;
pub type Note = String;
pub type Role = String;

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
  pub role: Role,
  pub who: Person,
  pub scenes: Vec<Scene>,
  pub silent_play: Vec<bool>,
}

impl SceneEntry {
  pub fn is_scene_silent_play(&self, scene: &Scene) -> Option<bool> {
    let index = self.scenes.iter().position(|x| x == scene);
    index.map(|i| self.silent_play[i])
  }
}

#[derive(Debug)]
pub struct ScheduleEntry {
  pub date: Option<NaiveDate>,
  pub start_stop_time: (NaiveTime, Option<NaiveTime>),
  pub scenes: Vec<Scene>,
  pub room: Option<Room>,
  pub note: Option<Note>,
  pub uuid: md5::Digest,
}

impl ScheduleEntry {
  pub fn new(
    date: Option<NaiveDate>,
    start_stop_time: (NaiveTime, Option<NaiveTime>),
    scenes: Vec<Scene>,
    room: Option<Room>,
    note: Option<Note>,
  ) -> Self {
    let uuid = Self::get_uuid(&scenes, &date, &start_stop_time, &room, &note);
    Self {
      date,
      start_stop_time,
      scenes,
      room,
      note,
      uuid,
    }
  }

  pub fn start_stop_date_time(&self) -> Option<(NaiveDateTime, Option<NaiveDateTime>)> {
    match self.date {
      Some(date) => {
        let start_date_time = date.and_time(self.start_stop_time.0);
        let stop_date_time = match self.start_stop_time.1 {
          Some(stop_time) => Some(date.and_time(stop_time)),
          None => None,
        };
        Some((start_date_time, stop_date_time))
      }
      None => None,
    }
  }

  fn get_uuid(
    scenes: &Vec<String>,
    date: &Option<NaiveDate>,
    start_stop_time: &(NaiveTime, Option<NaiveTime>),
    room: &Option<String>,
    note: &Option<String>,
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
    let room_str = match room {
      Some(r) => r.to_owned(),
      None => "None".to_owned(),
    };
    let note_str = match note {
      Some(n) => n.to_owned(),
      None => "None".to_owned(),
    };
    let uuid = md5::compute(format!(
      "{}{}{}{}{}",
      date_str,
      start_stop_time_str,
      scenes.join(""),
      room_str,
      note_str,
    ));
    uuid
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ThemeType {
  Light,
  Dark,
}

const CONFIG_FILE: &'static str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub excel_file_path: String,
  pub schedule_sheet_num: usize,
  pub scene_sheet_num: usize,
  pub out_dir: String,
}

impl Config {
  pub fn load() -> Result<Self, Box<dyn Error>> {
    let config_file_path = CONFIG_FILE;
    if !std::path::Path::new(config_file_path).exists() {
      return Ok(Self::default());
    }
    let config_file = std::fs::File::open(config_file_path)?;

    let config = serde_json::from_reader(config_file)?;
    Ok(config)
  }

  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    let config_file_path = CONFIG_FILE;
    let config_file = std::fs::File::create(config_file_path)?;
    serde_json::to_writer_pretty(config_file, self)?;
    Ok(())
  }

  pub fn default() -> Self {
    Self {
      excel_file_path: "".to_owned(),
      schedule_sheet_num: 0,
      scene_sheet_num: 1,
      out_dir: "".to_owned(),
    }
  }
}
