use serde::{Deserialize, Serialize};
use super::CourseId;

/// Represents a course offering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: CourseId,
    pub name: String,
    /// Maximum students per section
    pub max_students: u32,
    /// Number of periods this course meets per week
    #[serde(default = "default_periods_per_week")]
    pub periods_per_week: u8,
    /// Grade restrictions (None = open to all grades)
    #[serde(default)]
    pub grade_restrictions: Option<Vec<u8>>,
    /// Room features required (e.g., "lab", "computers")
    #[serde(default)]
    pub required_features: Vec<String>,
    /// Number of sections to create
    pub sections: u8,
}

fn default_periods_per_week() -> u8 {
    5 // Default to 5 periods per week (daily class)
}

impl Course {
    /// Check if a student of the given grade can take this course
    pub fn allows_grade(&self, grade: u8) -> bool {
        match &self.grade_restrictions {
            Some(grades) => grades.contains(&grade),
            None => true,
        }
    }
}
