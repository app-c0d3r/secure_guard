//! Communication module
//! 
//! Handles all communication between the agent and the SecureGuard backend server.
//! Supports WebSocket for real-time communication and HTTP for fallback.

pub mod client;
pub mod messages;
pub mod protocol;
pub mod encryption;

pub use client::Client;