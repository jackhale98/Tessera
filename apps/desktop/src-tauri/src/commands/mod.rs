//! Tauri command handlers

pub mod components;
pub mod entities;
pub mod project;
pub mod requirements;
pub mod risks;
pub mod traceability;

// Re-export all commands for registration
pub use components::*;
pub use entities::*;
pub use project::*;
pub use requirements::*;
pub use risks::*;
pub use traceability::*;
