use serde::{Deserialize, Serialize};
use super::{Period, RoomId};

/// Represents a physical room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub capacity: u32,
    /// Features this room has (e.g., "lab", "computers", "projector")
    #[serde(default)]
    pub features: Vec<String>,
    /// Periods when the room is unavailable
    #[serde(default)]
    pub unavailable: Vec<Period>,
}

impl Room {
    /// Check if room has all required features
    pub fn has_features(&self, required: &[String]) -> bool {
        required.iter().all(|f| self.features.contains(f))
    }

    /// Check if room is available during a period
    pub fn is_available(&self, period: &Period) -> bool {
        !self.unavailable.contains(period)
    }
}
