//! Requirement-specific commands (stats only - CRUD via generic entities)

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::Serialize;
use tauri::State;
use tdt_core::services::RequirementService;

/// Requirement statistics
#[derive(Debug, Clone, Serialize)]
pub struct RequirementStatsResponse {
    pub total: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub unverified: usize,
    pub orphaned: usize,
    pub by_status: StatusCountsResponse,
}

/// Status counts
#[derive(Debug, Clone, Serialize)]
pub struct StatusCountsResponse {
    pub draft: usize,
    pub review: usize,
    pub approved: usize,
    pub released: usize,
    pub obsolete: usize,
}

/// Get requirement statistics
#[tauri::command]
pub async fn get_requirement_stats(
    state: State<'_, AppState>,
) -> CommandResult<RequirementStatsResponse> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RequirementService::new(project, cache);
    let stats = service.stats()?;

    Ok(RequirementStatsResponse {
        total: stats.total,
        inputs: stats.inputs,
        outputs: stats.outputs,
        unverified: stats.unverified,
        orphaned: stats.orphaned,
        by_status: StatusCountsResponse {
            draft: stats.by_status.draft,
            review: stats.by_status.review,
            approved: stats.by_status.approved,
            released: stats.by_status.released,
            obsolete: stats.by_status.obsolete,
        },
    })
}
