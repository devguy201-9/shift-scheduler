use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[sqlx(type_name = "shift_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShiftType {
    Morning,
    Evening,
    DayOff,
}

impl fmt::Display for ShiftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ShiftType::Morning => "MORNING",
            ShiftType::Evening => "EVENING",
            ShiftType::DayOff => "DAY_OFF",
        };
        write!(f, "{}", value)
    }
}

impl FromStr for ShiftType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MORNING" => Ok(Self::Morning),
            "EVENING" => Ok(Self::Evening),
            "DAY_OFF" => Ok(Self::DayOff),
            _ => Err(format!("Invalid ShiftType: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[sqlx(type_name = "staff_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StaffStatus {
    Active,
    Inactive,
}

impl fmt::Display for StaffStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            StaffStatus::Active => "ACTIVE",
            StaffStatus::Inactive => "INACTIVE",
        };
        write!(f, "{}", value)
    }
}

impl FromStr for StaffStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(Self::Active),
            "INACTIVE" => Ok(Self::Inactive),
            _ => Err(format!("Invalid StaffStatus: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Type)]
#[sqlx(type_name = "job_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Processing => "PROCESSING",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
        }
    }
}

impl FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(Self::Pending),
            "PROCESSING" => Ok(Self::Processing),
            "COMPLETED" => Ok(Self::Completed),
            "FAILED" => Ok(Self::Failed),
            _ => Err(format!("Invalid JobStatus: {}", s)),
        }
    }
}
