use crate::error::Result;
use crate::types::{Course, CourseId, Section, Student, UnassignedCourse};
use good_lp::{constraint, variable, variables, Expression, Solution, SolverModel};
use indicatif::ProgressBar;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// Phase 4: ILP-based student assignment
///
/// Maximize: Σ(1000 * required_assignment) + Σ((10-rank) * elective_assignment)
/// Subject to:
///   - capacity constraints (hard)
///   - time conflict constraints (hard)
///   - at most one section per course per student (hard)
pub fn solve_student_assignment(
    mut sections: Vec<Section>,
    students: &[Student],
    courses: &[Course],
    progress: &ProgressBar,
) -> Result<(Vec<Section>, Vec<UnassignedCourse>)> {
    // Use BTreeMap for deterministic iteration order
    let course_map: BTreeMap<&CourseId, &Course> = courses.iter().map(|c| (&c.id, c)).collect();

    // Build section lookup structures (deterministic order)
    let section_indices: BTreeMap<&CourseId, Vec<usize>> = {
        let mut map: BTreeMap<&CourseId, Vec<usize>> = BTreeMap::new();
        for (idx, section) in sections.iter().enumerate() {
            map.entry(&section.course_id).or_default().push(idx);
        }
        map
    };

    // Build section period lookup for conflict detection
    let section_periods: Vec<HashSet<(u8, u8)>> = sections
        .iter()
        .map(|s| s.periods.iter().map(|p| (p.day, p.slot)).collect())
        .collect();

    progress.set_message("Building ILP model...");

    let mut vars = variables!();

    // x[s][k] = 1 if student s assigned to section k
    // Using BTreeMap for deterministic iteration order
    let mut x: BTreeMap<(usize, usize), _> = BTreeMap::new();

    // Only create variables for valid student-section combinations
    for (s, student) in students.iter().enumerate() {
        for (k, section) in sections.iter().enumerate() {
            // Check if student wants this course
            if !student.wants_course(&section.course_id) {
                continue;
            }

            // Check grade restrictions
            if let Some(course) = course_map.get(&section.course_id) {
                if !course.allows_grade(student.grade) {
                    continue;
                }
            }

            x.insert((s, k), vars.add(variable().binary()));
        }
    }

    progress.set_message("Building objective function...");
    progress.set_position(50);

    // Build objective: maximize weighted assignments
    let mut objective = Expression::default();

    for (s, student) in students.iter().enumerate() {
        for (k, section) in sections.iter().enumerate() {
            if let Some(&var) = x.get(&(s, k)) {
                let weight = if student.required_courses.contains(&section.course_id) {
                    1000.0 // Strong incentive for required courses
                } else if let Some(rank) = student.elective_rank(&section.course_id) {
                    (10 - rank.min(9)) as f64 // Elective preference (1-10)
                } else {
                    0.0
                };

                if weight > 0.0 {
                    objective += weight * var;
                }
            }
        }
    }

    let mut problem = vars.maximise(objective).using(good_lp::solvers::highs::highs);

    progress.set_message("Adding constraints...");
    progress.set_position(55);

    // Constraint 1: At most one section per course per student
    for (s, student) in students.iter().enumerate() {
        let all_courses: HashSet<&CourseId> = student.all_requested_courses().collect();

        for course_id in all_courses {
            if let Some(section_idxs) = section_indices.get(course_id) {
                let vars_for_course: Vec<_> = section_idxs
                    .iter()
                    .filter_map(|&k| x.get(&(s, k)).copied())
                    .collect();

                if vars_for_course.len() > 1 {
                    let sum: Expression = vars_for_course.into_iter().map(Expression::from).sum();
                    problem = problem.with(constraint!(sum <= 1));
                }
            }
        }
    }

    progress.set_position(60);

    // Constraint 2: Section capacity
    for (k, section) in sections.iter().enumerate() {
        let vars_for_section: Vec<_> = students
            .iter()
            .enumerate()
            .filter_map(|(s, _)| x.get(&(s, k)).copied())
            .collect();

        if !vars_for_section.is_empty() {
            let sum: Expression = vars_for_section.into_iter().map(Expression::from).sum();
            problem = problem.with(constraint!(sum <= section.capacity as f64));
        }
    }

    progress.set_position(65);

    // Constraint 3: No time conflicts per student
    for (s, _student) in students.iter().enumerate() {
        // Get all sections this student could be assigned to
        let student_sections: Vec<usize> = sections
            .iter()
            .enumerate()
            .filter(|(k, _)| x.contains_key(&(s, *k)))
            .map(|(k, _)| k)
            .collect();

        // Check each pair for conflicts
        for i in 0..student_sections.len() {
            for j in (i + 1)..student_sections.len() {
                let k1 = student_sections[i];
                let k2 = student_sections[j];

                // Check if these sections overlap in time
                let periods1 = &section_periods[k1];
                let periods2 = &section_periods[k2];

                let has_conflict = periods1.iter().any(|p| periods2.contains(p));

                if has_conflict {
                    if let (Some(&v1), Some(&v2)) = (x.get(&(s, k1)), x.get(&(s, k2))) {
                        // At most one of these can be selected
                        problem = problem.with(constraint!(v1 + v2 <= 1));
                    }
                }
            }
        }
    }

    progress.set_message("Solving ILP...");
    progress.set_position(70);

    // Solve
    let solution = problem.solve().map_err(|e| {
        crate::error::SchedulerError::SolverFailed(format!("{:?}", e))
    })?;

    progress.set_message("Extracting solution...");
    progress.set_position(85);

    // Extract assignments
    let mut unassigned = Vec::new();

    for (s, student) in students.iter().enumerate() {
        for (k, section) in sections.iter_mut().enumerate() {
            if let Some(&var) = x.get(&(s, k)) {
                if solution.value(var) > 0.5 {
                    section.enrolled_students.push(student.id.clone());
                }
            }
        }

        // Track unassigned required courses
        for course_id in &student.required_courses {
            let assigned = sections.iter().any(|sec| {
                &sec.course_id == course_id && sec.enrolled_students.contains(&student.id)
            });

            if !assigned {
                // Determine reason
                let reason = determine_unassigned_reason(
                    student,
                    course_id,
                    &sections,
                    &section_periods,
                    &course_map,
                );
                unassigned.push(UnassignedCourse {
                    student_id: student.id.clone(),
                    course_id: course_id.clone(),
                    reason,
                });
            }
        }
    }

    Ok((sections, unassigned))
}

