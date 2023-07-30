use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub type Person = String;
pub type Scene = String;
pub type Room = String;
pub type Note = String;
pub type Role = String;
pub type PersonToSceneAndScheduleEntry<'a> =
  Vec<(Person, Vec<&'a (&'a ScheduleEntry, Option<&'a SceneEntry>)>)>;

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
        let stop_date_time = self
          .start_stop_time
          .1
          .map(|stop_time| date.and_time(stop_time));
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

    md5::compute(format!(
      "{}{}{}{}{}",
      date_str,
      start_stop_time_str,
      scenes.join(""),
      room_str,
      note_str,
    ))
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ThemeType {
  Light,
  Dark,
}
