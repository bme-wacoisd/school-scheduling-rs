use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use school_scheduler::parser::{load_input_from_dir, validate_input};
use school_scheduler::reporter::{
    generate_reports, generate_student_schedule, generate_teacher_schedule, print_summary,
    OutputFormat,
};
use school_scheduler::scheduler::generate_schedule;
use school_scheduler::types::{StudentId, TeacherId};
use school_scheduler::validator::validate_schedule;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "school-scheduler")]
#[command(about = "Constraint-based school schedule generator")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run demo with sample data
    Demo {
        /// Only save if score improves or matches previous best
        #[arg(long)]
        monotonic: bool,
    },

    /// Generate a schedule from input data
    Schedule {
        /// Directory containing input JSON files
        #[arg(short, long)]
        data: PathBuf,

        /// Output directory for schedule files
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,

        /// Output format(s): json, markdown, text, or all
        #[arg(short, long, default_value = "all")]
        format: String,

        /// Suppress progress output, print JSON summary only
        #[arg(short, long)]
        quiet: bool,

        /// Only save if score improves or matches previous best
        #[arg(long)]
        monotonic: bool,
    },

    /// Validate an existing schedule
    Validate {
        /// Path to schedule.json file
        #[arg(short, long)]
        schedule: PathBuf,

        /// Directory containing input data for validation
        #[arg(short, long)]
        data: PathBuf,

        /// Show detailed validation results
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate reports from a schedule
    Report {
        /// Path to schedule.json file
        #[arg(short, long)]
        schedule: PathBuf,

        /// Directory containing input data
        #[arg(short, long)]
        data: PathBuf,

        /// Output format: json, markdown, or text
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Generate schedule for specific student ID
        #[arg(long)]
        student: Option<String>,

        /// Generate schedule for specific teacher ID
        #[arg(long)]
        teacher: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Demo { monotonic } => run_demo(monotonic),
        Commands::Schedule {
            data,
            output,
            format,
            quiet,
            monotonic,
        } => run_schedule(&data, &output, &format, quiet, monotonic),
        Commands::Validate {
            schedule,
            data,
            verbose,
        } => run_validate(&schedule, &data, verbose),
        Commands::Report {
            schedule,
            data,
            format,
            student,
            teacher,
        } => run_report(&schedule, &data, &format, student, teacher),
    }
}

fn run_demo(monotonic: bool) -> Result<()> {
    println!("{}", "School Scheduler Demo".bold().cyan());
    println!("{}", "─".repeat(40));

    let demo_path = PathBuf::from("data/demo");
    let output_path = PathBuf::from("output");

    if !demo_path.join("students.json").exists() {
        println!(
            "{}",
            "Demo data not found. Creating sample data...".yellow()
        );
        create_demo_data(&demo_path)?;
    }

    // Load baseline score if monotonic mode and schedule exists
    let baseline_score = if monotonic {
        load_baseline_score(&output_path.join("schedule.json"))
    } else {
        None
    };

    if let Some(score) = baseline_score {
        println!("Baseline score: {:.1}/100", score);
    }

    println!("Loading demo data from: {}", demo_path.display());

    let input = load_input_from_dir(&demo_path).context("Failed to load demo data")?;

    // Validate input
    let validation_result = validate_input(&input)?;
    if !validation_result.warnings.is_empty() {
        for warning in &validation_result.warnings {
            println!("{} {}", "Warning:".yellow(), warning);
        }
    }

    println!(
        "Loaded {} students, {} teachers, {} courses, {} rooms",
        input.students.len(),
        input.teachers.len(),
        input.courses.len(),
        input.rooms.len()
    );

    // Generate schedule
    println!("\nGenerating schedule...\n");
    let schedule = generate_schedule(&input, false)?;

    // Validate
    let validation = validate_schedule(&schedule, &input);

    // Check if we should save (monotonic mode)
    let should_save = match baseline_score {
        Some(baseline) if monotonic => {
            if validation.total_score >= baseline {
                if validation.total_score > baseline {
                    println!(
                        "{}",
                        format!("✓ Improved: {:.1} → {:.1}", baseline, validation.total_score)
                            .green()
                            .bold()
                    );
                } else {
                    println!(
                        "{}",
                        format!("= Matched: {:.1}", validation.total_score).cyan()
                    );
                }
                true
            } else {
                println!(
                    "{}",
                    format!(
                        "✗ Regression: {:.1} → {:.1} (not saving)",
                        baseline, validation.total_score
                    )
                    .red()
                    .bold()
                );
                false
            }
        }
        _ => true,
    };

    // Print summary
    print_summary(&schedule, &validation);

    // Write output only if should_save
    if should_save {
        generate_reports(
            &schedule,
            &input,
            &validation,
            &output_path,
            &[OutputFormat::Json, OutputFormat::Markdown, OutputFormat::Text],
        )?;

        println!(
            "Reports written to: {}",
            output_path.display().to_string().green()
        );
    }

    Ok(())
}

