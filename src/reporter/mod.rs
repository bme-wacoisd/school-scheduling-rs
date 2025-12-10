mod json;
mod markdown;
mod text;

pub use json::*;
pub use markdown::*;
pub use text::*;

use crate::error::Result;
use crate::types::{Schedule, ScheduleInput, StudentId, TeacherId};
use crate::validator::ValidationReport;
use std::fs;
use std::path::Path;

/// Output format for reports
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Json,
    Markdown,
    Text,
}

/// Generate all reports and write to output directory
pub fn generate_reports(
    schedule: &Schedule,
    input: &ScheduleInput,
    validation: &ValidationReport,
    output_dir: &Path,
    formats: &[OutputFormat],
) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    // Clone schedule and update score from validation
    let mut schedule_with_score = schedule.clone();
    schedule_with_score.metadata.score = validation.total_score;

    for format in formats {
        match format {
            OutputFormat::Json => {
                let json = generate_json_report(&schedule_with_score)?;
                fs::write(output_dir.join("schedule.json"), json)?;
            }
            OutputFormat::Markdown => {
                let md = generate_markdown_report(&schedule_with_score, input, validation);
                fs::write(output_dir.join("schedule.md"), md)?;
            }
            OutputFormat::Text => {
                let txt = generate_text_report(&schedule_with_score, input, validation);
                fs::write(output_dir.join("schedule.txt"), txt)?;
            }
        }
    }

    Ok(())
}

/// Generate a student's individual schedule
pub fn generate_student_schedule(
    schedule: &Schedule,
    input: &ScheduleInput,
    student_id: &StudentId,
) -> Option<String> {
    let student = input.students.iter().find(|s| &s.id == student_id)?;

    let mut lines = vec![
        format!("# Schedule for {} ({})", student.name, student.id),
        format!("Grade: {}\n", student.grade),
    ];

    // Get enrolled sections
    let enrolled: Vec<_> = schedule
        .sections
        .iter()
        .filter(|s| s.has_student(student_id))
        .collect();

    if enrolled.is_empty() {
        lines.push("No courses enrolled.".to_string());
    } else {
        // Group by period
        let mut by_slot: Vec<(&str, String)> = Vec::new();

        for section in &enrolled {
            let course = input
                .courses
                .iter()
                .find(|c| c.id == section.course_id)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");

            let teacher = section
                .teacher_id
                .as_ref()
                .and_then(|tid| input.teachers.iter().find(|t| &t.id == tid))
                .map(|t| t.name.as_str())
                .unwrap_or("TBD");

            let room = section
                .room_id
                .as_ref()
                .map(|r| r.0.as_str())
                .unwrap_or("TBD");

            if let Some(period) = section.periods.first() {
                by_slot.push((
                    period.day_name(),
                    format!(
                        "Period {}: {} ({}) - Room {}",
                        period.slot + 1,
                        course,
                        teacher,
                        room
                    ),
                ));
            }
        }

        lines.push("## Daily Schedule\n".to_string());
        for (day, info) in by_slot {
            lines.push(format!("**{}**: {}", day, info));
        }
    }

    // Show unassigned courses
    let unassigned: Vec<_> = schedule
        .unassigned
        .iter()
        .filter(|u| &u.student_id == student_id)
        .collect();

    if !unassigned.is_empty() {
        lines.push("\n## Unassigned Courses\n".to_string());
        for u in unassigned {
            let course_name = input
                .courses
                .iter()
                .find(|c| c.id == u.course_id)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");
            lines.push(format!("- {} ({}): {}", course_name, u.course_id, u.reason));
        }
    }

    Some(lines.join("\n"))
}

/// Generate a teacher's schedule
pub fn generate_teacher_schedule(
    schedule: &Schedule,
    input: &ScheduleInput,
    teacher_id: &TeacherId,
) -> Option<String> {
    let teacher = input.teachers.iter().find(|t| &t.id == teacher_id)?;

    let mut lines = vec![
        format!("# Schedule for {} ({})", teacher.name, teacher.id),
        String::new(),
    ];

    // Get assigned sections
    let sections: Vec<_> = schedule
        .sections
        .iter()
        .filter(|s| s.teacher_id.as_ref() == Some(teacher_id))
        .collect();

    if sections.is_empty() {
        lines.push("No sections assigned.".to_string());
    } else {
        lines.push(format!("## Teaching {} sections\n", sections.len()));

        for section in sections {
            let course = input
                .courses
                .iter()
                .find(|c| c.id == section.course_id)
                .map(|c| c.name.as_str())
                .unwrap_or("Unknown");

            let room = section
                .room_id
                .as_ref()
                .map(|r| r.0.as_str())
                .unwrap_or("TBD");

            let period_str = section
                .periods
                .first()
                .map(|p| format!("Period {}", p.slot + 1))
                .unwrap_or_else(|| "TBD".to_string());

            lines.push(format!(
                "- **{}** ({}): {} - Room {} ({} students)",
                course,
                section.id,
                period_str,
                room,
                section.enrollment()
            ));
        }
    }

    Some(lines.join("\n"))
}
