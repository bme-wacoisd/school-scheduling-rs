use crate::types::{CourseId, Schedule, ScheduleInput};
use crate::validator::ValidationReport;
use colored::Colorize;
use std::collections::HashMap;

/// Generate a plain text report (with colors for terminal)
pub fn generate_text_report(
    schedule: &Schedule,
    input: &ScheduleInput,
    validation: &ValidationReport,
) -> String {
    let mut lines = Vec::new();

    lines.push("═".repeat(60));
    lines.push("               SCHEDULE REPORT".to_string());
    lines.push("═".repeat(60));
    lines.push(String::new());

    // Summary
    lines.push(format!(
        "Generated: {}",
        schedule.metadata.generated_at
    ));
    lines.push(format!(
        "Solve Time: {}ms",
        schedule.metadata.solve_time_ms
    ));
    lines.push(String::new());

    lines.push("─".repeat(40));
    lines.push("STATISTICS".to_string());
    lines.push("─".repeat(40));
    lines.push(format!(
        "  Sections:      {}",
        validation.statistics.total_sections
    ));
    lines.push(format!(
        "  Students:      {}",
        validation.statistics.total_students
    ));
    lines.push(format!(
        "  Assignments:   {}",
        validation.statistics.total_assignments
    ));
    lines.push(format!(
        "  Unassigned:    {} required, {} electives",
        validation.statistics.unassigned_required,
        validation.statistics.unassigned_electives
    ));
    lines.push(format!(
        "  Fill Rate:     {:.1}%",
        validation.statistics.avg_section_fill_rate
    ));
    lines.push(format!(
        "  Score:         {:.1}/100",
        validation.total_score
    ));
    lines.push(String::new());

    // Validation
    lines.push("─".repeat(40));
    if validation.is_valid {
        lines.push("VALIDATION: PASSED".green().to_string());
    } else {
        lines.push("VALIDATION: FAILED".red().to_string());
        for v in &validation.hard_violations {
            lines.push(format!("  ! {}: {}", v.constraint, v.message));
        }
    }
    lines.push("─".repeat(40));
    lines.push(String::new());

    // Course sections
    let course_map: HashMap<&CourseId, &str> = input
        .courses
        .iter()
        .map(|c| (&c.id, c.name.as_str()))
        .collect();

    let mut by_course: HashMap<&CourseId, Vec<&crate::types::Section>> = HashMap::new();
    for section in &schedule.sections {
        by_course.entry(&section.course_id).or_default().push(section);
    }

    lines.push("COURSE SECTIONS".to_string());
    lines.push("─".repeat(40));

    for (course_id, sections) in &by_course {
        let name = course_map.get(course_id).unwrap_or(&"Unknown");
        let total_enrolled: usize = sections.iter().map(|s| s.enrollment()).sum();
        let total_capacity: u32 = sections.iter().map(|s| s.capacity).sum();

        lines.push(format!(
            "\n{} ({} sections, {}/{} students)",
            name.bold(),
            sections.len(),
            total_enrolled,
            total_capacity
        ));

        for section in sections {
            let period = section
                .periods
                .first()
                .map(|p| format!("P{}", p.slot + 1))
                .unwrap_or_else(|| "TBD".to_string());

            let teacher = section
                .teacher_id
                .as_ref()
                .and_then(|tid| input.teachers.iter().find(|t| &t.id == tid))
                .map(|t| t.name.as_str())
                .unwrap_or("TBD");

            let fill_pct = (section.enrollment() as f64 / section.capacity as f64) * 100.0;
            let fill_indicator = if fill_pct >= 90.0 {
                "●".red()
            } else if fill_pct >= 70.0 {
                "●".yellow()
            } else {
                "●".green()
            };

            lines.push(format!(
                "  {} {} | {} | {} | {}/{} {}",
                fill_indicator,
                section.id,
                period,
                teacher,
                section.enrollment(),
                section.capacity,
                format!("({:.0}%)", fill_pct).dimmed()
            ));
        }
    }

    lines.push(String::new());
    lines.push("═".repeat(60));

    lines.join("\n")
}

/// Print a quick summary to stdout
pub fn print_summary(schedule: &Schedule, validation: &ValidationReport) {
    println!();
    if validation.is_valid {
        println!("{}", "✓ Schedule generated successfully".green().bold());
    } else {
        println!("{}", "✗ Schedule has validation errors".red().bold());
    }
    println!();
    println!("  Sections:    {}", validation.statistics.total_sections);
    println!("  Assignments: {}", validation.statistics.total_assignments);
    println!(
        "  Unassigned:  {}",
        validation.statistics.unassigned_required + validation.statistics.unassigned_electives
    );
    println!("  Score:       {:.1}/100", validation.total_score);
    println!("  Time:        {}ms", schedule.metadata.solve_time_ms);
    println!();
}
