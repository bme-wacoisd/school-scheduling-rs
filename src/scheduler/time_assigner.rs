use crate::types::{Course, CourseId, Period, ScheduleConfig, Section, Teacher, TeacherId};
use std::collections::{HashMap, HashSet};

/// Grade-aware time slot tracker
struct GradeSlotTracker {
    /// grade -> slot -> count of courses at this slot
    usage: HashMap<u8, HashMap<u8, u32>>,
}

impl GradeSlotTracker {
    fn new() -> Self {
        Self {
            usage: HashMap::new(),
        }
    }

    fn record_usage(&mut self, grades: Option<&Vec<u8>>, slot: u8) {
        if let Some(grades) = grades {
            for grade in grades {
                *self
                    .usage
                    .entry(*grade)
                    .or_default()
                    .entry(slot)
                    .or_insert(0) += 1;
            }
        }
    }

    fn get_penalty(&self, grades: Option<&Vec<u8>>, slot: u8) -> u32 {
        if let Some(grades) = grades {
            grades
                .iter()
                .map(|g| {
                    self.usage
                        .get(g)
                        .and_then(|m| m.get(&slot))
                        .copied()
                        .unwrap_or(0)
                        * 500
                })
                .sum()
        } else {
            0
        }
    }
}

/// Phase 2: Assign time slots to sections (CRITICAL for schedule quality)
pub fn assign_time_slots(
    sections: &mut [Section],
    courses: &[Course],
    teachers: &[Teacher],
    config: &ScheduleConfig,
) {
    let course_map: HashMap<&CourseId, &Course> = courses.iter().map(|c| (&c.id, c)).collect();
    let teacher_map: HashMap<&TeacherId, &Teacher> = teachers.iter().map(|t| (&t.id, t)).collect();

    let mut teacher_schedules: HashMap<TeacherId, HashSet<u8>> = HashMap::new();
    let mut slot_usage: Vec<u32> = vec![0; config.periods_per_day as usize];
    let mut grade_tracker = GradeSlotTracker::new();

    // Collect section info without borrowing sections
    let section_info: Vec<(usize, CourseId, Option<TeacherId>)> = sections
        .iter()
        .enumerate()
        .map(|(idx, s)| (idx, s.course_id.clone(), s.teacher_id.clone()))
        .collect();

    // Group by course
    let mut sections_by_course: HashMap<CourseId, Vec<(usize, Option<TeacherId>)>> = HashMap::new();
    for (idx, course_id, teacher_id) in section_info {
        sections_by_course
            .entry(course_id)
            .or_default()
            .push((idx, teacher_id));
    }

    // Process courses - prioritize courses with grade restrictions
    let mut course_ids: Vec<CourseId> = sections_by_course.keys().cloned().collect();
    course_ids.sort_by_key(|cid| {
        let course = course_map.get(cid);
        match course.and_then(|c| c.grade_restrictions.as_ref()) {
            Some(grades) => (0, grades.len()), // Grade-restricted first, fewer grades = higher priority
            None => (1, 0),                     // Open courses last
        }
    });

    for course_id in course_ids {
        let course = match course_map.get(&course_id) {
            Some(c) => *c,
            None => continue,
        };

        let section_info_list = match sections_by_course.get(&course_id) {
            Some(list) => list.clone(),
            None => continue,
        };

        let mut course_used_slots: HashSet<u8> = HashSet::new();

        for (section_idx, teacher_id) in section_info_list {
            // Find best slot for this section
            let best_slot = find_best_slot(
                teacher_id.as_ref(),
                &teacher_map,
                &teacher_schedules,
                &course_used_slots,
                course.grade_restrictions.as_ref(),
                &slot_usage,
                &grade_tracker,
                config,
            );

            // Assign the slot
            let section = &mut sections[section_idx];

            // For simplicity, assign same slot each day (5-day schedule)
            for day in 0..config.days_per_week {
                section.periods.push(Period::new(day, best_slot));
            }

            // Update tracking
            if let Some(tid) = teacher_id {
                teacher_schedules.entry(tid).or_default().insert(best_slot);
            }
            slot_usage[best_slot as usize] += 1;
            course_used_slots.insert(best_slot);
            grade_tracker.record_usage(course.grade_restrictions.as_ref(), best_slot);
        }
    }
}

