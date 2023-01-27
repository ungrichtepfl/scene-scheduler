use chrono::{NaiveDate, NaiveTime};

use crate::structures::{ExcelParseError, SceneEntry, ScheduleEntry};
use calamine::{DataType, Range};

fn parse_scenes(scenes: &String) -> Vec<String> {
  scenes.split("/").map(|s| s.trim().to_owned()).collect()
}

pub fn get_schedule_entry(date: &String, time: &String, scenes: &String) -> ScheduleEntry {
  dbg!(date, time, scenes);
  ScheduleEntry::new(parse_date(date), parse_time(time), parse_scenes(scenes))
}

fn parse_date(date: &String) -> Option<NaiveDate> {
  if !["Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa.", "So."]
    .iter()
    .any(|s| date.contains(s))
  {
    return None;
  }
  let english_date = change_german_days_to_englisch(date);
  Some(NaiveDate::parse_from_str(english_date.trim(), "%a. %_d.%_m.%y").unwrap())
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
      panic!() // TODO: Add error handling
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

  dbg!(schedule_entries.len());
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
        new_schedule_entries.push(ScheduleEntry {
          start_stop_time: (entry.start_stop_time.0, Some(previous_start_time)),
          ..entry
        })
      } else {
        new_schedule_entries.push(entry);
      }
    }
    previous_date = previous_date_tmp;
    previous_start_time = previous_start_time_tmp;
  }
  dbg!(new_schedule_entries.len());

  new_schedule_entries.reverse();
  new_schedule_entries
}

pub fn parse_schedule_plan_content(
  excel_range: &Range<DataType>,
) -> Result<Vec<ScheduleEntry>, ExcelParseError> {
  let mut start_parsing = false;
  let mut previous_date: Option<String> = None;
  let mut schedule_entries = vec![];
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
      schedule_entries.push(get_schedule_entry(&date, &times, &scenes));
    } else if let Some(date) = &previous_date {
      schedule_entries.push(get_schedule_entry(&date, &times, &scenes));
    } else {
      panic!("No date found")
    }
  }
  Ok(add_corresponding_stop_time(schedule_entries))
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
      dbg!(&all_scenes);
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
          if mark.contains("x") {
            scenes_for_current_role.push(all_scenes[i].clone());
            silent_play.push(false);
          } else if mark.contains("-") {
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
