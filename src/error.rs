use thiserror::Error;

/// Domain-specific errors for the scheduler
#[derive(Error, Debug)]
pub enum SchedulerError {
    // Input/Parse errors
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse JSON in '{file}': {message}")]
    JsonParse { file: String, message: String },

    #[error("Invalid constraint: {0}")]
    InvalidConstraint(String),

    // Data validation errors
    #[error("Student '{student_id}' references unknown course '{course_id}'")]
    UnknownCourse { student_id: String, course_id: String },

    #[error("Teacher '{teacher_id}' is not qualified to teach course '{course_id}'")]
    UnqualifiedTeacher {
        teacher_id: String,
        course_id: String,
    },

    #[error("Not enough sections for course '{course_id}': need {needed}, have {available}")]
    InsufficientSections {
        course_id: String,
        needed: u32,
        available: u32,
    },

    #[error("Course '{course_id}' has no qualified teachers")]
    NoQualifiedTeacher { course_id: String },

    #[error("Duplicate ID found: {id_type} '{id}'")]
    DuplicateId { id_type: String, id: String },

    // Solver errors
    #[error("ILP solver failed: {0}")]
    SolverFailed(String),

    #[error("No feasible solution found")]
    Infeasible,

    #[error("Solver timeout after {seconds} seconds")]
    SolverTimeout { seconds: u64 },

    // Validation errors
    #[error("Schedule violates hard constraint: {0}")]
    HardConstraintViolation(String),
}

/// Use anyhow::Result at application boundaries
pub type Result<T> = anyhow::Result<T>;
