mod hard_constraints;
mod soft_constraints;

pub use hard_constraints::*;
pub use soft_constraints::*;

use crate::types::{Schedule, ScheduleInput};

/// Result of schedule validation
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub hard_violations: Vec<Violation>,
    pub soft_scores: Vec<SoftScore>,
    pub total_score: f64,
    pub statistics: ScheduleStatistics,
}

/// A constraint violation
#[derive(Debug, Clone)]
pub struct Violation {
    pub constraint: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

/// Score for a soft constraint
#[derive(Debug, Clone)]
pub struct SoftScore {
    pub constraint: String,
    pub score: f64,
    pub max_score: f64,
    pub details: String,
}

/// Statistics about the schedule
#[derive(Debug, Clone)]
pub struct ScheduleStatistics {
    pub total_sections: usize,
    pub total_students: usize,
    pub total_assignments: usize,
    pub unassigned_required: usize,
    pub unassigned_electives: usize,
    pub avg_section_fill_rate: f64,
    pub section_balance_score: f64,
}

/// Validate a complete schedule
pub fn validate_schedule(schedule: &Schedule, input: &ScheduleInput) -> ValidationReport {
    let mut hard_violations = Vec::new();

    // Check hard constraints
    hard_violations.extend(check_teacher_conflicts(schedule));
    hard_violations.extend(check_student_conflicts(schedule));
    hard_violations.extend(check_room_conflicts(schedule));
    hard_violations.extend(check_capacity_violations(schedule));

    // Calculate soft constraint scores
    let soft_scores = calculate_soft_scores(schedule, input);

    // Calculate statistics
    let statistics = calculate_statistics(schedule, input);

    // Calculate total score
    let total_score = if hard_violations.iter().any(|v| v.severity == Severity::Error) {
        0.0
    } else {
        let soft_total: f64 = soft_scores.iter().map(|s| s.score).sum();
        let soft_max: f64 = soft_scores.iter().map(|s| s.max_score).sum();
        if soft_max > 0.0 {
            (soft_total / soft_max) * 100.0
        } else {
            100.0
        }
    };

    ValidationReport {
        is_valid: hard_violations.iter().all(|v| v.severity != Severity::Error),
        hard_violations,
        soft_scores,
        total_score,
        statistics,
    }
}

fn calculate_statistics(schedule: &Schedule, input: &ScheduleInput) -> ScheduleStatistics {
    let total_sections = schedule.sections.len();
    let total_students = input.students.len();
    let total_assignments = schedule.total_assignments();

    // Count unassigned
    let unassigned_required = schedule
        .unassigned
        .iter()
        .filter(|u| {
            input
                .students
                .iter()
                .find(|s| s.id == u.student_id)
                .map(|s| s.required_courses.contains(&u.course_id))
                .unwrap_or(false)
        })
        .count();

    let unassigned_electives = schedule.unassigned.len() - unassigned_required;

    // Average fill rate
    let avg_section_fill_rate = if total_sections > 0 {
        schedule
            .sections
            .iter()
            .map(|s| s.enrollment() as f64 / s.capacity as f64)
            .sum::<f64>()
            / total_sections as f64
            * 100.0
    } else {
        0.0
    };

    // Section balance
    let section_balance_score = crate::scheduler::calculate_balance_score(&schedule.sections);

    ScheduleStatistics {
        total_sections,
        total_students,
        total_assignments,
        unassigned_required,
        unassigned_electives,
        avg_section_fill_rate,
        section_balance_score,
    }
}
