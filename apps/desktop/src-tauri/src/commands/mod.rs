//! Tauri command handlers

pub mod assemblies;
pub mod capas;
pub mod components;
pub mod controls;
pub mod deviations;
pub mod entities;
pub mod features;
pub mod hazards;
pub mod lots;
pub mod mates;
pub mod ncrs;
pub mod processes;
pub mod project;
pub mod quotes;
pub mod requirements;
pub mod results;
pub mod risks;
pub mod stackups;
pub mod suppliers;
pub mod tests;
pub mod traceability;
pub mod work_instructions;

// Re-export all commands for registration
pub use assemblies::*;
pub use capas::*;
pub use components::*;
pub use controls::*;
pub use deviations::*;
pub use entities::*;
pub use features::*;
pub use hazards::*;
pub use lots::*;
pub use mates::*;
pub use ncrs::*;
pub use processes::*;
pub use project::*;
pub use quotes::*;
pub use requirements::*;
pub use results::*;
pub use risks::*;
pub use stackups::*;
pub use suppliers::*;
pub use tests::*;
pub use traceability::*;
pub use work_instructions::*;
