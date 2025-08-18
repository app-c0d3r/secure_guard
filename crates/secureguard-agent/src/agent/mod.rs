//! Agent core module
//! 
//! This module contains the main agent logic, including system monitoring,
//! data collection, and coordination of all agent services.

pub mod core;
pub mod monitor;
pub mod scanner;
pub mod collector;
pub mod updater;

pub use core::AgentCore;