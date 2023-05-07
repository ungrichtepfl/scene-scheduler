mod gui;
mod ics;
mod io;
mod scheduler;
mod sorting;
mod structures;

use iced::{Sandbox, Settings};
use scheduler::Scheduler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  Scheduler::run(Settings::default())?;

  Ok(())
}
