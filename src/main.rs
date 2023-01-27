// use chrono::format::ParseError;
use std::path::Path;
mod ics;
mod io;
mod sorting;
mod structures;
use crate::ics::*;
use crate::io::{excel::*, parsing::*};
use crate::sorting::*;
// use crate::structures::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let root_dir = Path::new(file!()).parent().and_then(|p| p.parent()).expect(
    format!(
      "Root file path not found. File '{}' has probably moved.",
      file!()
    )
    .as_str(),
  );
  let test_file_path = root_dir.join("tests/data/test_schedule.xlsx");
  let test_file_path_str = test_file_path
    .to_str()
    .expect("Check file name, wrong UTF-8 encoding for this os.");

  let schedule_sheet_num = 0;
  let schedule_excel_range = read_excel(test_file_path_str, schedule_sheet_num)?;
  let schedule_entries = parse_schedule_plan_content(&schedule_excel_range)?;
  dbg!(&schedule_entries);
  let scene_sheet_num = 1;
  let scene_excel_range = read_excel(test_file_path_str, scene_sheet_num)?;
  let scene_entries = parse_scene_plan_content(scene_excel_range)?;
  dbg!(&scene_entries);
  let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
  dbg!(&schedule_to_scene_entries);
  let person_to_schedule_and_scene_entries =
    get_person_to_scene_and_schedule_entry(&schedule_to_scene_entries);

  dbg!(&person_to_schedule_and_scene_entries);
  let out_dir = String::from("ics");
  write_ics_file(&person_to_schedule_and_scene_entries, &out_dir)?;

  Ok(())
}
