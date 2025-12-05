use crate::error::{Result, SchedulerError};
use crate::types::{
    Constraint, Course, Room, ScheduleConfig, ScheduleInput, Student, Teacher,
};
use std::fs;
use std::path::Path;

/// Load all input data from a directory
pub fn load_input_from_dir(dir: &Path) -> Result<ScheduleInput> {
    let students = load_students(&dir.join("students.json"))?;
    let teachers = load_teachers(&dir.join("teachers.json"))?;
    let courses = load_courses(&dir.join("courses.json"))?;
    let rooms = load_rooms(&dir.join("rooms.json"))?;
    let config = load_config_or_default(&dir.join("config.toml"));
    let constraints = default_constraints();

    Ok(ScheduleInput {
        students,
        teachers,
        courses,
        rooms,
        constraints,
        config,
    })
}

/// Load students from JSON file
pub fn load_students(path: &Path) -> Result<Vec<Student>> {
    load_json_file(path)
}

/// Load teachers from JSON file
pub fn load_teachers(path: &Path) -> Result<Vec<Teacher>> {
    load_json_file(path)
}

/// Load courses from JSON file
pub fn load_courses(path: &Path) -> Result<Vec<Course>> {
    load_json_file(path)
}

/// Load rooms from JSON file
pub fn load_rooms(path: &Path) -> Result<Vec<Room>> {
    load_json_file(path)
}

/// Load config from TOML file, or use defaults
pub fn load_config_or_default(path: &Path) -> ScheduleConfig {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => ScheduleConfig::default(),
        }
    } else {
        ScheduleConfig::default()
    }
}

/// Generic JSON file loader
fn load_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let path_str = path.display().to_string();
    let content = fs::read_to_string(path).map_err(|e| SchedulerError::FileRead {
        path: path_str.clone(),
        source: e,
    })?;

    serde_json::from_str(&content).map_err(|e| {
        SchedulerError::JsonParse {
            file: path_str,
            message: e.to_string(),
        }
        .into()
    })
}

/// Default set of constraints
fn default_constraints() -> Vec<Constraint> {
    vec![
        // Hard constraints
        Constraint::NoTeacherConflict,
        Constraint::NoStudentConflict,
        Constraint::NoRoomConflict,
        Constraint::RoomCapacity,
        Constraint::TeacherQualified,
        Constraint::TeacherAvailability,
        Constraint::RoomFeatures,
        Constraint::GradeRestriction,
        Constraint::TeacherMaxSections,
        // Soft constraints
        Constraint::BalancedSections { weight: 0.5 },
        Constraint::StudentElectivePreference { weight: 1.0 },
    ]
}
