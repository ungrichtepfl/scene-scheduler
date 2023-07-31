mod config;
mod gui;
mod ics;
mod io;
mod scheduler;
mod sorting;
mod structures;

use gui::Gui;
use iced::{Sandbox, Settings};
use structures::SceneSchedulerError;

fn main() -> Result<(), SceneSchedulerError> {
  Gui::run(Settings::default())?;

  Ok(())
}
