use crate::types::{Course, CourseId, Section, SectionId, Teacher, TeacherId};
use std::collections::HashMap;

/// Phase 1: Create sections for each course and assign teachers
pub fn create_sections(courses: &[Course], teachers: &[Teacher]) -> Vec<Section> {
    let teachers_by_course = build_teachers_by_course(teachers);
    let mut teacher_section_counts: HashMap<&TeacherId, u8> = HashMap::new();
    let mut sections = Vec::new();

    for course in courses {
        let qualified_teachers = teachers_by_course
            .get(&course.id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        for section_num in 0..course.sections {
            let section_id = SectionId(format!("{}-{}", course.id.0, section_num + 1));

            let mut section = Section::new(section_id, course.id.clone(), course.max_students);

            // Assign teacher using round-robin among qualified teachers
            if !qualified_teachers.is_empty() {
                // Find teacher with fewest sections who can still take more
                let teacher = qualified_teachers
                    .iter()
                    .filter(|t| {
                        let count = teacher_section_counts.get(&t.id).copied().unwrap_or(0);
                        count < t.max_sections
                    })
                    .min_by_key(|t| teacher_section_counts.get(&t.id).copied().unwrap_or(0));

                if let Some(teacher) = teacher {
                    section.teacher_id = Some(teacher.id.clone());
                    *teacher_section_counts.entry(&teacher.id).or_insert(0) += 1;
                }
            }

            sections.push(section);
        }
    }

    sections
}

fn build_teachers_by_course(teachers: &[Teacher]) -> HashMap<&CourseId, Vec<&Teacher>> {
    let mut map: HashMap<&CourseId, Vec<&Teacher>> = HashMap::new();
    for teacher in teachers {
        for course_id in &teacher.subjects {
            map.entry(course_id).or_default().push(teacher);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CourseId;

    #[test]
    fn test_creates_correct_number_of_sections() {
        let courses = vec![
            Course {
                id: CourseId("math".to_string()),
                name: "Math".to_string(),
                max_students: 30,
                periods_per_week: 5,
                grade_restrictions: None,
                required_features: vec![],
                sections: 3,
            },
        ];

        let teachers = vec![
            Teacher {
                id: TeacherId("t1".to_string()),
                name: "Teacher 1".to_string(),
                subjects: vec![CourseId("math".to_string())],
                max_sections: 5,
                unavailable: vec![],
            },
        ];

        let sections = create_sections(&courses, &teachers);
        assert_eq!(sections.len(), 3);
    }

    #[test]
    fn test_assigns_teachers_round_robin() {
        let courses = vec![
            Course {
                id: CourseId("math".to_string()),
                name: "Math".to_string(),
                max_students: 30,
                periods_per_week: 5,
                grade_restrictions: None,
                required_features: vec![],
                sections: 4,
            },
        ];

        let teachers = vec![
            Teacher {
                id: TeacherId("t1".to_string()),
                name: "Teacher 1".to_string(),
                subjects: vec![CourseId("math".to_string())],
                max_sections: 2,
                unavailable: vec![],
            },
            Teacher {
                id: TeacherId("t2".to_string()),
                name: "Teacher 2".to_string(),
                subjects: vec![CourseId("math".to_string())],
                max_sections: 2,
                unavailable: vec![],
            },
        ];

        let sections = create_sections(&courses, &teachers);

        // Both teachers should have 2 sections each
        let t1_count = sections
            .iter()
            .filter(|s| s.teacher_id.as_ref().map(|t| &t.0) == Some(&"t1".to_string()))
            .count();
        let t2_count = sections
            .iter()
            .filter(|s| s.teacher_id.as_ref().map(|t| &t.0) == Some(&"t2".to_string()))
            .count();

        assert_eq!(t1_count, 2);
        assert_eq!(t2_count, 2);
    }
}
