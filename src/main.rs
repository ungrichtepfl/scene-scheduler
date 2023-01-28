mod ics;
mod io;
mod scheduler;
mod sorting;
mod structures;

use std::path::Path;

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
  let scene_sheet_num = 1;
  let out_dir = String::from("ics");
  let config = structures::Config {
    excel_file_path: test_file_path_str.to_owned(),
    schedule_sheet_num,
    scene_sheet_num,
    out_dir,
  };
  scheduler::run(config)?;

  Ok(())
}
