use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "election_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum ElectionStatus {
    Draft,
    Open,
    Closing,
    Sealed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "access_type", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum AccessType {
    Public,
    Private,
}

// Optional: Provide conversion to/from string for serialization
impl ElectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ElectionStatus::Draft => "DRAFT",
            ElectionStatus::Open => "OPEN",
            ElectionStatus::Closing => "CLOSING",
            ElectionStatus::Sealed => "SEALED",
        }
    }
}

impl AccessType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessType::Public => "PUBLIC",
            AccessType::Private => "PRIVATE",
        }
    }
}
