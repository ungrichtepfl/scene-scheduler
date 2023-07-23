use std::path::Path;

use crate::structures::{Person, SceneEntry, ScheduleEntry};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone};
use chrono_tz::{Europe::Zurich, Tz};
use ics::properties::{Description, DtEnd, DtStart, Location, Status, Summary};
use ics::{escape_text, Event, ICalendar};

const ICAL_STR_FORMAT: &str = "%Y%m%dT%H%M%SZ";
const DEFAULT_EVENT_DURATION_HOURS: i64 = 4;

pub fn write_ics_file(
  person_to_scene_and_schedule_entry: &Vec<(Person, Vec<&(&ScheduleEntry, Option<&SceneEntry>)>)>,
  out_dir: &str,
  default_location: &str,
) -> std::io::Result<()> {
  if person_to_scene_and_schedule_entry.is_empty() {
    return Ok(());
  }

  std::fs::create_dir_all(out_dir)?;

  for (person, schedule_to_scene_entries) in person_to_scene_and_schedule_entry {
    // dtsart.format("%Y%m%dT%H%M%SZ").to_string()
    let mut calendar = ICalendar::new("2.0", "-//Fungiking//NONSGML Scene Scheduler//DE");
    for (schedule_entry, scene_entry) in schedule_to_scene_entries {
      if let Some(start_end_date_time_naive) = schedule_entry.start_stop_date_time() {
        let (start_date_time_str, stop_date_time_str) =
          get_start_and_end_time_utc(&start_end_date_time_naive);

        // create event which contains the information regarding the conference
        // add properties
        let mut event = Event::new(
          format!("{:x}", schedule_entry.uuid),
          chrono::Utc::now().format(ICAL_STR_FORMAT).to_string(),
        );
        event.push(DtStart::new(start_date_time_str));
        event.push(DtEnd::new(stop_date_time_str));
        event.push(Status::confirmed());
        event.push(Summary::new("Theater"));
        if let Some(location) = &schedule_entry.room {
          event.push(Location::new(location));
        } else {
          event.push(Location::new(default_location));
        }
        // Values that are "TEXT" must be escaped (only if the text contains a comma,
        // semicolon, backslash or newline).
        let mut description = String::new();
        if let Some(scene_entry) = scene_entry {
          description.push_str(format!("Rolle: {}\n", scene_entry.role).as_str());
        };
        if !schedule_entry.scenes.is_empty() {
          description.push_str(format!("Szenen: {}\n", schedule_entry.scenes.join(", ")).as_str());
        };
        if let Some(note) = &schedule_entry.note {
          description.push_str(format!("Anmerkung: {}\n", note).as_str());
        };
        event.push(Description::new(escape_text(description)));
        // add event to calendar
        calendar.add_event(event);
      } else {
        // No date specified yet
        continue;
      }
    }

    // write calendar to file
    let mut out_file_path = Path::new(out_dir).join(person);
    out_file_path.set_extension("ics");
    calendar.save_file(out_file_path)?;
  }
  Ok(())
}

fn get_start_and_end_time_utc(
  start_end_date_time: &(NaiveDateTime, Option<NaiveDateTime>),
) -> (String, String) {
  let (start_date_time_naive, stop_date_time_opt) = start_end_date_time;

  let start_date = naive_to_date_time(&start_date_time_naive);
  let stop_date = if let Some(stop_date_time_naive) = stop_date_time_opt {
    naive_to_date_time(&stop_date_time_naive)
  } else {
    start_date + Duration::hours(DEFAULT_EVENT_DURATION_HOURS)
  };
  (
    start_date.naive_utc().format(ICAL_STR_FORMAT).to_string(),
    stop_date.naive_utc().format(ICAL_STR_FORMAT).to_string(),
  )
}

fn naive_to_date_time(naive_date_time: &NaiveDateTime) -> DateTime<Tz> {
  // TODO: Pick timezone from user input or locale of the computer
  Zurich.from_local_datetime(naive_date_time).unwrap() // TODO: Better error handling stop date
}
