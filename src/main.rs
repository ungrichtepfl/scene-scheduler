mod config;
mod gui;
mod ics;
mod io;
mod scheduler;
mod sorting;
mod structures;

use gui::Gui;
use iced::{Sandbox, Settings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  Gui::run(Settings::default())?;

  Ok(())
}
