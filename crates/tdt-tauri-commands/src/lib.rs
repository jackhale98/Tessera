//! Shared Tauri command handlers for TDT desktop and mobile apps
//!
//! This crate contains all the Tauri IPC command handlers, application state,
//! and error types shared between the desktop and mobile TDT applications.

pub mod commands;
pub mod error;
pub mod state;

// Re-export commonly used types
pub use error::{CommandError, CommandResult};
pub use state::AppState;
