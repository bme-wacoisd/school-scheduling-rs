use serde::{Deserialize, Serialize};
use super::{CourseId, Period, RoomId, SectionId, StudentId, TeacherId};

/// Represents a section of a course (a specific class instance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: SectionId,
    pub course_id: CourseId,
    pub teacher_id: Option<TeacherId>,
    pub room_id: Option<RoomId>,
    pub periods: Vec<Period>,
    pub enrolled_students: Vec<StudentId>,
    pub capacity: u32,
}

impl Section {
    /// Create a new empty section
    pub fn new(id: SectionId, course_id: CourseId, capacity: u32) -> Self {
        Self {
            id,
            course_id,
            teacher_id: None,
            room_id: None,
            periods: Vec::new(),
            enrolled_students: Vec::new(),
            capacity,
        }
    }

    /// Current enrollment count
    pub fn enrollment(&self) -> usize {
        self.enrolled_students.len()
    }

    /// Available seats
    pub fn available_seats(&self) -> u32 {
        self.capacity.saturating_sub(self.enrolled_students.len() as u32)
    }

    /// Check if section is at capacity
    pub fn is_full(&self) -> bool {
        self.enrolled_students.len() >= self.capacity as usize
    }

    /// Check if a student is enrolled
    pub fn has_student(&self, student_id: &StudentId) -> bool {
        self.enrolled_students.contains(student_id)
    }

    /// Enroll a student (does not check capacity)
    pub fn enroll(&mut self, student_id: StudentId) {
        if !self.has_student(&student_id) {
            self.enrolled_students.push(student_id);
        }
    }

    /// Remove a student
    pub fn unenroll(&mut self, student_id: &StudentId) {
        self.enrolled_students.retain(|s| s != student_id);
    }
}
