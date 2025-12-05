use crate::types::{CourseId, Schedule, ScheduleInput};
use crate::validator::SoftScore;
use std::collections::HashMap;

/// Calculate all soft constraint scores
pub fn calculate_soft_scores(schedule: &Schedule, input: &ScheduleInput) -> Vec<SoftScore> {
    vec![
        score_required_courses(schedule, input),
        score_elective_preferences(schedule, input),
        score_section_balance(schedule),
    ]
}

/// Score for required course fulfillment
fn score_required_courses(schedule: &Schedule, input: &ScheduleInput) -> SoftScore {
    let total_required: usize = input
        .students
        .iter()
        .map(|s| s.required_courses.len())
        .sum();

    let fulfilled: usize = input
        .students
        .iter()
        .map(|student| {
            student
                .required_courses
                .iter()
                .filter(|course_id| {
                    schedule
                        .sections
                        .iter()
                        .any(|sec| &sec.course_id == *course_id && sec.has_student(&student.id))
                })
                .count()
        })
        .sum();

    let score = fulfilled as f64;
    let max_score = total_required as f64;

    SoftScore {
        constraint: "RequiredCourses".to_string(),
        score,
        max_score,
        details: format!("{}/{} required courses fulfilled", fulfilled, total_required),
    }
}

/// Score for elective preference satisfaction
fn score_elective_preferences(schedule: &Schedule, input: &ScheduleInput) -> SoftScore {
    let mut total_points = 0.0;
    let mut max_points = 0.0;

    for student in &input.students {
        for (rank, course_id) in student.elective_preferences.iter().enumerate() {
            let weight = (10 - rank.min(9)) as f64;
            max_points += weight;

            let assigned = schedule
                .sections
                .iter()
                .any(|sec| &sec.course_id == course_id && sec.has_student(&student.id));

            if assigned {
                total_points += weight;
            }
        }
    }

    SoftScore {
        constraint: "ElectivePreferences".to_string(),
        score: total_points,
        max_score: max_points,
        details: format!(
            "{:.1}/{:.1} elective preference points",
            total_points, max_points
        ),
    }
}

/// Score for section balance
fn score_section_balance(schedule: &Schedule) -> SoftScore {
    // Group sections by course
    let mut by_course: HashMap<&CourseId, Vec<usize>> = HashMap::new();
    for section in &schedule.sections {
        by_course
            .entry(&section.course_id)
            .or_default()
            .push(section.enrollment());
    }

    let mut total_imbalance = 0.0;
    let mut course_count = 0;

    for enrollments in by_course.values() {
        if enrollments.len() < 2 {
            continue;
        }

        let max = *enrollments.iter().max().unwrap_or(&0) as f64;
        let min = *enrollments.iter().min().unwrap_or(&0) as f64;

        if max > 0.0 {
            // Imbalance as percentage
            total_imbalance += (max - min) / max;
        }
        course_count += 1;
    }

    let avg_imbalance = if course_count > 0 {
        total_imbalance / course_count as f64
    } else {
        0.0
    };

    // Convert to score (0 = bad, 100 = perfect)
    let score = (1.0 - avg_imbalance) * 100.0;

    SoftScore {
        constraint: "SectionBalance".to_string(),
        score,
        max_score: 100.0,
        details: format!(
            "{:.1}% average imbalance across {} multi-section courses",
            avg_imbalance * 100.0,
            course_count
        ),
    }
}