fn find_best_slot(
    teacher_id: Option<&TeacherId>,
    teacher_map: &HashMap<&TeacherId, &Teacher>,
    teacher_schedules: &HashMap<TeacherId, HashSet<u8>>,
    course_used_slots: &HashSet<u8>,
    grades: Option<&Vec<u8>>,
    slot_usage: &[u32],
    grade_tracker: &GradeSlotTracker,
    config: &ScheduleConfig,
) -> u8 {
    (0..config.periods_per_day)
        .filter(|&slot| {
            // Check teacher availability
            if let Some(tid) = teacher_id {
                // Teacher already teaching at this slot?
                if teacher_schedules
                    .get(tid)
                    .map(|s| s.contains(&slot))
                    .unwrap_or(false)
                {
                    return false;
                }
                // Teacher unavailable?
                if let Some(teacher) = teacher_map.get(tid) {
                    // Check if teacher is unavailable for any day at this slot
                    for day in 0..config.days_per_week {
                        if teacher.unavailable.contains(&Period::new(day, slot)) {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .min_by_key(|&slot| {
            let mut penalty = slot_usage[slot as usize];

            // Heavy penalty for reusing slot within same course
            if course_used_slots.contains(&slot) {
                penalty += 1000;
            }

            // Penalty for same-grade conflicts
            penalty += grade_tracker.get_penalty(grades, slot);

            penalty
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SectionId;

    #[test]
    fn test_different_sections_get_different_slots() {
        let courses = vec![Course {
            id: CourseId("math".to_string()),
            name: "Math".to_string(),
            max_students: 30,
            periods_per_week: 5,
            grade_restrictions: None,
            required_features: vec![],
            sections: 2,
        }];

        let teachers = vec![Teacher {
            id: TeacherId("t1".to_string()),
            name: "Teacher".to_string(),
            subjects: vec![CourseId("math".to_string())],
            max_sections: 5,
            unavailable: vec![],
        }];

        let mut sections = vec![
            Section::new(
                SectionId("math-1".to_string()),
                CourseId("math".to_string()),
                30,
            ),
            Section::new(
                SectionId("math-2".to_string()),
                CourseId("math".to_string()),
                30,
            ),
        ];
        sections[0].teacher_id = Some(TeacherId("t1".to_string()));
        sections[1].teacher_id = Some(TeacherId("t1".to_string()));

        let config = ScheduleConfig::default();
        assign_time_slots(&mut sections, &courses, &teachers, &config);

        // Sections should have different time slots
        let slot_0 = sections[0].periods.first().map(|p| p.slot);
        let slot_1 = sections[1].periods.first().map(|p| p.slot);

        assert!(slot_0.is_some());
        assert!(slot_1.is_some());
        assert_ne!(slot_0, slot_1, "Same course sections should get different slots");
    }

    #[test]
    fn test_grade_restricted_courses_avoid_conflicts() {
        let courses = vec![
            Course {
                id: CourseId("gov".to_string()),
                name: "Government".to_string(),
                max_students: 30,
                periods_per_week: 5,
                grade_restrictions: Some(vec![12]),
                required_features: vec![],
                sections: 1,
            },
            Course {
                id: CourseId("eng12".to_string()),
                name: "English 12".to_string(),
                max_students: 30,
                periods_per_week: 5,
                grade_restrictions: Some(vec![12]),
                required_features: vec![],
                sections: 1,
            },
        ];

        let teachers = vec![
            Teacher {
                id: TeacherId("t1".to_string()),
                name: "Teacher 1".to_string(),
                subjects: vec![CourseId("gov".to_string())],
                max_sections: 5,
                unavailable: vec![],
            },
            Teacher {
                id: TeacherId("t2".to_string()),
                name: "Teacher 2".to_string(),
                subjects: vec![CourseId("eng12".to_string())],
                max_sections: 5,
                unavailable: vec![],
            },
        ];

        let mut sections = vec![
            Section::new(
                SectionId("gov-1".to_string()),
                CourseId("gov".to_string()),
                30,
            ),
            Section::new(
                SectionId("eng12-1".to_string()),
                CourseId("eng12".to_string()),
                30,
            ),
        ];
        sections[0].teacher_id = Some(TeacherId("t1".to_string()));
        sections[1].teacher_id = Some(TeacherId("t2".to_string()));

        let config = ScheduleConfig::default();
        assign_time_slots(&mut sections, &courses, &teachers, &config);

        // 12th grade required courses should get different slots
        let gov_slot = sections[0].periods.first().map(|p| p.slot);
        let eng_slot = sections[1].periods.first().map(|p| p.slot);

        assert_ne!(
            gov_slot, eng_slot,
            "Same-grade required courses should get different slots"
        );
    }
}
