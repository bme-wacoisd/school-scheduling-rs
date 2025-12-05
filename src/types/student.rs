use serde::{Deserialize, Serialize};
use super::{CourseId, StudentId};

/// Represents a student with their course requirements and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: StudentId,
    pub name: String,
    pub grade: u8,
    pub required_courses: Vec<CourseId>,
    /// Elective preferences in priority order (first = highest priority)
    pub elective_preferences: Vec<CourseId>,
}

impl Student {
    /// Get all courses this student wants (required + electives)
    pub fn all_requested_courses(&self) -> impl Iterator<Item = &CourseId> {
        self.required_courses.iter().chain(self.elective_preferences.iter())
    }

    /// Check if this student wants a particular course
    pub fn wants_course(&self, course_id: &CourseId) -> bool {
        self.required_courses.contains(course_id) || self.elective_preferences.contains(course_id)
    }

    /// Get the preference rank for an elective (0 = highest priority)
    pub fn elective_rank(&self, course_id: &CourseId) -> Option<usize> {
        self.elective_preferences.iter().position(|c| c == course_id)
    }
}
