use calamine::{open_workbook, DataType, Range, Reader, Xlsx};
// use chrono::format::ParseError;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use ics::properties::{Description, DtEnd, DtStart, Status, Summary};
use ics::{escape_text, Event, ICalendar};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::path::Path;

type Person = String;

#[derive(Debug)]
pub struct ExcelParseError {
    file: String,
    sheet: usize,
    row: usize,
    column: usize,
    token: String,
    expected: String,
}
impl fmt::Display for ExcelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parsing error for file '{}' in sheet number '{}' (row {}, column {}). {}. Unexpected token '{}'.", self.file, self.sheet, self.row, self.column, self.expected,self.token)
    }
}

impl Error for ExcelParseError {}

#[derive(Debug)]
pub struct SceneEntry {
    pub role: String,
    pub who: Person,
    pub scenes: Vec<String>,
    pub silent_play: Vec<bool>,
}

#[derive(Debug)]
pub struct ScheduleEntry {
    pub date: Option<NaiveDate>,
    pub start_stop_time: (NaiveTime, Option<NaiveTime>),
    pub scenes: Vec<String>,
    pub uuid: md5::Digest,
}

pub fn get_schedule_entry(date: &String, time: &String, scenes: &String) -> ScheduleEntry {
    dbg!(date, time, scenes);
    ScheduleEntry {
        date: parse_date(date),
        start_stop_time: parse_time(time),
        scenes: parse_scenes(scenes),
        uuid: md5::compute(date.to_owned() + time.as_str() + scenes.as_str()),
    }
}

fn parse_date(date: &String) -> Option<NaiveDate> {
    if !["Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa.", "So."]
        .iter()
        .any(|s| date.contains(s))
    {
        return None;
    }
    let english_date = change_german_days_to_englisch(date);
    Some(NaiveDate::parse_from_str(english_date.trim(), "%a. %_d.%_m.%y").unwrap())
}

fn change_german_days_to_englisch(date: &String) -> String {
    if date.contains("Mo") {
        date.replace("Mo", "Mon")
    } else if date.contains("Di") {
        date.replace("Di", "Tue")
    } else if date.contains("Mi") {
        date.replace("Mi", "Wed")
    } else if date.contains("Do") {
        date.replace("Do", "Thu")
    } else if date.contains("Fr") {
        date.replace("Fr", "Fri")
    } else if date.contains("Sa") {
        date.replace("Sa", "Sat")
    } else if date.contains("So") {
        date.replace("So", "Sun")
    } else {
        date.clone()
    }
}

fn parse_time(time: &String) -> (NaiveTime, Option<NaiveTime>) {
    if time.contains("-") {
        if let [start, stop] = time.split('-').collect::<Vec<&str>>().as_slice() {
            (
                NaiveTime::parse_from_str(start.trim(), "%H:%M").unwrap(), // TODO: Add error handling
                Some(NaiveTime::parse_from_str(stop.trim(), "%H:%M").unwrap()), // TODO: Add error handling
            )
        } else {
            panic!() // TODO: Add error handling
        }
    } else {
        (
            NaiveTime::parse_from_str(time.trim(), "%H:%M").unwrap(), // TODO: Add error handling
            None,
        )
    }
}

fn add_corresponding_stop_time(schedule_entries: Vec<ScheduleEntry>) -> Vec<ScheduleEntry> {
    if schedule_entries.len() <= 1 {
        return schedule_entries;
    }

    dbg!(schedule_entries.len());
    let mut previous_start_time = schedule_entries.last().unwrap().start_stop_time.0.clone();
    let mut previous_date = schedule_entries.last().unwrap().date.clone();
    let mut new_schedule_entries = vec![];

    for (i, entry) in schedule_entries.into_iter().rev().enumerate() {
        let previous_date_tmp = entry.date.clone();
        let previous_start_time_tmp = entry.start_stop_time.0.clone();
        if !previous_date.eq(&entry.date) || i == 0 {
            new_schedule_entries.push(entry);
        } else {
            if entry.start_stop_time.1.is_none() {
                new_schedule_entries.push(ScheduleEntry {
                    start_stop_time: (entry.start_stop_time.0, Some(previous_start_time)),
                    ..entry
                })
            } else {
                new_schedule_entries.push(entry);
            }
        }
        previous_date = previous_date_tmp;
        previous_start_time = previous_start_time_tmp;
    }
    dbg!(new_schedule_entries.len());

    new_schedule_entries.reverse();
    new_schedule_entries
}

fn parse_scenes(scenes: &String) -> Vec<String> {
    scenes.split("/").map(|s| s.trim().to_owned()).collect()
}

pub fn read_excel(path: &str, sheet_num: usize) -> Result<Range<DataType>, calamine::Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range_at(sheet_num)
        .ok_or(calamine::Error::Msg("Cannot find sheet."))??;
    Ok(range)
}

fn get_schedule_to_scene_entry<'a>(
    schedule_entries: &'a Vec<ScheduleEntry>,
    scene_entries: &'a Vec<SceneEntry>,
) -> Vec<(&'a ScheduleEntry, &'a SceneEntry)> {
    let mut schedule_to_scene_entries = vec![];
    for schedule_entry in schedule_entries {
        for scene_entry in scene_entries {
            if scene_entry
                .scenes
                .iter()
                .any(|s| schedule_entry.scenes.contains(s))
            {
                schedule_to_scene_entries.push((schedule_entry, scene_entry));
            }
        }
    }
    schedule_to_scene_entries
}

