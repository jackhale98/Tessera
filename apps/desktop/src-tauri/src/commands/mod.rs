//! Tauri command handlers

pub mod capas;
pub mod components;
pub mod deviations;
pub mod entities;
pub mod lots;
pub mod ncrs;
pub mod project;
pub mod requirements;
pub mod risks;
pub mod traceability;

// Re-export all commands for registration
pub use capas::*;
pub use components::*;
pub use deviations::*;
pub use entities::*;
pub use lots::*;
pub use ncrs::*;
pub use project::*;
pub use requirements::*;
pub use risks::*;
pub use traceability::*;
