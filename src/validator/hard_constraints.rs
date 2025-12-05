use crate::types::Schedule;
use crate::validator::{Severity, Violation};
use std::collections::{HashMap, HashSet};

/// Check for teacher double-booking
pub fn check_teacher_conflicts(schedule: &Schedule) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut teacher_periods: HashMap<&str, HashSet<(u8, u8)>> = HashMap::new();

    for section in &schedule.sections {
        if let Some(ref teacher_id) = section.teacher_id {
            let periods = teacher_periods.entry(&teacher_id.0).or_default();

            for period in &section.periods {
                let key = (period.day, period.slot);
                if !periods.insert(key) {
                    violations.push(Violation {
                        constraint: "NoTeacherConflict".to_string(),
                        message: format!(
                            "Teacher '{}' double-booked at {}",
                            teacher_id, period
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    violations
}

/// Check for student double-booking
pub fn check_student_conflicts(schedule: &Schedule) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut student_periods: HashMap<&str, HashSet<(u8, u8)>> = HashMap::new();

    for section in &schedule.sections {
        for student_id in &section.enrolled_students {
            let periods = student_periods.entry(&student_id.0).or_default();

            for period in &section.periods {
                let key = (period.day, period.slot);
                if !periods.insert(key) {
                    violations.push(Violation {
                        constraint: "NoStudentConflict".to_string(),
                        message: format!(
                            "Student '{}' double-booked at {}",
                            student_id, period
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    violations
}

/// Check for room double-booking
pub fn check_room_conflicts(schedule: &Schedule) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut room_periods: HashMap<&str, HashSet<(u8, u8)>> = HashMap::new();

    for section in &schedule.sections {
        if let Some(ref room_id) = section.room_id {
            let periods = room_periods.entry(&room_id.0).or_default();

            for period in &section.periods {
                let key = (period.day, period.slot);
                if !periods.insert(key) {
                    violations.push(Violation {
                        constraint: "NoRoomConflict".to_string(),
                        message: format!(
                            "Room '{}' double-booked at {}",
                            room_id, period
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    violations
}

/// Check for capacity violations
pub fn check_capacity_violations(schedule: &Schedule) -> Vec<Violation> {
    let mut violations = Vec::new();

    for section in &schedule.sections {
        if section.enrollment() > section.capacity as usize {
            violations.push(Violation {
                constraint: "RoomCapacity".to_string(),
                message: format!(
                    "Section '{}' over capacity: {} enrolled, {} capacity",
                    section.id,
                    section.enrollment(),
                    section.capacity
                ),
                severity: Severity::Error,
            });
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Period, Section, SectionId, CourseId, TeacherId, StudentId};

    #[test]
    fn test_detects_teacher_conflict() {
        let schedule = Schedule {
            sections: vec![
                Section {
                    id: SectionId("s1".to_string()),
                    course_id: CourseId("math".to_string()),
                    teacher_id: Some(TeacherId("t1".to_string())),
                    room_id: None,
                    periods: vec![Period::new(0, 0)],
                    enrolled_students: vec![],
                    capacity: 30,
                },
                Section {
                    id: SectionId("s2".to_string()),
                    course_id: CourseId("eng".to_string()),
                    teacher_id: Some(TeacherId("t1".to_string())),
                    room_id: None,
                    periods: vec![Period::new(0, 0)], // Same time
                    enrolled_students: vec![],
                    capacity: 30,
                },
            ],
            unassigned: vec![],
            metadata: Default::default(),
        };

        let violations = check_teacher_conflicts(&schedule);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_detects_student_conflict() {
        let schedule = Schedule {
            sections: vec![
                Section {
                    id: SectionId("s1".to_string()),
                    course_id: CourseId("math".to_string()),
                    teacher_id: None,
                    room_id: None,
                    periods: vec![Period::new(0, 0)],
                    enrolled_students: vec![StudentId("stu1".to_string())],
                    capacity: 30,
                },
                Section {
                    id: SectionId("s2".to_string()),
                    course_id: CourseId("eng".to_string()),
                    teacher_id: None,
                    room_id: None,
                    periods: vec![Period::new(0, 0)], // Same time
                    enrolled_students: vec![StudentId("stu1".to_string())], // Same student
                    capacity: 30,
                },
            ],
            unassigned: vec![],
            metadata: Default::default(),
        };

        let violations = check_student_conflicts(&schedule);
        assert!(!violations.is_empty());
    }
}