fn determine_unassigned_reason(
    student: &Student,
    course_id: &CourseId,
    sections: &[Section],
    section_periods: &[HashSet<(u8, u8)>],
    course_map: &BTreeMap<&CourseId, &Course>,
) -> String {
    // Check grade restriction
    if let Some(course) = course_map.get(course_id) {
        if !course.allows_grade(student.grade) {
            return format!(
                "Grade {} not allowed (restricted to {:?})",
                student.grade,
                course.grade_restrictions
            );
        }
    }

    // Check if all sections are full
    let course_sections: Vec<(usize, &Section)> = sections
        .iter()
        .enumerate()
        .filter(|(_, s)| &s.course_id == course_id)
        .collect();

    if course_sections.is_empty() {
        return "No sections available".to_string();
    }

    let all_full = course_sections.iter().all(|(_, s)| s.is_full());
    if all_full {
        return "All sections at capacity".to_string();
    }

    // Check for time conflicts
    let student_periods: HashSet<(u8, u8)> = sections
        .iter()
        .enumerate()
        .filter(|(_, s)| s.enrolled_students.contains(&student.id))
        .flat_map(|(idx, _)| section_periods[idx].iter().copied())
        .collect();

    let has_available_slot = course_sections.iter().any(|(idx, sec)| {
        !sec.is_full() && section_periods[*idx].iter().all(|p| !student_periods.contains(p))
    });

    if !has_available_slot {
        return "Time conflict with other courses".to_string();
    }

    "Unknown reason".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Period, SectionId, StudentId, TeacherId};

    fn make_test_section(id: &str, course: &str, slot: u8, capacity: u32) -> Section {
        Section {
            id: SectionId(id.to_string()),
            course_id: CourseId(course.to_string()),
            teacher_id: Some(TeacherId("t1".to_string())),
            room_id: None,
            periods: (0..5).map(|d| Period::new(d, slot)).collect(),
            enrolled_students: vec![],
            capacity,
        }
    }

    #[test]
    fn test_assigns_required_courses() {
        let sections = vec![
            make_test_section("math-1", "math", 0, 30),
        ];

        let students = vec![Student {
            id: StudentId("s1".to_string()),
            name: "Student 1".to_string(),
            grade: 10,
            required_courses: vec![CourseId("math".to_string())],
            elective_preferences: vec![],
        }];

        let courses = vec![Course {
            id: CourseId("math".to_string()),
            name: "Math".to_string(),
            max_students: 30,
            periods_per_week: 5,
            grade_restrictions: None,
            required_features: vec![],
            sections: 1,
        }];

        let progress = ProgressBar::hidden();
        let (result, unassigned) =
            solve_student_assignment(sections, &students, &courses, &progress).unwrap();

        assert!(unassigned.is_empty());
        assert!(result[0].enrolled_students.contains(&StudentId("s1".to_string())));
    }

    #[test]
    fn test_respects_capacity() {
        let sections = vec![
            make_test_section("math-1", "math", 0, 1), // Only 1 seat
        ];

        let students = vec![
            Student {
                id: StudentId("s1".to_string()),
                name: "Student 1".to_string(),
                grade: 10,
                required_courses: vec![CourseId("math".to_string())],
                elective_preferences: vec![],
            },
            Student {
                id: StudentId("s2".to_string()),
                name: "Student 2".to_string(),
                grade: 10,
                required_courses: vec![CourseId("math".to_string())],
                elective_preferences: vec![],
            },
        ];

        let courses = vec![Course {
            id: CourseId("math".to_string()),
            name: "Math".to_string(),
            max_students: 1,
            periods_per_week: 5,
            grade_restrictions: None,
            required_features: vec![],
            sections: 1,
        }];

        let progress = ProgressBar::hidden();
        let (result, unassigned) =
            solve_student_assignment(sections, &students, &courses, &progress).unwrap();

        // Only 1 student should be enrolled
        assert_eq!(result[0].enrollment(), 1);
        // One student should be unassigned
        assert_eq!(unassigned.len(), 1);
    }

    #[test]
    fn test_prevents_time_conflicts() {
        let sections = vec![
            make_test_section("math-1", "math", 0, 30),
            make_test_section("eng-1", "eng", 0, 30), // Same time slot
        ];

        let students = vec![Student {
            id: StudentId("s1".to_string()),
            name: "Student 1".to_string(),
            grade: 10,
            required_courses: vec![
                CourseId("math".to_string()),
                CourseId("eng".to_string()),
            ],
            elective_preferences: vec![],
        }];

        let courses = vec![
            Course {
                id: CourseId("math".to_string()),
                name: "Math".to_string(),
                max_students: 30,
                periods_per_week: 5,
                grade_restrictions: None,
                required_features: vec![],
                sections: 1,
            },
            Course {
                id: CourseId("eng".to_string()),
                name: "English".to_string(),
                max_students: 30,
                periods_per_week: 5,
                grade_restrictions: None,
                required_features: vec![],
                sections: 1,
            },
        ];

        let progress = ProgressBar::hidden();
        let (result, unassigned) =
            solve_student_assignment(sections, &students, &courses, &progress).unwrap();

        // Student can only be in one class at slot 0
        let enrolled_count = result
            .iter()
            .filter(|s| s.enrolled_students.contains(&StudentId("s1".to_string())))
            .count();

        assert_eq!(enrolled_count, 1);
        assert_eq!(unassigned.len(), 1);
    }
}
