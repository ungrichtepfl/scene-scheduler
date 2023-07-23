use crate::structures::{Person, SceneEntry, ScheduleEntry};
use chrono::NaiveDate;
use std::collections::HashSet;

pub fn get_schedule_to_scene_entry<'a>(
  schedule_entries: &'a Vec<ScheduleEntry>,
  scene_entries: &'a Vec<SceneEntry>,
) -> Vec<(&'a ScheduleEntry, Option<&'a SceneEntry>)> {
  let mut schedule_to_scene_entries = vec![];
  for schedule_entry in schedule_entries {
    if schedule_entry.scenes.is_empty() {
      // When no scenes are specified, all scenes are played or not yet known
      schedule_to_scene_entries.push((schedule_entry, None));
      continue;
    }
    for scene_entry in scene_entries {
      if scene_entry
        .scenes
        .iter()
        .any(|s| schedule_entry.scenes.contains(s))
      {
        schedule_to_scene_entries.push((schedule_entry, Some(scene_entry)));
      } else {
        //TODO: What to do if no match is found?
        println!("No match for scene entry: {:?}", scene_entry);
      }
    }
  }
  schedule_to_scene_entries
}

pub fn get_person_to_scene_and_schedule_entry<'a>(
  schedule_to_scene_entries: &'a Vec<(&'a ScheduleEntry, Option<&'a SceneEntry>)>,
) -> Vec<(Person, Vec<&'a (&'a ScheduleEntry, Option<&'a SceneEntry>)>)> {
  let all_persons = schedule_to_scene_entries
    .iter()
    .filter_map(|(_, scene_entry)| scene_entry.map(|x| x.who.to_owned()))
    .collect::<HashSet<Person>>();
  let mut person_to_scene_and_schedule_entry = vec![];
  for person in all_persons {
    let schedule_entries_for_person = schedule_to_scene_entries
      .into_iter()
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
    let any_non_silent_play = schedule_entry.scenes.iter().any(|scene| {
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
      if let Some(silent_play_date) = &schedule_entry.date {
        if *silent_play_date >= *mandatory_silent_play {
          filtered_schedule_to_scene_entries.push((*schedule_entry, *scene_entry));
        }
      } else {
        // if date is not set, keep the entry
        filtered_schedule_to_scene_entries.push((*schedule_entry, *scene_entry));
      }
    }
  }
  filtered_schedule_to_scene_entries
}

pub fn filter_by_non_empty_schedule_entry_date<'a>(
  schedule_to_scene_entries: &'a Vec<(&ScheduleEntry, Option<&SceneEntry>)>,
) -> Vec<(&'a ScheduleEntry, Option<&'a SceneEntry>)> {
  schedule_to_scene_entries
    .into_iter()
    .filter_map(|(schedule_entry, scene_entry)| {
      if !schedule_entry.date.is_none() {
        Some((*schedule_entry, *scene_entry))
      } else {
        None
      }
    })
    .collect()
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
        None,
        (NaiveTime::from_hms_opt(10, 0, 0).unwrap(), None),
        vec!["Scene 1".to_string(), "Scene 2".to_string()],
        Some("Room 1".to_string()),
        None,
      ),
      ScheduleEntry::new(
        Some(NaiveDate::from_ymd_opt(2022, 5, 1).unwrap()),
        (NaiveTime::from_hms_opt(10, 0, 0).unwrap(), None),
        vec!["Scene 3".to_string()],
        Some("Room 1".to_string()),
        None,
      ),
      ScheduleEntry::new(
        Some(NaiveDate::from_ymd_opt(2022, 7, 1).unwrap()),
        (NaiveTime::from_hms_opt(10, 0, 0).unwrap(), None),
        vec!["Scene 4".to_string(), "Scene 5".to_string()],
        None,
        None,
      ),
      // ScheduleEntry::new(
      //   Some(NaiveDate::from_ymd(2022, 8, 1)),
      //   (NaiveTime::from_hms(10, 0, 0), None),
      //   vec![],
      //   None,
      //   None,// TODO: Add test for this
      // ),
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
    assert_eq!(schedule_to_scene_entries.len(), 6, "Should have 6 entries.",);
    for (schedule_entry, scene_entry) in schedule_to_scene_entries {
      if let Some(scene_entry) = scene_entry {
        assert!(
          schedule_entry
            .scenes
            .iter()
            .any(|scene| scene_entry.scenes.contains(scene)),
          "Some scene of schedule entry {:?} should be in scene entry {:?}",
          schedule_entry.scenes,
          scene_entry.scenes,
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
          assert!(
            schedule_entry
              .scenes
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

  #[test]
  fn test_filter_by_silent_play() {
    let (schedule_entries, scene_entries) = test_data();
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    let mandatory_silent_play = mandatory_silent_play();
    let filtered_schedule_to_scene_entries =
      filter_by_silent_play(&schedule_to_scene_entries, &mandatory_silent_play);
    assert_eq!(
      filtered_schedule_to_scene_entries.len(),
      4,
      "Should have 4 entries for each scene",
    );
  }

  #[test]
  fn test_filter_by_non_empty_schedule_entry_date() {
    let (schedule_entries, scene_entries) = test_data();
    let schedule_to_scene_entries = get_schedule_to_scene_entry(&schedule_entries, &scene_entries);
    let filtered_schedule_to_scene_entries =
      filter_by_non_empty_schedule_entry_date(&schedule_to_scene_entries);
    assert_eq!(
      filtered_schedule_to_scene_entries.len(),
      5,
      "Should have 5 entries for each scene",
    );
  }
}