fn run_schedule(data: &PathBuf, output: &PathBuf, format: &str, quiet: bool, monotonic: bool) -> Result<()> {
    let input = load_input_from_dir(data).context("Failed to load input data")?;

    // Load baseline score if monotonic mode
    let baseline_score = if monotonic {
        load_baseline_score(&output.join("schedule.json"))
    } else {
        None
    };

    if !quiet {
        validate_input(&input)?;
        if let Some(score) = baseline_score {
            println!("Baseline score: {:.1}/100", score);
        }
        println!(
            "Loaded {} students, {} teachers, {} courses, {} rooms",
            input.students.len(),
            input.teachers.len(),
            input.courses.len(),
            input.rooms.len()
        );
    }

    let schedule = generate_schedule(&input, quiet)?;
    let validation = validate_schedule(&schedule, &input);

    // Check if we should save (monotonic mode)
    let should_save = match baseline_score {
        Some(baseline) if monotonic => {
            if validation.total_score >= baseline {
                if !quiet {
                    if validation.total_score > baseline {
                        println!(
                            "{}",
                            format!("✓ Improved: {:.1} → {:.1}", baseline, validation.total_score)
                                .green()
                                .bold()
                        );
                    } else {
                        println!(
                            "{}",
                            format!("= Matched: {:.1}", validation.total_score).cyan()
                        );
                    }
                }
                true
            } else {
                if !quiet {
                    println!(
                        "{}",
                        format!(
                            "✗ Regression: {:.1} → {:.1} (not saving)",
                            baseline, validation.total_score
                        )
                        .red()
                        .bold()
                    );
                }
                false
            }
        }
        _ => true,
    };

    if should_save {
        let formats = parse_formats(format);
        generate_reports(&schedule, &input, &validation, output, &formats)?;
    }

    if quiet {
        // Print JSON summary only
        let summary = school_scheduler::reporter::generate_json_summary(&schedule)?;
        println!("{}", summary);
    } else {
        print_summary(&schedule, &validation);
        if should_save {
            println!("Reports written to: {}", output.display().to_string().green());
        }
    }

    Ok(())
}

/// Load the score from an existing schedule file
fn load_baseline_score(path: &PathBuf) -> Option<f64> {
    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(path).ok()?;
    let schedule: school_scheduler::types::Schedule = serde_json::from_str(&content).ok()?;

    // The score is stored in metadata, but we need to recalculate for accuracy
    // For now, just return the stored score
    Some(schedule.metadata.score)
}

