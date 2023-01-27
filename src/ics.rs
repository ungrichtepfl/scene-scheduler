use crate::structures::{Person, SceneEntry, ScheduleEntry};
use chrono::{Duration, NaiveDateTime, TimeZone};
use chrono_tz::Europe::Zurich;
use ics::properties::{Description, DtEnd, DtStart, Status, Summary};
use ics::{escape_text, Event, ICalendar};

const ICAL_STR_FORMAT: &str = "%Y%m%dT%H%M%SZ";
const DEFAULT_EVENT_DURATION_HOURS: i64 = 4;

pub fn write_ics_file(
  person_to_scene_and_schedule_entry: &Vec<(Person, Vec<&(&ScheduleEntry, &SceneEntry)>)>,
  out_dir: &String,
) -> std::io::Result<()> {
  if person_to_scene_and_schedule_entry.is_empty() {
    return Ok(());
  }

  std::fs::create_dir_all(out_dir)?;

  for (person, schedule_to_scene_entries) in person_to_scene_and_schedule_entry {
    // dtsart.format("%Y%m%dT%H%M%SZ").to_string()
    let mut calendar = ICalendar::new("2.0", "-//EnsembLee//NONSGML Scene Scheduler//DE");
    for (schedule_entry, scene_entry) in schedule_to_scene_entries {
      // TODO: filter for silet play
      if let Some((start_date_time, stop_date_time)) = get_start_and_end_time(&schedule_entry) {
        // create event which contains the information regarding the conference
        // add properties
        let mut event = Event::new(
          format!("{:x}", schedule_entry.uuid),
          chrono::Utc::now().format(ICAL_STR_FORMAT).to_string(),
        );
        event.push(DtStart::new(start_date_time));
        event.push(DtEnd::new(stop_date_time));
        event.push(Status::confirmed());
        event.push(Summary::new("Theater"));
        // Values that are "TEXT" must be escaped (only if the text contains a comma,
        // semicolon, backslash or newline).
        let description = format!(
          "Role: {}\nScenen: {}",
          scene_entry.role,
          schedule_entry.scenes.join("/")
        );
        event.push(Description::new(escape_text(description)));
        // add event to calendar
        calendar.add_event(event);
      } else {
        // No date specified yet
        continue;
      }
    }

    // write calendar to file
    calendar.save_file(format!("{}/{}.ics", out_dir, person))?;
  }
  Ok(())
}

fn get_start_and_end_time(schedule_entry: &ScheduleEntry) -> Option<(String, String)> {
  if let Some(start_date_naive) = &schedule_entry.date {
    let (start_time, stop_time_opt) = &schedule_entry.start_stop_time;
    let start_date_time_naive =
      NaiveDateTime::new(start_date_naive.to_owned(), start_time.to_owned());

    let start_date = Zurich.from_local_datetime(&start_date_time_naive).unwrap(); // TODO: Better error handling start time
    let stop_date = if let Some(stop_time) = stop_time_opt {
      let stop_date_time_naive =
        NaiveDateTime::new(start_date_naive.to_owned(), stop_time.to_owned());

      Zurich.from_local_datetime(&stop_date_time_naive).unwrap() // TODO: Better error handling stop date
    } else {
      start_date + Duration::hours(DEFAULT_EVENT_DURATION_HOURS)
    };
    Some((
      start_date.naive_utc().format(ICAL_STR_FORMAT).to_string(),
      stop_date.naive_utc().format(ICAL_STR_FORMAT).to_string(),
    ))
  } else {
    None
  }
}