pub fn parse_schedule_plan_content(
    excel_range: &Range<DataType>,
) -> Result<Vec<ScheduleEntry>, ExcelParseError> {
    let mut start_parsing = false;
    let mut previous_date: Option<String> = None;
    let mut schedule_entries = vec![];
    for row in excel_range.rows() {
        if row.len() != 3 {
            todo!("Add parser error when more than 3 rows."); // TODO:
        }
        let (opt_date, times, scenes) = match row {
            [DataType::String(x), DataType::String(y), DataType::String(z)] => {
                (Some(x.to_owned()), y.to_owned(), z.to_owned())
            }
            [DataType::String(x), DataType::String(y), DataType::Float(z)] => {
                (Some(x.to_owned()), y.to_owned(), z.to_string())
            }
            [DataType::Empty, DataType::String(y), DataType::Float(z)] => {
                (None, y.to_owned(), z.to_string())
            }
            [DataType::Empty, DataType::String(y), DataType::String(z)] => {
                (None, y.to_owned(), z.to_owned())
            }
            _ => {
                todo!("Add parser error when not only strings. Row: {row:?}")
            }
        };

        if !start_parsing {
            if !times.eq(&String::from("Zeit")) {
                continue;
            } else {
                start_parsing = true;
                continue;
            }
        }
        if let Some(date) = &opt_date {
            previous_date = Some(date.clone());
            schedule_entries.push(get_schedule_entry(&date, &times, &scenes));
        } else if let Some(date) = &previous_date {
            schedule_entries.push(get_schedule_entry(&date, &times, &scenes));
        } else {
            panic!("No date found")
        }
    }
    Ok(add_corresponding_stop_time(schedule_entries))
}

fn parse_scene_plan_content(
    excel_range: Range<DataType>,
) -> Result<Vec<SceneEntry>, ExcelParseError> {
    let mut all_scenes = vec![];
    let mut scene_entries = vec![];
    let scene_start_index = 2;
    for (i, row) in excel_range.rows().enumerate() {
        if i == 0 {
            for scene in &row[scene_start_index..] {
                match scene {
                    DataType::String(x) => all_scenes.push(x.to_owned()),
                    DataType::Float(x) => all_scenes.push(x.to_string()),
                    _ => todo!("Error handling all scenes"), // TODO: Error handling all scenes
                }
            }
            dbg!(&all_scenes);
        } else {
            let role = match &row[0] {
                DataType::String(x) => x.clone(),
                _ => todo!("Error handling role"), // TODO: Error handling role
            };
            let who = match &row[1] {
                DataType::String(x) => x.clone(),
                _ => todo!("Error handling who"), // TODO: Error handling who
            };
            let mut scenes_for_current_role = vec![];
            let mut silent_play = vec![];
            for (i, scene) in row[scene_start_index..].iter().enumerate() {
                if let DataType::String(mark) = scene {
                    if mark.contains("x") {
                        scenes_for_current_role.push(all_scenes[i].clone());
                        silent_play.push(false);
                    } else if mark.contains("-") {
                        scenes_for_current_role.push(all_scenes[i].clone());
                        silent_play.push(true);
                    }
                }
            }
            scene_entries.push(SceneEntry {
                role,
                who,
                scenes: scenes_for_current_role,
                silent_play,
            })
        }
    }
    Ok(scene_entries)
}
fn get_person_to_scene_and_schedule_entry<'a>(
    schedule_to_scene_entries: &'a Vec<(&'a ScheduleEntry, &'a SceneEntry)>,
) -> Vec<(Person, Vec<&'a (&'a ScheduleEntry, &'a SceneEntry)>)> {
    let all_persons = schedule_to_scene_entries
        .iter()
        .map(|(_, scene_entry)| scene_entry.who.to_owned())
        .collect::<HashSet<Person>>();
    dbg!(&all_persons);
    let mut person_to_scene_and_schedule_entry = vec![];
    for person in all_persons {
        let schedule_entries_for_person = schedule_to_scene_entries
            .into_iter()
            .filter(|(_, scene_entry)| scene_entry.who == person)
            .collect();
        person_to_scene_and_schedule_entry.push((person, schedule_entries_for_person));
    }
    person_to_scene_and_schedule_entry
}

use chrono::Duration;
use chrono_tz::Europe::Zurich;

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
        dbg!(Some((
            start_date.naive_utc().format(ICAL_STR_FORMAT).to_string(),
            stop_date.naive_utc().format(ICAL_STR_FORMAT).to_string(),
        )))
    } else {
        None
    }
}

fn write_ics_file(
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
            if let Some((start_date_time, stop_date_time)) = get_start_and_end_time(&schedule_entry)
            {
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

const ICAL_STR_FORMAT: &str = "%Y%m%dT%H%M%SZ";
const DEFAULT_EVENT_DURATION_HOURS: i64 = 4;

fn main() -> Result<(), Box<dyn Error>> {
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
    dbg!(&schedule_entries);
    let scene_sheet_num = 1;
    let scene_excel_range = read_excel(test_file_path_str, scene_sheet_num)?;
    let scene_entries = parse_scene_plan_content(scene_excel_range)?;
    dbg!(&scene_entries);
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    dbg!(&schedule_to_scene_entries);
    let person_to_schedule_and_scene_entries =
        get_person_to_scene_and_schedule_entry(&schedule_to_scene_entries);

    dbg!(&person_to_schedule_and_scene_entries);
    let out_dir = String::from("ics");
    write_ics_file(&person_to_schedule_and_scene_entries, &out_dir)?;

    Ok(())
}
