use crate::config::{SCENE_MARK, SILENT_PLAY_MARK};
use crate::structures::{Note, Room, SceneSchedulerError, Scenes, ScheduleEntry};
use chrono::{NaiveDate, NaiveTime};
use lazy_static::lazy_static;
use regex::Regex;

fn parse_scenes(scenes: &str) -> Scenes {
  if scenes.trim() == "" {
    Scenes::Normal(vec![])
  } else if scenes.trim().as_bytes()[0].is_ascii_digit() {
    Scenes::Normal(
      scenes
        .split(&['/', ','])
        .map(|s| s.trim().to_owned())
        .collect(),
    )
  } else {
    Scenes::Special(scenes.trim().to_owned())
  }
}

fn parse_note(note: &str) -> Note {
  note.trim().to_owned()
}

fn parse_room(room: &str) -> Room {
  room.trim().to_owned()
}

lazy_static! {
  static ref DATE_REGEX: Regex = Regex::new(r"\d\d?\.\d\d?\.\d\d").expect("Wrong static regex");
}
fn parse_date(date: &str) -> Option<NaiveDate> {
  let date_cap = DATE_REGEX.captures(date)?;
  let date_str = date_cap.get(0)?.as_str().to_owned();

  NaiveDate::parse_from_str(date_str.trim(), "%_d.%_m.%y").ok()
}

fn parse_time(time: &String) -> Option<(NaiveTime, Option<NaiveTime>)> {
  match time.split(&['-', '–']).collect::<Vec<_>>()[..] {
    [start, stop] => {
      let start_date = NaiveTime::parse_from_str(start.trim(), "%H:%M").ok()?;
      let stop_date = NaiveTime::parse_from_str(stop.trim(), "%H:%M").ok()?;
      Some((start_date, Some(stop_date)))
    }
    [start] => {
      let start_date = NaiveTime::parse_from_str(start.trim(), "%H:%M").ok()?;
      Some((start_date, None))
    }
    _ => None,
  }
}

fn add_corresponding_stop_time(schedule_entries: Vec<ScheduleEntry>) -> Vec<ScheduleEntry> {
  if schedule_entries.len() <= 1 {
    return schedule_entries;
  }

  let mut previous_start_time = schedule_entries.last().unwrap().start_stop_time.0;
  let mut previous_date = schedule_entries.last().unwrap().date;
  let mut new_schedule_entries = vec![];

  for (i, entry) in schedule_entries.into_iter().rev().enumerate() {
    let previous_date_tmp = entry.date;
    let previous_start_time_tmp = entry.start_stop_time.0;
    if !previous_date.eq(&entry.date) || i == 0 {
      new_schedule_entries.push(entry);
    } else if entry.start_stop_time.1.is_none() {
      new_schedule_entries.push(ScheduleEntry::new(
        entry.date,
        (entry.start_stop_time.0, Some(previous_start_time)),
        entry.scenes,
        entry.room,
        entry.note,
      ))
    } else {
      new_schedule_entries.push(entry);
    }
    previous_date = previous_date_tmp;
    previous_start_time = previous_start_time_tmp;
  }

  new_schedule_entries.reverse();
  new_schedule_entries
}

pub mod excel {
  use super::*;
  use calamine::{DataType, Range};
  use chrono::NaiveDate;

  use crate::structures::{SceneEntry, ScheduleEntry};

  pub fn parse_mandatory_silent_play_and_place(
    excel_range: &Range<DataType>,
    file_path: &str,
    sheet_name: &str,
  ) -> Result<(Option<NaiveDate>, Room), SceneSchedulerError> {
    let first_row = excel_range
      .rows()
      .next()
      .ok_or_else(|| SceneSchedulerError::ExcelError {
        message: String::from(
          "No first row found. Needs to contain at least the information about the location.",
        ),
        file: file_path.to_owned(),
        sheet: sheet_name.to_owned(),
      })?;

    if first_row.len() < 2 {
      return Err(SceneSchedulerError::ExcelError {
        message: String::from(
          "Wrong Excel file format. First row should contain the information about the location.",
        ),
        file: file_path.to_owned(),
        sheet: sheet_name.to_owned(),
      });
    }
    let room =
      parse_room_from_excel(&first_row[1]).ok_or_else(|| SceneSchedulerError::ExcelParseError {
        file: file_path.to_owned(),
        sheet: sheet_name.to_owned(),
        row: 1,
        column: 2,
        expected: String::from("The location should be specified."),
        token: first_row[1].to_string(),
      })?;
    let mandatory_silent_play = match parse_date_from_excel(&first_row[3]) {
      Some(Some(date)) => Some(date),
      Some(None) => {
        return Err(SceneSchedulerError::ExcelParseError {
          file: file_path.to_owned(),
          sheet: sheet_name.to_owned(),
          row: 1,
          column: 4,
          expected: String::from("The date should be specified."),
          token: first_row[3].to_string(),
        })
      }
      None => None,
    };
    Ok((mandatory_silent_play, room))
  }

