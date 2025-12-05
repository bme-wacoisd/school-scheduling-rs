use serde::{Deserialize, Serialize};

/// Represents a specific time period in the week
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Period {
    /// Day of week (0-4 for Mon-Fri)
    pub day: u8,
    /// Period within the day (0-7 for periods 1-8)
    pub slot: u8,
}

impl Period {
    pub fn new(day: u8, slot: u8) -> Self {
        Self { day, slot }
    }

    /// Convert to a linear index (0-39 for 5 days * 8 periods)
    pub fn to_linear(&self, periods_per_day: u8) -> usize {
        (self.day as usize) * (periods_per_day as usize) + (self.slot as usize)
    }

    /// Create from a linear index
    pub fn from_linear(index: usize, periods_per_day: u8) -> Self {
        let day = (index / periods_per_day as usize) as u8;
        let slot = (index % periods_per_day as usize) as u8;
        Self { day, slot }
    }

    /// Human-readable day name
    pub fn day_name(&self) -> &'static str {
        match self.day {
            0 => "Monday",
            1 => "Tuesday",
            2 => "Wednesday",
            3 => "Thursday",
            4 => "Friday",
            _ => "Unknown",
        }
    }

    /// Human-readable format
    pub fn display(&self) -> String {
        format!("{} Period {}", self.day_name(), self.slot + 1)
    }
}

impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "D{}P{}", self.day + 1, self.slot + 1)
    }
}
