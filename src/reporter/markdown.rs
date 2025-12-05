use crate::types::{CourseId, Schedule, ScheduleInput};
use crate::validator::ValidationReport;
use std::collections::HashMap;

/// Generate a markdown report of the schedule
pub fn generate_markdown_report(
    schedule: &Schedule,
    input: &ScheduleInput,
    validation: &ValidationReport,
) -> String {
    let mut lines = vec![
        "# Schedule Report".to_string(),
        String::new(),
        format!("Generated: {}", schedule.metadata.generated_at),
        format!("Algorithm: v{}", schedule.metadata.algorithm_version),
        format!("Solve time: {}ms", schedule.metadata.solve_time_ms),
        String::new(),
    ];

    // Summary
    lines.push("## Summary\n".to_string());
    lines.push(format!(
        "| Metric | Value |",
    ));
    lines.push("|--------|-------|".to_string());
    lines.push(format!(
        "| Total Sections | {} |",
        validation.statistics.total_sections
    ));
    lines.push(format!(
        "| Total Students | {} |",
        validation.statistics.total_students
    ));
    lines.push(format!(
        "| Total Assignments | {} |",
        validation.statistics.total_assignments
    ));
    lines.push(format!(
        "| Unassigned Required | {} |",
        validation.statistics.unassigned_required
    ));
    lines.push(format!(
        "| Unassigned Electives | {} |",
        validation.statistics.unassigned_electives
    ));
    lines.push(format!(
        "| Avg Fill Rate | {:.1}% |",
        validation.statistics.avg_section_fill_rate
    ));
    lines.push(format!(
        "| Overall Score | {:.1}/100 |",
        validation.total_score
    ));
    lines.push(String::new());

    // Validation status
    if validation.is_valid {
        lines.push("## Validation: ✅ PASSED\n".to_string());
    } else {
        lines.push("## Validation: ❌ FAILED\n".to_string());
        for violation in &validation.hard_violations {
            lines.push(format!("- **{}**: {}", violation.constraint, violation.message));
        }
        lines.push(String::new());
    }

    // Soft scores
    lines.push("## Soft Constraint Scores\n".to_string());
    for score in &validation.soft_scores {
        let pct = if score.max_score > 0.0 {
            (score.score / score.max_score) * 100.0
        } else {
            100.0
        };
        lines.push(format!(
            "- **{}**: {:.1}% ({})",
            score.constraint, pct, score.details
        ));
    }
    lines.push(String::new());

    // Course breakdown
    lines.push("## Course Sections\n".to_string());

    let course_map: HashMap<&CourseId, &str> = input
        .courses
        .iter()
        .map(|c| (&c.id, c.name.as_str()))
        .collect();

    let mut by_course: HashMap<&CourseId, Vec<&crate::types::Section>> = HashMap::new();
    for section in &schedule.sections {
        by_course.entry(&section.course_id).or_default().push(section);
    }

    let mut course_ids: Vec<_> = by_course.keys().collect();
    course_ids.sort_by_key(|c| course_map.get(*c).unwrap_or(&""));

    for course_id in course_ids {
        let course_name = course_map.get(course_id).unwrap_or(&"Unknown");
        let sections = &by_course[course_id];

        lines.push(format!("### {}\n", course_name));
        lines.push("| Section | Period | Room | Teacher | Enrolled |".to_string());
        lines.push("|---------|--------|------|---------|----------|".to_string());

        for section in sections {
            let period = section
                .periods
                .first()
                .map(|p| format!("P{}", p.slot + 1))
                .unwrap_or_else(|| "TBD".to_string());

            let room = section
                .room_id
                .as_ref()
                .map(|r| r.0.clone())
                .unwrap_or_else(|| "TBD".to_string());

            let teacher = section
                .teacher_id
                .as_ref()
                .and_then(|tid| input.teachers.iter().find(|t| &t.id == tid))
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "TBD".to_string());

            lines.push(format!(
                "| {} | {} | {} | {} | {}/{} |",
                section.id,
                period,
                room,
                teacher,
                section.enrollment(),
                section.capacity
            ));
        }
        lines.push(String::new());
    }

    // Unassigned
    if !schedule.unassigned.is_empty() {
        lines.push("## Unassigned Students\n".to_string());
        lines.push("| Student | Course | Reason |".to_string());
        lines.push("|---------|--------|--------|".to_string());

        for u in &schedule.unassigned {
            let course_name = course_map.get(&u.course_id).unwrap_or(&"Unknown");
            lines.push(format!("| {} | {} | {} |", u.student_id, course_name, u.reason));
        }
    }

    lines.join("\n")
}
