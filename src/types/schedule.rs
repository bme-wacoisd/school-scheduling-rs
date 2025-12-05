use serde::{Deserialize, Serialize};
use super::{CourseId, Section, SectionId, StudentId};
use std::collections::HashMap;

/// Represents an unassigned course for a student
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnassignedCourse {
    pub student_id: StudentId,
    pub course_id: CourseId,
    pub reason: String,
}

/// Metadata about the generated schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    pub generated_at: String,
    pub algorithm_version: String,
    pub score: f64,
    pub solve_time_ms: u64,
}

impl Default for ScheduleMetadata {
    fn default() -> Self {
        Self {
            generated_at: String::new(),
            algorithm_version: String::new(),
            score: 0.0,
            solve_time_ms: 0,
        }
    }
}

/// The complete generated schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub sections: Vec<Section>,
    pub unassigned: Vec<UnassignedCourse>,
    pub metadata: ScheduleMetadata,
}

impl Schedule {
    /// Create a new empty schedule
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            unassigned: Vec::new(),
            metadata: ScheduleMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                algorithm_version: env!("CARGO_PKG_VERSION").to_string(),
                score: 0.0,
                solve_time_ms: 0,
            },
        }
    }

    /// Get sections by course
    pub fn sections_for_course(&self, course_id: &CourseId) -> Vec<&Section> {
        self.sections
            .iter()
            .filter(|s| &s.course_id == course_id)
            .collect()
    }

    /// Get sections a student is enrolled in
    pub fn student_sections(&self, student_id: &StudentId) -> Vec<&Section> {
        self.sections
            .iter()
            .filter(|s| s.enrolled_students.contains(student_id))
            .collect()
    }

    /// Get a section by ID
    pub fn get_section(&self, section_id: &SectionId) -> Option<&Section> {
        self.sections.iter().find(|s| &s.id == section_id)
    }

    /// Get a mutable section by ID
    pub fn get_section_mut(&mut self, section_id: &SectionId) -> Option<&mut Section> {
        self.sections.iter_mut().find(|s| &s.id == section_id)
    }

    /// Build a map of section ID to section index for fast lookups
    pub fn section_index_map(&self) -> HashMap<&SectionId, usize> {
        self.sections
            .iter()
            .enumerate()
            .map(|(i, s)| (&s.id, i))
            .collect()
    }

    /// Total number of student-section assignments
    pub fn total_assignments(&self) -> usize {
        self.sections.iter().map(|s| s.enrollment()).sum()
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}