  pub fn parse_schedule_plan_content(
    excel_range: &Range<DataType>,
    file_path: &str,
    sheet_name: &str,
  ) -> Result<Vec<ScheduleEntry>, SceneSchedulerError> {
    let mut start_parsing = false;
    let mut previous_date: Option<NaiveDate> = None;
    let mut schedule_entries = vec![];
    for (i, row) in excel_range.rows().enumerate() {
      if i == 0 {
        continue;
      }
      if row.len() < 5 {
        return Err(SceneSchedulerError::ExcelError {
          message: String::from("Wrong Excel file format. There should be 5 columns."),
          file: file_path.to_owned(),
          sheet: sheet_name.to_owned(),
        });
      }

      if !start_parsing {
        // TODO: Don't hardcode
        if !row[0].to_string().trim().eq(&String::from("Datum")) {
          continue;
        } else {
          start_parsing = true;
          continue;
        }
      }

      let date = match parse_date_from_excel(&row[0]) {
        Some(Some(date)) => {
          previous_date = Some(date);
          date
        }
        Some(None) => {
          return Err(SceneSchedulerError::ExcelParseError {
            file: file_path.to_owned(),
            sheet: sheet_name.to_owned(),
            row: i + 1,
            column: 1,
            expected: String::from("Wrong date string format should be DD.MM.YY., e.g. 01.01.22."),
            token: row[0].to_string(),
          })
        }
        None => previous_date.ok_or_else(|| SceneSchedulerError::ExcelParseError {
          file: file_path.to_owned(),
          sheet: sheet_name.to_owned(),
          row: i + 1,
          column: 1,
          expected: String::from("The date should be specified."),
          token: row[0].to_string(),
        })?,
      };
      let start_stop_time =
        parse_time_from_excel(&row[1]).ok_or_else(|| SceneSchedulerError::ExcelParseError {
          file: file_path.to_owned(),
          sheet: sheet_name.to_owned(),
          row: i + 1,
          column: 2,
          expected: String::from("Wrong time string format should be HH:MM, e.g. 12:00"),
          token: row[1].to_string(),
        })?;
      let scenes = parse_scenes_from_excel(&row[2]);
      let room = parse_room_from_excel(&row[3]);
      let note = parse_note_from_excel(&row[4]);

      schedule_entries.push(ScheduleEntry::new(
        date,
        start_stop_time,
        scenes,
        room,
        note,
      ));
    }
    Ok(add_corresponding_stop_time(schedule_entries))
  }

  fn parse_note_from_excel(note: &DataType) -> Option<Note> {
    if note == &DataType::Empty {
      None
    } else {
      Some(parse_note(&note.to_string()))
    }
  }

  fn parse_scenes_from_excel(scenes: &DataType) -> Scenes {
    if scenes == &DataType::Empty {
      Scenes::Normal(vec![])
    } else {
      parse_scenes(&scenes.to_string())
    }
  }

  fn parse_time_from_excel(time: &DataType) -> Option<(NaiveTime, Option<NaiveTime>)> {
    if time == &DataType::Empty {
      return None;
    }
    if let Some(time) = time.as_time() {
      Some((time, None))
    } else {
      parse_time(&time.to_string())
    }
  }

  fn parse_date_from_excel(date: &DataType) -> Option<Option<NaiveDate>> {
    if date == &DataType::Empty || date.to_string().trim().is_empty() {
      return None; // TODO: Make this better
    }
    if let Some(date) = date.as_date() {
      Some(Some(date))
    } else {
      Some(parse_date(&date.to_string()))
    }
  }

  fn parse_room_from_excel(room: &DataType) -> Option<Room> {
    if room == &DataType::Empty {
      None
    } else {
      Some(parse_room(&room.to_string()))
    }
  }

  pub fn parse_scene_plan_content(
    excel_range: Range<DataType>,
    file_path: &str,
    sheet_name: &str,
  ) -> Result<Vec<SceneEntry>, SceneSchedulerError> {
    let mut all_scenes = vec![];
    let mut scene_entries = vec![];
    let scene_start_index = 2;
    for (i, row) in excel_range.rows().enumerate() {
      if i == 0 {
        let mut column_index = scene_start_index;
        for scene in &row[scene_start_index..] {
          match scene {
            DataType::String(x) => all_scenes.push(x.to_owned()),
            DataType::Float(x) => all_scenes.push(x.to_string()),
            _ => {
              return Err(SceneSchedulerError::ExcelParseError {
                file: file_path.to_owned(),
                sheet: sheet_name.to_owned(),
                row: i + 1,
                column: column_index + 1,
                expected: String::from("Scene name should be a string or a float."),
                token: row[0].to_string(),
              })
            }
          }
          column_index += 1;
        }
      } else {
        if row[0] == DataType::Empty && row[1] == DataType::Empty {
          // End of scene plan
          break;
        }
        let role = match &row[0] {
          DataType::String(x) => x.clone(),
          _ => {
            return Err(SceneSchedulerError::ExcelParseError {
              file: file_path.to_owned(),
              sheet: sheet_name.to_owned(),
              row: i + 1,
              column: 1,
              expected: String::from("Role should be a string."),
              token: row[0].to_string(),
            })
          }
        };
        let who = match &row[1] {
          DataType::String(x) => x.clone(),
          _ => {
            return Err(SceneSchedulerError::ExcelParseError {
              file: file_path.to_owned(),
              sheet: sheet_name.to_owned(),
              row: i + 1,
              column: 2,
              expected: String::from("Person who plays the role should be a string."),
              token: row[1].to_string(),
            })
          }
        };
        let mut scenes_for_current_role = vec![];
        let mut silent_play = vec![];
        for (i, scene) in row[scene_start_index..].iter().enumerate() {
          if let DataType::String(mark) = scene {
            if mark.contains(SCENE_MARK) {
              scenes_for_current_role.push(all_scenes[i].clone());
              silent_play.push(false);
            } else if mark.contains(SILENT_PLAY_MARK) {
              scenes_for_current_role.push(all_scenes[i].clone());
              silent_play.push(true);
            }
          }
        }
        scene_entries.push(SceneEntry {
          role,
          who,
          scenes: scenes_for_current_role,
          silent_play,
        })
      }
    }
    Ok(scene_entries)
  }
}
