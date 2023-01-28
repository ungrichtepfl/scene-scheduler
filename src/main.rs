use std::path::Path;
mod ics;
mod io;
mod sorting;
mod structures;
use crate::ics::*;
use crate::io::{excel::*, parsing::excel::*};
use crate::sorting::*;

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
  let (mandatory_silet_play, location) =
    parse_mandatory_silent_play_and_place(&schedule_excel_range)?;
  let scene_sheet_num = 1;
  let scene_excel_range = read_excel(test_file_path_str, scene_sheet_num)?;
  let scene_entries = parse_scene_plan_content(scene_excel_range)?;
  let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
  let filtered_schedule_to_scene_entries =
    filter_by_non_empty_schedule_entry_date(&schedule_to_scene_entries);
  let filtered_schedule_to_scene_entries_2 =
    if let Some(mandatory_silet_play) = &mandatory_silet_play {
      filter_by_silent_play(&filtered_schedule_to_scene_entries, mandatory_silet_play)
    } else {
      filtered_schedule_to_scene_entries
    };
  let person_to_schedule_and_scene_entries =
    get_person_to_scene_and_schedule_entry(&filtered_schedule_to_scene_entries_2);

  let out_dir = String::from("ics");
  write_ics_file(&person_to_schedule_and_scene_entries, &out_dir, &location)?;

  Ok(())
}
