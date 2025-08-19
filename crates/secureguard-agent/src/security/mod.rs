//! Security module for the SecureGuard agent

pub mod auth;

pub use auth::AuthManager;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub severity: String,
    pub description: String,
}
