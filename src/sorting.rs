use crate::structures::{Person, PersonToSceneAndScheduleEntry, SceneEntry, Scenes, ScheduleEntry};
use chrono::NaiveDate;
use std::collections::HashSet;

pub fn get_schedule_to_scene_entry<'a>(
  schedule_entries: &'a Vec<ScheduleEntry>,
  scene_entries: &'a Vec<SceneEntry>,
) -> Vec<(&'a ScheduleEntry, Option<&'a SceneEntry>)> {
  let mut schedule_to_scene_entries = vec![];
  for schedule_entry in schedule_entries {
    match &schedule_entry.scenes {
      Scenes::Special(_) => {
        // When it is a special scene, all scenes are played or not yet known
        schedule_to_scene_entries.push((schedule_entry, None));
        continue;
      }
      Scenes::Normal(scenes) => {
        if scenes.is_empty() {
          // When no scenes are specified, all scenes are played or not yet known
          schedule_to_scene_entries.push((schedule_entry, None));
        } else {
          for scene_entry in scene_entries {
            if scene_entry.scenes.iter().any(|s| scenes.contains(s)) {
              schedule_to_scene_entries.push((schedule_entry, Some(scene_entry)));
            } else {
              //TODO: What to do if no match is found?
              println!("No match for scene entry: {:?}", scene_entry);
            }
          }
        }
      }
    }
  }
  schedule_to_scene_entries
}

pub fn get_person_to_scene_and_schedule_entry<'a>(
  schedule_to_scene_entries: &'a [(&'a ScheduleEntry, Option<&'a SceneEntry>)],
) -> PersonToSceneAndScheduleEntry<'a> {
  let all_persons = schedule_to_scene_entries
    .iter()
    .filter_map(|(_, scene_entry)| scene_entry.map(|x| x.who.to_owned()))
    .collect::<HashSet<Person>>();
  let mut person_to_scene_and_schedule_entry = vec![];
  for person in all_persons {
    let schedule_entries_for_person = schedule_to_scene_entries
      .iter()
      .filter(|(_, scene_entry)| {
        if let Some(scene_entry) = scene_entry {
          scene_entry.who == person
        } else {
          // if no scene entry this means that all scenes will be played
          true
        }
      })
      .collect();
    person_to_scene_and_schedule_entry.push((person, schedule_entries_for_person));
  }
  person_to_scene_and_schedule_entry
}

