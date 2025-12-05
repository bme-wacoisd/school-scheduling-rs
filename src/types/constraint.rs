use serde::{Deserialize, Serialize};

/// Classification of constraint strictness
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstraintType {
    Hard,
    Soft { weight: f64 },
}

/// All supported constraints
#[derive(Debug, Clone)]
pub enum Constraint {
    // Hard constraints (must be satisfied)
    NoTeacherConflict,
    NoStudentConflict,
    NoRoomConflict,
    RoomCapacity,
    TeacherQualified,
    TeacherAvailability,
    RoomFeatures,
    GradeRestriction,
    TeacherMaxSections,

    // Soft constraints (weighted in objective)
    BalancedSections { weight: f64 },
    StudentElectivePreference { weight: f64 },
    MinimizeGaps { weight: f64 },
    TeacherPreferences { weight: f64 },
    LunchAvailability { weight: f64, periods: Vec<u8> },
}

impl Constraint {
    /// Get the constraint type (hard or soft with weight)
    pub fn constraint_type(&self) -> ConstraintType {
        match self {
            Constraint::NoTeacherConflict
            | Constraint::NoStudentConflict
            | Constraint::NoRoomConflict
            | Constraint::RoomCapacity
            | Constraint::TeacherQualified
            | Constraint::TeacherAvailability
            | Constraint::RoomFeatures
            | Constraint::GradeRestriction
            | Constraint::TeacherMaxSections => ConstraintType::Hard,

            Constraint::BalancedSections { weight }
            | Constraint::StudentElectivePreference { weight }
            | Constraint::MinimizeGaps { weight }
            | Constraint::TeacherPreferences { weight }
            | Constraint::LunchAvailability { weight, .. } => {
                ConstraintType::Soft { weight: *weight }
            }
        }
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Constraint::NoTeacherConflict => "No Teacher Conflict",
            Constraint::NoStudentConflict => "No Student Conflict",
            Constraint::NoRoomConflict => "No Room Conflict",
            Constraint::RoomCapacity => "Room Capacity",
            Constraint::TeacherQualified => "Teacher Qualified",
            Constraint::TeacherAvailability => "Teacher Availability",
            Constraint::RoomFeatures => "Room Features",
            Constraint::GradeRestriction => "Grade Restriction",
            Constraint::TeacherMaxSections => "Teacher Max Sections",
            Constraint::BalancedSections { .. } => "Balanced Sections",
            Constraint::StudentElectivePreference { .. } => "Student Elective Preference",
            Constraint::MinimizeGaps { .. } => "Minimize Gaps",
            Constraint::TeacherPreferences { .. } => "Teacher Preferences",
            Constraint::LunchAvailability { .. } => "Lunch Availability",
        }
    }
}

/// Configuration for the schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    #[serde(default = "default_periods_per_day")]
    pub periods_per_day: u8,
    #[serde(default = "default_days_per_week")]
    pub days_per_week: u8,
    #[serde(default)]
    pub lunch_periods: Vec<u8>,
}

fn default_periods_per_day() -> u8 {
    8
}

fn default_days_per_week() -> u8 {
    5
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            periods_per_day: 8,
            days_per_week: 5,
            lunch_periods: vec![3, 4], // Periods 4 and 5 (0-indexed)
        }
    }
}

/// All input data bundled together
#[derive(Debug)]
pub struct ScheduleInput {
    pub students: Vec<super::Student>,
    pub teachers: Vec<super::Teacher>,
    pub courses: Vec<super::Course>,
    pub rooms: Vec<super::Room>,
    pub constraints: Vec<Constraint>,
    pub config: ScheduleConfig,
}