fn run_validate(schedule_path: &PathBuf, data: &PathBuf, verbose: bool) -> Result<()> {
    let input = load_input_from_dir(data)?;

    let schedule_json = std::fs::read_to_string(schedule_path)?;
    let schedule: school_scheduler::types::Schedule = serde_json::from_str(&schedule_json)?;

    let validation = validate_schedule(&schedule, &input);

    if validation.is_valid {
        println!("{}", "✓ Schedule is valid".green().bold());
    } else {
        println!("{}", "✗ Schedule has violations".red().bold());
        for v in &validation.hard_violations {
            println!("  - {}: {}", v.constraint.red(), v.message);
        }
    }

    if verbose {
        println!("\n{}", "Soft Constraint Scores:".bold());
        for score in &validation.soft_scores {
            let pct = if score.max_score > 0.0 {
                (score.score / score.max_score) * 100.0
            } else {
                100.0
            };
            println!("  {}: {:.1}%", score.constraint, pct);
        }

        println!("\n{}", "Statistics:".bold());
        println!("  Sections: {}", validation.statistics.total_sections);
        println!("  Assignments: {}", validation.statistics.total_assignments);
        println!(
            "  Unassigned: {} required, {} electives",
            validation.statistics.unassigned_required,
            validation.statistics.unassigned_electives
        );
    }

    println!("\nOverall Score: {:.1}/100", validation.total_score);

    Ok(())
}

fn run_report(
    schedule_path: &PathBuf,
    data: &PathBuf,
    _format: &str,
    student: Option<String>,
    teacher: Option<String>,
) -> Result<()> {
    let input = load_input_from_dir(data)?;

    let schedule_json = std::fs::read_to_string(schedule_path)?;
    let schedule: school_scheduler::types::Schedule = serde_json::from_str(&schedule_json)?;

    if let Some(student_id) = student {
        let id = StudentId(student_id);
        match generate_student_schedule(&schedule, &input, &id) {
            Some(report) => println!("{}", report),
            None => println!("Student not found"),
        }
    } else if let Some(teacher_id) = teacher {
        let id = TeacherId(teacher_id);
        match generate_teacher_schedule(&schedule, &input, &id) {
            Some(report) => println!("{}", report),
            None => println!("Teacher not found"),
        }
    } else {
        let validation = validate_schedule(&schedule, &input);
        print_summary(&schedule, &validation);
    }

    Ok(())
}

fn parse_formats(format: &str) -> Vec<OutputFormat> {
    if format == "all" {
        return vec![OutputFormat::Json, OutputFormat::Markdown, OutputFormat::Text];
    }

    format
        .split(',')
        .filter_map(|f| match f.trim().to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "markdown" | "md" => Some(OutputFormat::Markdown),
            "text" | "txt" => Some(OutputFormat::Text),
            _ => None,
        })
        .collect()
}