pub fn filter_by_silent_play<'a>(
  schedule_to_scene_entries: &'a Vec<(&ScheduleEntry, Option<&SceneEntry>)>,
  mandatory_silent_play: &'a NaiveDate,
) -> Vec<(&'a ScheduleEntry, Option<&'a SceneEntry>)> {
  let mut filtered_schedule_to_scene_entries = vec![];
  for (schedule_entry, scene_entry) in schedule_to_scene_entries {
    match &schedule_entry.scenes {
      Scenes::Special(_) => {
        filtered_schedule_to_scene_entries.push((*schedule_entry, *scene_entry));
        continue;
      }
      Scenes::Normal(scenes) => {
        if scenes.is_empty() {
          // When no scenes are specified, all scenes are played or not yet known
          filtered_schedule_to_scene_entries.push((*schedule_entry, *scene_entry));
          continue;
        }
        let any_non_silent_play = scenes.iter().any(|scene| {
          if let Some(scene_entry) = scene_entry {
            scene_entry.is_scene_silent_play(scene) == Some(false)
          } else {
            // if scene entry is not known, assume it is not silent play
            true
          }
        });
        if any_non_silent_play {
          filtered_schedule_to_scene_entries.push((*schedule_entry, *scene_entry));
        } else {
          if schedule_entry.date >= *mandatory_silent_play {
            filtered_schedule_to_scene_entries.push((*schedule_entry, *scene_entry));
          }
        }
      }
    }
  }
  filtered_schedule_to_scene_entries
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::structures::{SceneEntry, ScheduleEntry};
  use chrono::NaiveTime;

  fn mandatory_silent_play() -> NaiveDate {
    NaiveDate::from_ymd_opt(2022, 6, 1).unwrap()
  }

  fn test_data() -> (Vec<ScheduleEntry>, Vec<SceneEntry>) {
    let schedule_entries: Vec<ScheduleEntry> = vec![
      ScheduleEntry::new(
        NaiveDate::from_ymd_opt(2022, 5, 1).unwrap(),
        (NaiveTime::from_hms_opt(10, 0, 0).unwrap(), None),
        Scenes::Normal(vec!["Scene 3".to_string()]),
        Some("Room 1".to_string()),
        None,
      ),
      ScheduleEntry::new(
        NaiveDate::from_ymd_opt(2022, 7, 1).unwrap(),
        (NaiveTime::from_hms_opt(10, 0, 0).unwrap(), None),
        Scenes::Normal(vec!["Scene 4".to_string(), "Scene 5".to_string()]),
        None,
        None,
      ),
      ScheduleEntry::new(
        NaiveDate::from_ymd_opt(2022, 8, 1).unwrap(),
        (NaiveTime::from_hms_opt(10, 0, 0).unwrap(), None),
        Scenes::Normal(vec![]),
        None,
        None,
      ),
      ScheduleEntry::new(
        NaiveDate::from_ymd_opt(2022, 4, 1).unwrap(),
        (NaiveTime::from_hms_opt(10, 0, 0).unwrap(), None),
        Scenes::Normal(vec![]),
        None,
        None,
      ),
    ];
    let scene_entries = vec![
      SceneEntry {
        role: "Role 1".to_string(),
        who: "Person 1".to_string(),
        scenes: vec![
          "Scene 1".to_string(),
          "Scene 2".to_string(),
          "Scene 5".to_string(),
        ],
        silent_play: vec![false, true, false],
      },
      SceneEntry {
        role: "Role 2".to_string(),
        who: "Person 2".to_string(),
        scenes: vec!["Scene 3".to_string()],
        silent_play: vec![true],
      },
      SceneEntry {
        role: "Role 3".to_string(),
        who: "Person 2".to_string(),
        scenes: vec!["Scene 3".to_string(), "Scene 5".to_string()],
        silent_play: vec![true, true],
      },
      SceneEntry {
        role: "Role 4".to_string(),
        who: "Person 3".to_string(),
        scenes: vec!["Scene 4".to_string()],
        silent_play: vec![true],
      },
    ];
    (schedule_entries, scene_entries)
  }

  #[test]
  fn test_get_schedule_to_scene_entry() {
    let (schedule_entries, scene_entries) = test_data();
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    assert_eq!(schedule_to_scene_entries.len(), 7, "Should have 7 entries.",);
    for (schedule_entry, scene_entry) in schedule_to_scene_entries {
      if let Some(scene_entry) = scene_entry {
        match &schedule_entry.scenes {
          Scenes::Special(_) => {}
          Scenes::Normal(scenes) => {
            assert!(
              scenes
                .iter()
                .any(|scene| scene_entry.scenes.contains(scene)),
              "Some scene of schedule entry {:?} should be in scene entry {:?}",
              schedule_entry.scenes,
              scene_entry.scenes,
            );
          }
        }
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
      3,
      "Should have 3 entries for each person",
    );
    for (person, schedule_entries_for_person) in person_to_scene_and_schedule_entry {
      for (schedule_entry, scene_entry) in schedule_entries_for_person {
        if let Some(scene_entry) = scene_entry {
          assert_eq!(
            person, scene_entry.who,
            "Person {:?} should be the same as the person in the scene entry {:?}",
            person, scene_entry
          );
          if let Scenes::Normal(ref scenes) = schedule_entry.scenes {
            assert!(
              scenes
                .iter()
                .any(|scene| scene_entry.scenes.contains(scene)),
              "Some scene of schedule entry {:?} should be in scene entry {:?}",
              schedule_entry.scenes,
              scene_entry.scenes,
            );
          }
        }
      }
    }
  }

  #[test]
  fn test_filter_by_silent_play() {
    let (schedule_entries, scene_entries) = test_data();
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    let mandatory_silent_play = mandatory_silent_play();
    let filtered_schedule_to_scene_entries =
      filter_by_silent_play(&schedule_to_scene_entries, &mandatory_silent_play);
    assert_eq!(
      filtered_schedule_to_scene_entries.len(),
      5,
      "Should have 5 entries for each scene",
    );
  }
}
