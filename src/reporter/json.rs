use crate::error::Result;
use crate::types::Schedule;

/// Generate JSON report of the schedule
pub fn generate_json_report(schedule: &Schedule) -> Result<String> {
    Ok(serde_json::to_string_pretty(schedule)?)
}

/// Summary statistics as JSON
#[derive(serde::Serialize)]
pub struct JsonSummary {
    pub total_sections: usize,
    pub total_assignments: usize,
    pub unassigned_count: usize,
    pub solve_time_ms: u64,
    pub score: f64,
}

pub fn generate_json_summary(schedule: &Schedule) -> Result<String> {
    let summary = JsonSummary {
        total_sections: schedule.sections.len(),
        total_assignments: schedule.total_assignments(),
        unassigned_count: schedule.unassigned.len(),
        solve_time_ms: schedule.metadata.solve_time_ms,
        score: schedule.metadata.score,
    };

    Ok(serde_json::to_string_pretty(&summary)?)
}
