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
      } else {
        //TODO: What to do if no match is found?
        println!("No match for scene entry: {:?}", scene_entry);
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::structures::{SceneEntry, ScheduleEntry};
  use chrono::NaiveTime;

  fn test_data() -> (Vec<ScheduleEntry>, Vec<SceneEntry>) {
    let schedule_entries: Vec<ScheduleEntry> = vec![
      ScheduleEntry::new(
        None,
        (NaiveTime::from_hms(10, 0, 0), None),
        vec!["Scene 1".to_string(), "Scene 2".to_string()],
      ),
      ScheduleEntry::new(
        None,
        (NaiveTime::from_hms(10, 0, 0), None),
        vec!["Scene 3".to_string()],
      ),
    ];
    let scene_entries = vec![
      SceneEntry {
        role: "Role 1".to_string(),
        who: "Person 1".to_string(),
        scenes: vec!["Scene 1".to_string(), "Scene 2".to_string()],
        silent_play: vec![false, false],
      },
      SceneEntry {
        role: "Role 2".to_string(),
        who: "Person 2".to_string(),
        scenes: vec!["Scene 3".to_string()],
        silent_play: vec![false],
      },
      SceneEntry {
        role: "Role 3".to_string(),
        who: "Person 2".to_string(),
        scenes: vec!["Scene 3".to_string()],
        silent_play: vec![false],
      },
    ];
    (schedule_entries, scene_entries)
  }

  #[test]
  fn test_get_schedule_to_scene_entry() {
    let (schedule_entries, scene_entries) = test_data();
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    assert_eq!(
      schedule_to_scene_entries.len(),
      3,
      "Should have 3 entries for each scene",
    );
    for (schedule_entry, scene_entry) in schedule_to_scene_entries {
      for scene in &scene_entry.scenes {
        assert!(
          schedule_entry.scenes.contains(scene),
          "Schedule entry {:?} should contain scene {:?}",
          schedule_entry,
          scene
        );
      }
    }
  }

  #[test]
  fn test_get_person_to_scene_and_schedule_entry() {
    let (schedule_entries, scene_entries) = test_data();
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    let person_to_scene_and_schedule_entry =
      get_person_to_scene_and_schedule_entry(&schedule_to_scene_entries);
    assert_eq!(
      person_to_scene_and_schedule_entry.len(),
      2,
      "Should have 2 entries for each person",
    );
    for (person, schedule_entries_for_person) in person_to_scene_and_schedule_entry {
      for (schedule_entry, scene_entry) in schedule_entries_for_person {
        assert_eq!(
          person, scene_entry.who,
          "Person {:?} should be the same as the person in the scene entry {:?}",
          person, scene_entry
        );
        for scene in &scene_entry.scenes {
          assert!(
            schedule_entry.scenes.contains(scene),
            "Schedule entry {:?} should contain scene {:?}",
            schedule_entry,
            scene
          );
        }
      }
    }
  }
}
