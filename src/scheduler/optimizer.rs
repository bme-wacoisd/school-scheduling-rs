use crate::types::{CourseId, Section, StudentId};
use std::collections::{HashMap, HashSet};

/// Phase 5: Post-ILP optimization for section balancing
///
/// ILP maximizes assignments but ignores section balancing.
/// This phase attempts to move students between sections of the same course
/// to achieve more balanced enrollment.
pub fn optimize_section_balance(mut sections: Vec<Section>) -> Vec<Section> {
    const MAX_ITERATIONS: u32 = 100;

    // Build student schedules: student_id -> set of occupied periods
    let mut student_schedules: HashMap<StudentId, HashSet<(u8, u8)>> = HashMap::new();
    for section in &sections {
        let periods: HashSet<(u8, u8)> = section
            .periods
            .iter()
            .map(|p| (p.day, p.slot))
            .collect();

        for student_id in &section.enrolled_students {
            student_schedules
                .entry(student_id.clone())
                .or_default()
                .extend(periods.iter());
        }
    }

    // Group sections by course
    let sections_by_course: HashMap<CourseId, Vec<usize>> = {
        let mut map: HashMap<CourseId, Vec<usize>> = HashMap::new();
        for (idx, section) in sections.iter().enumerate() {
            map.entry(section.course_id.clone()).or_default().push(idx);
        }
        map
    };

    for _ in 0..MAX_ITERATIONS {
        let mut improved = false;

        for (_course_id, section_indices) in &sections_by_course {
            if section_indices.len() < 2 {
                continue;
            }

            // Find largest and smallest sections
            let (smallest_idx, largest_idx) = {
                let mut sorted_indices = section_indices.clone();
                sorted_indices.sort_by_key(|&idx| sections[idx].enrollment());

                let smallest = *sorted_indices.first().unwrap();
                let largest = *sorted_indices.last().unwrap();
                (smallest, largest)
            };

            let diff = sections[largest_idx].enrollment() as i32
                - sections[smallest_idx].enrollment() as i32;

            // Only balance if difference is significant
            if diff <= 1 {
                continue;
            }

            // Try to move a student from largest to smallest
            let students_to_try: Vec<StudentId> =
                sections[largest_idx].enrolled_students.clone();

            for student_id in students_to_try {
                if can_move_student(
                    &student_id,
                    largest_idx,
                    smallest_idx,
                    &sections,
                    &student_schedules,
                ) {
                    // Perform the move
                    move_student(
                        &student_id,
                        largest_idx,
                        smallest_idx,
                        &mut sections,
                        &mut student_schedules,
                    );
                    improved = true;
                    break;
                }
            }
        }

        if !improved {
            break;
        }
    }

    sections
}

/// Check if a student can be moved from one section to another
fn can_move_student(
    student_id: &StudentId,
    from_idx: usize,
    to_idx: usize,
    sections: &[Section],
    student_schedules: &HashMap<StudentId, HashSet<(u8, u8)>>,
) -> bool {
    let to_section = &sections[to_idx];

    // Check capacity
    if to_section.is_full() {
        return false;
    }

    // Check for time conflicts
    let to_periods: HashSet<(u8, u8)> = to_section
        .periods
        .iter()
        .map(|p| (p.day, p.slot))
        .collect();

    let from_periods: HashSet<(u8, u8)> = sections[from_idx]
        .periods
        .iter()
        .map(|p| (p.day, p.slot))
        .collect();

    if let Some(schedule) = student_schedules.get(student_id) {
        // Get periods used by student, excluding the section they're moving from
        let other_periods: HashSet<(u8, u8)> = schedule
            .iter()
            .filter(|p| !from_periods.contains(p))
            .copied()
            .collect();

        // Check if target section conflicts with other courses
        if to_periods.iter().any(|p| other_periods.contains(p)) {
            return false;
        }
    }

    true
}

