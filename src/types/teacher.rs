use serde::{Deserialize, Serialize};
use super::{CourseId, Period, TeacherId};

/// Represents a teacher with their qualifications and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
    pub id: TeacherId,
    pub name: String,
    /// Courses this teacher is qualified to teach
    pub subjects: Vec<CourseId>,
    /// Maximum number of sections this teacher can teach
    pub max_sections: u8,
    /// Periods when the teacher is unavailable
    #[serde(default)]
    pub unavailable: Vec<Period>,
}

impl Teacher {
    /// Check if teacher can teach a given course
    pub fn can_teach(&self, course_id: &CourseId) -> bool {
        self.subjects.contains(course_id)
    }

    /// Check if teacher is available during a period
    pub fn is_available(&self, period: &Period) -> bool {
        !self.unavailable.contains(period)
    }
}
