//! Security module for the SecureGuard agent

pub mod auth;

pub use auth::AuthManager;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub severity: String,
    pub description: String,
}