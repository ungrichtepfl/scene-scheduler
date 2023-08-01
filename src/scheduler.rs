use crate::config::*;
use crate::ics::*;
use crate::io::{excel::*, parsing::excel::*};
use crate::sorting::*;
use crate::structures::SceneSchedulerError;

#[derive(Debug)]
pub struct Scheduler {
  pub config: Config,
}
impl Scheduler {
  pub fn process(&self) -> Result<(), SceneSchedulerError> {
    let (schedule_excel_range, schedule_excel_worksheet_name) =
      read_excel(&self.config.excel_file_path, self.config.schedule_sheet_num)?;
    let schedule_entries = parse_schedule_plan_content(
      &schedule_excel_range,
      &self.config.excel_file_path,
      &schedule_excel_worksheet_name,
    )?;
    let (mandatory_silet_play, location): (_, String) = parse_mandatory_silent_play_and_place(
      &schedule_excel_range,
      &self.config.excel_file_path,
      &schedule_excel_worksheet_name,
    )?;
    let (scene_excel_range, scene_excel_worksheet_name) =
      read_excel(&self.config.excel_file_path, self.config.scene_sheet_num)?;
    let scene_entries = parse_scene_plan_content(
      scene_excel_range,
      &self.config.excel_file_path,
      &scene_excel_worksheet_name,
    )?;
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    let filtered_schedule_to_scene_entries =
      if let Some(mandatory_silet_play) = &mandatory_silet_play {
        filter_by_silent_play(&schedule_to_scene_entries, mandatory_silet_play)
      } else {
        schedule_to_scene_entries
      };
    let person_to_schedule_and_scene_entries =
      get_person_to_scene_and_schedule_entry(&filtered_schedule_to_scene_entries);

    write_ics_file(
      &person_to_schedule_and_scene_entries,
      &self.config.out_dir,
      &location,
    )?;

    Ok(())
  }
}
