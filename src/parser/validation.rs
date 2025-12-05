use crate::error::Result;
use crate::types::{Course, CourseId, Room, ScheduleInput, Student, Teacher, TeacherId};
use std::collections::{HashMap, HashSet};

/// Validation result with collected errors
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

/// Validate all input data
pub fn validate_input(input: &ScheduleInput) -> Result<ValidationResult> {
    let mut result = ValidationResult::default();

    // Build lookup maps
    let course_ids: HashSet<&CourseId> = input.courses.iter().map(|c| &c.id).collect();
    let _teacher_ids: HashSet<&TeacherId> = input.teachers.iter().map(|t| &t.id).collect();

    // Check for duplicate IDs
    check_duplicate_ids(&input.students, &mut result);
    check_duplicate_teacher_ids(&input.teachers, &mut result);
    check_duplicate_course_ids(&input.courses, &mut result);
    check_duplicate_room_ids(&input.rooms, &mut result);

    // Validate student course references
    for student in &input.students {
        for course_id in student.all_requested_courses() {
            if !course_ids.contains(course_id) {
                result.add_error(format!(
                    "Student '{}' references unknown course '{}'",
                    student.id, course_id
                ));
            }
        }
    }

    // Validate teacher subject references
    for teacher in &input.teachers {
        for course_id in &teacher.subjects {
            if !course_ids.contains(course_id) {
                result.add_warning(format!(
                    "Teacher '{}' lists unknown course '{}' in subjects",
                    teacher.id, course_id
                ));
            }
        }
    }

    // Check each course has at least one qualified teacher
    let teachers_by_course = build_teachers_by_course(&input.teachers);
    for course in &input.courses {
        if !teachers_by_course.contains_key(&course.id) {
            result.add_error(format!(
                "Course '{}' has no qualified teachers",
                course.id
            ));
        }
    }

    // Check grade restrictions make sense
    for course in &input.courses {
        if let Some(grades) = &course.grade_restrictions {
            for grade in grades {
                if *grade < 9 || *grade > 12 {
                    result.add_warning(format!(
                        "Course '{}' has unusual grade restriction: {}",
                        course.id, grade
                    ));
                }
            }
        }
    }

    // Check room capacity vs course max_students
    let max_room_capacity = input.rooms.iter().map(|r| r.capacity).max().unwrap_or(0);
    for course in &input.courses {
        if course.max_students > max_room_capacity {
            result.add_warning(format!(
                "Course '{}' max_students ({}) exceeds largest room capacity ({})",
                course.id, course.max_students, max_room_capacity
            ));
        }
    }

    if !result.is_valid() {
        return Err(anyhow::anyhow!(
            "Validation failed with {} errors:\n{}",
            result.errors.len(),
            result.errors.join("\n")
        ));
    }

    Ok(result)
}

fn check_duplicate_ids(students: &[Student], result: &mut ValidationResult) {
    let mut seen = HashSet::new();
    for student in students {
        if !seen.insert(&student.id) {
            result.add_error(format!("Duplicate student ID: '{}'", student.id));
        }
    }
}

fn check_duplicate_teacher_ids(teachers: &[Teacher], result: &mut ValidationResult) {
    let mut seen = HashSet::new();
    for teacher in teachers {
        if !seen.insert(&teacher.id) {
            result.add_error(format!("Duplicate teacher ID: '{}'", teacher.id));
        }
    }
}

fn check_duplicate_course_ids(courses: &[Course], result: &mut ValidationResult) {
    let mut seen = HashSet::new();
    for course in courses {
        if !seen.insert(&course.id) {
            result.add_error(format!("Duplicate course ID: '{}'", course.id));
        }
    }
}

fn check_duplicate_room_ids(rooms: &[Room], result: &mut ValidationResult) {
    let mut seen = HashSet::new();
    for room in rooms {
        if !seen.insert(&room.id) {
            result.add_error(format!("Duplicate room ID: '{}'", room.id));
        }
    }
}

/// Build a map from course ID to list of qualified teachers
pub fn build_teachers_by_course(teachers: &[Teacher]) -> HashMap<&CourseId, Vec<&Teacher>> {
    let mut map: HashMap<&CourseId, Vec<&Teacher>> = HashMap::new();
    for teacher in teachers {
        for course_id in &teacher.subjects {
            map.entry(course_id).or_default().push(teacher);
        }
    }
    map
}
