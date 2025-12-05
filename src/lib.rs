//! School Scheduler - Constraint-based school schedule generator
//!
//! This library provides a complete solution for generating school schedules
//! using Integer Linear Programming (ILP) optimization.
//!
//! # Algorithm Overview
//!
//! The scheduler works in 5 phases:
//! 1. **Section Creation**: Create sections for each course and assign teachers
//! 2. **Time Slot Assignment**: Assign time slots with grade-aware conflict avoidance
//! 3. **Room Assignment**: Assign rooms based on capacity and features
//! 4. **ILP Student Assignment**: Optimize student-to-section assignments
//! 5. **Post-ILP Optimization**: Balance section enrollments
//!
//! # Example
//!
//! ```no_run
//! use school_scheduler::parser::load_input_from_dir;
//! use school_scheduler::scheduler::generate_schedule;
//! use school_scheduler::validator::validate_schedule;
//! use std::path::Path;
//!
//! let input = load_input_from_dir(Path::new("./data/demo")).unwrap();
//! let schedule = generate_schedule(&input, false).unwrap();
//! let validation = validate_schedule(&schedule, &input);
//! println!("Score: {:.1}", validation.total_score);
//! ```

pub mod error;
pub mod parser;
pub mod reporter;
pub mod scheduler;
pub mod types;
pub mod validator;

pub use error::{Result, SchedulerError};