fn create_demo_data(path: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(path)?;

    // Students
    let students = serde_json::json!([
        {"id": "s001", "name": "Alice Johnson", "grade": 10, "required_courses": ["math10", "eng10", "sci10"], "elective_preferences": ["art", "music"]},
        {"id": "s002", "name": "Bob Smith", "grade": 10, "required_courses": ["math10", "eng10", "sci10"], "elective_preferences": ["music", "art"]},
        {"id": "s003", "name": "Carol Davis", "grade": 10, "required_courses": ["math10", "eng10", "sci10"], "elective_preferences": ["art", "pe"]},
        {"id": "s004", "name": "David Wilson", "grade": 11, "required_courses": ["math11", "eng11", "sci11"], "elective_preferences": ["pe", "art"]},
        {"id": "s005", "name": "Eve Brown", "grade": 11, "required_courses": ["math11", "eng11", "sci11"], "elective_preferences": ["music", "pe"]},
        {"id": "s006", "name": "Frank Miller", "grade": 11, "required_courses": ["math11", "eng11", "sci11"], "elective_preferences": ["art", "music"]},
        {"id": "s007", "name": "Grace Lee", "grade": 12, "required_courses": ["math12", "eng12", "gov"], "elective_preferences": ["pe", "music"]},
        {"id": "s008", "name": "Henry Taylor", "grade": 12, "required_courses": ["math12", "eng12", "gov"], "elective_preferences": ["art", "pe"]},
        {"id": "s009", "name": "Ivy Chen", "grade": 12, "required_courses": ["math12", "eng12", "gov"], "elective_preferences": ["music", "art"]},
        {"id": "s010", "name": "Jack Robinson", "grade": 10, "required_courses": ["math10", "eng10", "sci10"], "elective_preferences": ["pe", "music"]}
    ]);
    std::fs::write(
        path.join("students.json"),
        serde_json::to_string_pretty(&students)?,
    )?;

    // Teachers
    let teachers = serde_json::json!([
        {"id": "t001", "name": "Ms. Anderson", "subjects": ["math10", "math11", "math12"], "max_sections": 4, "unavailable": []},
        {"id": "t002", "name": "Mr. Baker", "subjects": ["eng10", "eng11", "eng12"], "max_sections": 4, "unavailable": []},
        {"id": "t003", "name": "Dr. Clark", "subjects": ["sci10", "sci11"], "max_sections": 3, "unavailable": []},
        {"id": "t004", "name": "Ms. Davis", "subjects": ["gov"], "max_sections": 2, "unavailable": []},
        {"id": "t005", "name": "Mr. Evans", "subjects": ["art", "music"], "max_sections": 4, "unavailable": []},
        {"id": "t006", "name": "Coach Fisher", "subjects": ["pe"], "max_sections": 4, "unavailable": []}
    ]);
    std::fs::write(
        path.join("teachers.json"),
        serde_json::to_string_pretty(&teachers)?,
    )?;

    // Courses
    let courses = serde_json::json!([
        {"id": "math10", "name": "Algebra 2", "max_students": 25, "grade_restrictions": [10], "required_features": [], "sections": 1},
        {"id": "math11", "name": "Pre-Calculus", "max_students": 25, "grade_restrictions": [11], "required_features": [], "sections": 1},
        {"id": "math12", "name": "Calculus", "max_students": 25, "grade_restrictions": [12], "required_features": [], "sections": 1},
        {"id": "eng10", "name": "English 10", "max_students": 25, "grade_restrictions": [10], "required_features": [], "sections": 1},
        {"id": "eng11", "name": "English 11", "max_students": 25, "grade_restrictions": [11], "required_features": [], "sections": 1},
        {"id": "eng12", "name": "English 12", "max_students": 25, "grade_restrictions": [12], "required_features": [], "sections": 1},
        {"id": "sci10", "name": "Biology", "max_students": 24, "grade_restrictions": [10], "required_features": ["lab"], "sections": 1},
        {"id": "sci11", "name": "Chemistry", "max_students": 24, "grade_restrictions": [11], "required_features": ["lab"], "sections": 1},
        {"id": "gov", "name": "Government", "max_students": 25, "grade_restrictions": [12], "required_features": [], "sections": 1},
        {"id": "art", "name": "Art", "max_students": 20, "grade_restrictions": null, "required_features": ["art_room"], "sections": 2},
        {"id": "music", "name": "Music", "max_students": 25, "grade_restrictions": null, "required_features": [], "sections": 2},
        {"id": "pe", "name": "Physical Education", "max_students": 30, "grade_restrictions": null, "required_features": ["gym"], "sections": 2}
    ]);
    std::fs::write(
        path.join("courses.json"),
        serde_json::to_string_pretty(&courses)?,
    )?;

    // Rooms
    let rooms = serde_json::json!([
        {"id": "101", "name": "Room 101", "capacity": 30, "features": [], "unavailable": []},
        {"id": "102", "name": "Room 102", "capacity": 30, "features": [], "unavailable": []},
        {"id": "103", "name": "Room 103", "capacity": 30, "features": [], "unavailable": []},
        {"id": "104", "name": "Room 104", "capacity": 30, "features": [], "unavailable": []},
        {"id": "201", "name": "Science Lab", "capacity": 24, "features": ["lab"], "unavailable": []},
        {"id": "301", "name": "Art Studio", "capacity": 20, "features": ["art_room"], "unavailable": []},
        {"id": "gym", "name": "Gymnasium", "capacity": 60, "features": ["gym"], "unavailable": []}
    ]);
    std::fs::write(
        path.join("rooms.json"),
        serde_json::to_string_pretty(&rooms)?,
    )?;

    println!("{}", "Demo data created successfully!".green());
    Ok(())
}
