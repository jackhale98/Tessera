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
pub mod settings;
pub mod stackups;
pub mod suppliers;
pub mod tests;
pub mod traceability;
pub mod version_control;
pub mod work_instructions;

use tdt_core::core::identity::EntityPrefix;

/// Get directory name for an entity prefix (shared across all command modules)
pub(crate) fn entity_dir_name(prefix: EntityPrefix) -> &'static str {
    match prefix {
        EntityPrefix::Req => "requirements",
        EntityPrefix::Risk => "risks",
        EntityPrefix::Test => "verification/protocols",
        EntityPrefix::Rslt => "verification/results",
        EntityPrefix::Cmp => "bom/components",
        EntityPrefix::Asm => "bom/assemblies",
        EntityPrefix::Feat => "tolerances/features",
        EntityPrefix::Mate => "tolerances/mates",
        EntityPrefix::Tol => "tolerances/stackups",
        EntityPrefix::Proc => "manufacturing/processes",
        EntityPrefix::Ctrl => "manufacturing/controls",
        EntityPrefix::Work => "manufacturing/work_instructions",
        EntityPrefix::Lot => "manufacturing/lots",
        EntityPrefix::Dev => "manufacturing/deviations",
        EntityPrefix::Ncr => "manufacturing/ncrs",
        EntityPrefix::Capa => "manufacturing/capas",
        EntityPrefix::Quot => "bom/quotes",
        EntityPrefix::Sup => "bom/suppliers",
        EntityPrefix::Haz => "risks/hazards",
        EntityPrefix::Act => "actions",
    }
}

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
pub use settings::*;
pub use stackups::*;
pub use suppliers::*;
pub use tests::*;
pub use traceability::*;
pub use version_control::*;
pub use work_instructions::*;
