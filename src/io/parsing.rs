use crate::config::{SCENE_MARK, SILENT_PLAY_MARK};
use crate::structures::{Note, Room, Scene, ScheduleEntry};
use chrono::{NaiveDate, NaiveTime};

fn parse_scenes(scenes: &String) -> Vec<Scene> {
  if scenes.contains("Gesamtdurchlauf") || scenes.contains("Aufführung") || scenes.contains("x") {
    return vec![];
  }
  scenes.split("/").map(|s| s.trim().to_owned()).collect()
}

fn parse_note(note: &String) -> Note {
  note.trim().to_owned()
}

fn parse_room(room: &String) -> Room {
  room.trim().to_owned()
}

fn parse_date(date: &String) -> NaiveDate {
  if !["Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa.", "So."]
    .iter()
    .any(|s| date.contains(s))
  {
    todo!("Add parser error when not a date. Date: {date:?}"); // TODO:
  }
  let english_date = change_german_days_to_englisch(date);
  // TODO: Add error handling
  NaiveDate::parse_from_str(english_date.trim(), "%a. %_d.%_m.%y").unwrap()
}

fn change_german_days_to_englisch(date: &String) -> String {
  if date.contains("Mo") {
    date.replace("Mo", "Mon")
  } else if date.contains("Di") {
    date.replace("Di", "Tue")
  } else if date.contains("Mi") {
    date.replace("Mi", "Wed")
  } else if date.contains("Do") {
    date.replace("Do", "Thu")
  } else if date.contains("Fr") {
    date.replace("Fr", "Fri")
  } else if date.contains("Sa") {
    date.replace("Sa", "Sat")
  } else if date.contains("So") {
    date.replace("So", "Sun")
  } else {
    date.clone()
  }
}

fn parse_time(time: &String) -> (NaiveTime, Option<NaiveTime>) {
  if time.contains("-") {
    if let [start, stop] = time.split('-').collect::<Vec<&str>>().as_slice() {
      (
        NaiveTime::parse_from_str(start.trim(), "%H:%M").unwrap(), // TODO: Add error handling
        Some(NaiveTime::parse_from_str(stop.trim(), "%H:%M").unwrap()), // TODO: Add error handling
      )
    } else {
      todo!() // TODO: Add error handling
    }
  } else {
    (
      NaiveTime::parse_from_str(time.trim(), "%H:%M").unwrap(), // TODO: Add error handling
      None,
    )
  }
}

fn add_corresponding_stop_time(schedule_entries: Vec<ScheduleEntry>) -> Vec<ScheduleEntry> {
  if schedule_entries.len() <= 1 {
    return schedule_entries;
  }

  let mut previous_start_time = schedule_entries.last().unwrap().start_stop_time.0.clone();
  let mut previous_date = schedule_entries.last().unwrap().date.clone();
  let mut new_schedule_entries = vec![];

  for (i, entry) in schedule_entries.into_iter().rev().enumerate() {
    let previous_date_tmp = entry.date.clone();
    let previous_start_time_tmp = entry.start_stop_time.0.clone();
    if !previous_date.eq(&entry.date) || i == 0 {
      new_schedule_entries.push(entry);
    } else {
      if entry.start_stop_time.1.is_none() {
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

  use crate::structures::{ExcelParseError, SceneEntry, ScheduleEntry};

  pub fn parse_mandatory_silent_play_and_place(
    excel_range: &Range<DataType>,
  ) -> Result<(Option<NaiveDate>, Room), ExcelParseError> {
    let first_row = excel_range.rows().next().unwrap(); // TODO: Add error handling
    if first_row.len() != 5 {
      todo!("Add parser error when more than 5 rows."); // TODO:
    }
    let mandatory_silent_play = parse_date_from_excel(&first_row[1]);
    let room = parse_room_from_excel(&first_row[3]);
    if let Some(room) = room {
      Ok((mandatory_silent_play, room))
    } else {
      todo!("Add error that room is not set.")
    }
  }

  pub fn parse_schedule_plan_content(
    excel_range: &Range<DataType>,
  ) -> Result<Vec<ScheduleEntry>, ExcelParseError> {
    let mut start_parsing = false;
    let mut previous_date: Option<NaiveDate> = None;
    let mut schedule_entries = vec![];
    for (i, row) in excel_range.rows().enumerate() {
      if i == 0 {
        continue;
      }
      if row.len() != 5 {
        todo!("Add parser error when more than 5 rows."); // TODO:
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

      let date = if let Some(date) = parse_date_from_excel(&row[0]) {
        previous_date = Some(date.clone());
        Some(date)
      } else if let Some(previous_date) = previous_date {
        Some(previous_date)
      } else {
        None // TODO:
      };
      let start_stop_time = parse_time_from_excel(&row[1]);
      let scenes = parse_scenes_from_excel(&row[2]);
      let room = parse_room_from_excel(&row[3]);
      let note = parse_note_from_excel(&row[4]);

      schedule_entries.push(ScheduleEntry::new(
        date, // TODO: Remove optional
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

  fn parse_scenes_from_excel(scenes: &DataType) -> Vec<Scene> {
    if scenes == &DataType::Empty {
      vec![]
    } else {
      parse_scenes(&scenes.to_string())
    }
  }

  fn parse_time_from_excel(time: &DataType) -> (NaiveTime, Option<NaiveTime>) {
    if time == &DataType::Empty {
      todo!("Add parser error when time is empty.") // TODO:
    }
    if let Some(time) = time.as_time() {
      (time, None)
    } else {
      parse_time(&time.to_string())
    }
  }

  fn parse_date_from_excel(date: &DataType) -> Option<NaiveDate> {
    if date == &DataType::Empty {
      return None; // TODO: Use result.
    }
    if let Some(date) = date.as_date() {
      Some(date)
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
  ) -> Result<Vec<SceneEntry>, ExcelParseError> {
    let mut all_scenes = vec![];
    let mut scene_entries = vec![];
    let scene_start_index = 2;
    for (i, row) in excel_range.rows().enumerate() {
      if i == 0 {
        for scene in &row[scene_start_index..] {
          match scene {
            DataType::String(x) => all_scenes.push(x.to_owned()),
            DataType::Float(x) => all_scenes.push(x.to_string()),
            _ => todo!("Error handling all scenes"), // TODO: Error handling all scenes
          }
        }
      } else {
        let role = match &row[0] {
          DataType::String(x) => x.clone(),
          _ => todo!("Error handling role"), // TODO: Error handling role
        };
        let who = match &row[1] {
          DataType::String(x) => x.clone(),
          _ => todo!("Error handling who"), // TODO: Error handling who
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
