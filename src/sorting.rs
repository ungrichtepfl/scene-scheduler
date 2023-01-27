use crate::structures::{Person, SceneEntry, ScheduleEntry};

use std::collections::HashSet;

pub fn get_schedule_to_scene_entry<'a>(
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

pub fn get_person_to_scene_and_schedule_entry<'a>(
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
