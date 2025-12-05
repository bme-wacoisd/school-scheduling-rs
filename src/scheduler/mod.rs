mod section_creator;
mod time_assigner;
mod room_assigner;
mod ilp_solver;
mod optimizer;

pub use section_creator::*;
pub use time_assigner::*;
pub use room_assigner::*;
pub use ilp_solver::*;
pub use optimizer::*;

use crate::error::Result;
use crate::types::{Schedule, ScheduleInput, ScheduleMetadata};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;

/// Main entry point for schedule generation
pub fn generate_schedule(input: &ScheduleInput, quiet: bool) -> Result<Schedule> {
    let start_time = Instant::now();

    let progress = if quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    };

    // Phase 1: Create sections
    progress.set_message("Creating sections...");
    progress.set_position(10);
    let mut sections = create_sections(&input.courses, &input.teachers);

    // Phase 2: Assign time slots (CRITICAL)
    progress.set_message("Assigning time slots...");
    progress.set_position(20);
    assign_time_slots(&mut sections, &input.courses, &input.teachers, &input.config);

    // Phase 3: Assign rooms
    progress.set_message("Assigning rooms...");
    progress.set_position(30);
    assign_rooms(&mut sections, &input.rooms, &input.courses);

    // Phase 4: ILP student assignment
    progress.set_message("Solving student assignments (ILP)...");
    progress.set_position(40);
    let (assigned_sections, unassigned) = solve_student_assignment(
        sections,
        &input.students,
        &input.courses,
        &progress,
    )?;

    // Phase 5: Post-ILP optimization
    progress.set_message("Optimizing section balance...");
    progress.set_position(90);
    let optimized_sections = optimize_section_balance(assigned_sections);

    progress.set_message("Complete!");
    progress.set_position(100);
    progress.finish_with_message("Schedule generated successfully");

    let elapsed = start_time.elapsed();

    Ok(Schedule {
        sections: optimized_sections,
        unassigned,
        metadata: ScheduleMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            algorithm_version: env!("CARGO_PKG_VERSION").to_string(),
            score: 0.0, // Will be calculated by validator
            solve_time_ms: elapsed.as_millis() as u64,
        },
    })
}
