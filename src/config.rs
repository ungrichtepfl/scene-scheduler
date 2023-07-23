use serde::{Deserialize, Serialize};
use std::error::Error;

pub const SCENE_MARK: &str = "x";
pub const SILENT_PLAY_MARK: &str = "s";

pub const GUI_CONFIG_FILE: &str = "gui_config.json";
pub const GUI_TITLE: &str = "Scene Scheduler";

const CONFIG_FILE: &str = "config.json";

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
