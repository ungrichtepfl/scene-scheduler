mod gui;
mod ics;
mod io;
mod scheduler;
mod sorting;
mod structures;

use gui::Scheduler;
use iced::{Sandbox, Settings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  Scheduler::run(Settings::default())?;

  Ok(())
}
