use crate::types::{Course, CourseId, Period, Room, RoomId, Section};
use std::collections::{HashMap, HashSet};

/// Phase 3: Assign rooms to sections
pub fn assign_rooms(sections: &mut [Section], rooms: &[Room], courses: &[Course]) {
    let course_map: HashMap<&CourseId, &Course> = courses.iter().map(|c| (&c.id, c)).collect();

    // Track room schedules: room_id -> set of occupied periods
    let mut room_schedules: HashMap<&RoomId, HashSet<Period>> = HashMap::new();

    // Sort rooms by capacity (smallest first) for efficient packing
    let mut sorted_rooms: Vec<&Room> = rooms.iter().collect();
    sorted_rooms.sort_by_key(|r| r.capacity);

    // Sort sections by room requirements (most constrained first)
    let mut section_indices: Vec<usize> = (0..sections.len()).collect();
    section_indices.sort_by_key(|&idx| {
        let section = &sections[idx];
        let course = course_map.get(&section.course_id);
        let feature_count = course
            .map(|c| c.required_features.len())
            .unwrap_or(0);
        // More features = more constrained = process first
        std::cmp::Reverse(feature_count)
    });

    for section_idx in section_indices {
        let section = &sections[section_idx];
        let course = course_map.get(&section.course_id);
        let required_features: &[String] = course
            .map(|c| c.required_features.as_slice())
            .unwrap_or(&[]);

        // Find suitable room
        let assigned_room = find_suitable_room(
            section,
            &sorted_rooms,
            required_features,
            &room_schedules,
        );

        if let Some(room) = assigned_room {
            // Update section
            sections[section_idx].room_id = Some(room.id.clone());

            // Update room schedule
            let schedule = room_schedules.entry(&room.id).or_default();
            for period in &sections[section_idx].periods {
                schedule.insert(*period);
            }
        }
    }
}

fn find_suitable_room<'a>(
    section: &Section,
    rooms: &[&'a Room],
    required_features: &[String],
    room_schedules: &HashMap<&RoomId, HashSet<Period>>,
) -> Option<&'a Room> {
    for room in rooms {
        // Check capacity
        if room.capacity < section.capacity {
            continue;
        }

        // Check features
        if !room.has_features(required_features) {
            continue;
        }

        // Check availability
        let schedule = room_schedules.get(&room.id);
        let available = section.periods.iter().all(|period| {
            // Room not booked at this time
            !schedule.map(|s| s.contains(period)).unwrap_or(false)
                // And room is not marked unavailable
                && room.is_available(period)
        });

        if available {
            return Some(room);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Period, SectionId};

    #[test]
    fn test_assigns_rooms_respecting_capacity() {
        let courses = vec![Course {
            id: CourseId("math".to_string()),
            name: "Math".to_string(),
            max_students: 25,
            periods_per_week: 5,
            grade_restrictions: None,
            required_features: vec![],
            sections: 1,
        }];

        let rooms = vec![
            Room {
                id: RoomId("small".to_string()),
                name: "Small Room".to_string(),
                capacity: 20,
                features: vec![],
                unavailable: vec![],
            },
            Room {
                id: RoomId("medium".to_string()),
                name: "Medium Room".to_string(),
                capacity: 30,
                features: vec![],
                unavailable: vec![],
            },
        ];

        let mut sections = vec![Section {
            id: SectionId("math-1".to_string()),
            course_id: CourseId("math".to_string()),
            teacher_id: None,
            room_id: None,
            periods: vec![Period::new(0, 0)],
            enrolled_students: vec![],
            capacity: 25,
        }];

        assign_rooms(&mut sections, &rooms, &courses);

        // Should assign medium room (small is too small)
        assert_eq!(
            sections[0].room_id,
            Some(RoomId("medium".to_string()))
        );
    }

    #[test]
    fn test_respects_room_features() {
        let courses = vec![Course {
            id: CourseId("chem".to_string()),
            name: "Chemistry".to_string(),
            max_students: 25,
            periods_per_week: 5,
            grade_restrictions: None,
            required_features: vec!["lab".to_string()],
            sections: 1,
        }];

        let rooms = vec![
            Room {
                id: RoomId("regular".to_string()),
                name: "Regular Room".to_string(),
                capacity: 30,
                features: vec![],
                unavailable: vec![],
            },
            Room {
                id: RoomId("lab".to_string()),
                name: "Science Lab".to_string(),
                capacity: 30,
                features: vec!["lab".to_string()],
                unavailable: vec![],
            },
        ];

        let mut sections = vec![Section {
            id: SectionId("chem-1".to_string()),
            course_id: CourseId("chem".to_string()),
            teacher_id: None,
            room_id: None,
            periods: vec![Period::new(0, 0)],
            enrolled_students: vec![],
            capacity: 25,
        }];

        assign_rooms(&mut sections, &rooms, &courses);

        // Should assign lab room
        assert_eq!(sections[0].room_id, Some(RoomId("lab".to_string())));
    }
}