/// Move a student from one section to another
fn move_student(
    student_id: &StudentId,
    from_idx: usize,
    to_idx: usize,
    sections: &mut [Section],
    student_schedules: &mut HashMap<StudentId, HashSet<(u8, u8)>>,
) {
    // Get periods
    let from_periods: HashSet<(u8, u8)> = sections[from_idx]
        .periods
        .iter()
        .map(|p| (p.day, p.slot))
        .collect();

    let to_periods: HashSet<(u8, u8)> = sections[to_idx]
        .periods
        .iter()
        .map(|p| (p.day, p.slot))
        .collect();

    // Update sections
    sections[from_idx].unenroll(student_id);
    sections[to_idx].enroll(student_id.clone());

    // Update student schedule
    if let Some(schedule) = student_schedules.get_mut(student_id) {
        for period in &from_periods {
            schedule.remove(period);
        }
        schedule.extend(to_periods);
    }
}

/// Calculate balance score for sections (lower is better)
pub fn calculate_balance_score(sections: &[Section]) -> f64 {
    // Group by course
    let mut by_course: HashMap<&CourseId, Vec<usize>> = HashMap::new();
    for section in sections {
        by_course
            .entry(&section.course_id)
            .or_default()
            .push(section.enrollment());
    }

    let mut total_variance = 0.0;
    let mut course_count = 0;

    for enrollments in by_course.values() {
        if enrollments.len() < 2 {
            continue;
        }

        let mean = enrollments.iter().sum::<usize>() as f64 / enrollments.len() as f64;
        let variance: f64 = enrollments
            .iter()
            .map(|&e| (e as f64 - mean).powi(2))
            .sum::<f64>()
            / enrollments.len() as f64;

        total_variance += variance;
        course_count += 1;
    }

    if course_count > 0 {
        total_variance / course_count as f64
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Period, SectionId};

    fn make_section(id: &str, course: &str, slot: u8, students: Vec<&str>) -> Section {
        Section {
            id: SectionId(id.to_string()),
            course_id: CourseId(course.to_string()),
            teacher_id: None,
            room_id: None,
            periods: (0..5).map(|d| Period::new(d, slot)).collect(),
            enrolled_students: students.into_iter().map(|s| StudentId(s.to_string())).collect(),
            capacity: 30,
        }
    }

    #[test]
    fn test_balances_sections() {
        let sections = vec![
            make_section(
                "math-1",
                "math",
                0,
                vec!["s1", "s2", "s3", "s4", "s5", "s6"],
            ),
            make_section("math-2", "math", 1, vec![]),
        ];

        let result = optimize_section_balance(sections);

        // Sections should be more balanced
        let enrollments: Vec<usize> = result.iter().map(|s| s.enrollment()).collect();
        let diff = (enrollments[0] as i32 - enrollments[1] as i32).abs();

        // Should be at most 1 apart
        assert!(diff <= 1, "Sections should be balanced, got {:?}", enrollments);
    }

    #[test]
    fn test_respects_time_conflicts() {
        // Student has another class at slot 1
        let sections = vec![
            make_section("math-1", "math", 0, vec!["s1"]),
            make_section("math-2", "math", 1, vec![]), // Would conflict with eng
            make_section("eng-1", "eng", 1, vec!["s1"]), // s1 is here
        ];

        // Build student schedules
        let student_schedules: HashMap<StudentId, HashSet<(u8, u8)>> = {
            let mut map = HashMap::new();
            // s1 is in math-1 (slot 0) and eng-1 (slot 1)
            map.insert(
                StudentId("s1".to_string()),
                (0..5).flat_map(|d| vec![(d, 0), (d, 1)]).collect(),
            );
            map
        };

        // s1 cannot move from math-1 to math-2 because of eng conflict
        let can_move = can_move_student(
            &StudentId("s1".to_string()),
            0, // from math-1
            1, // to math-2
            &sections,
            &student_schedules,
        );

        assert!(!can_move, "Should not allow move due to time conflict");
    }
}
