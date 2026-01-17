//! Mate entity commands
//!
//! Provides commands for managing fit analysis between mating features.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::core::identity::EntityId;
use tdt_core::entities::mate::{FitResult, Mate, MateType};
use tdt_core::services::common::SortDirection;
use tdt_core::services::mate::{
    CreateMate, MateFilter, MateService, MateSortField, MateStats, UpdateMate,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Mate summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct MateSummary {
    pub id: String,
    pub title: String,
    pub mate_type: String,
    pub feature_a: String,
    pub feature_b: String,
    pub fit_result: Option<String>,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Mate> for MateSummary {
    fn from(m: &Mate) -> Self {
        Self {
            id: m.id.to_string(),
            title: m.title.clone(),
            mate_type: m.mate_type.to_string(),
            feature_a: m.feature_a.id.to_string(),
            feature_b: m.feature_b.id.to_string(),
            fit_result: m.fit_analysis.as_ref().map(|f| f.fit_result.to_string()),
            status: format!("{:?}", m.status).to_lowercase(),
            author: m.author.clone(),
            created: m.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListMatesResult {
    pub items: Vec<MateSummary>,
    pub total_count: usize,
}

/// Recalc result
#[derive(Debug, Clone, Serialize)]
pub struct RecalcMateResult {
    pub mate: Mate,
    pub changed: bool,
    pub error: Option<String>,
}

/// Batch recalc summary
#[derive(Debug, Clone, Serialize)]
pub struct RecalcAllResult {
    pub processed: usize,
    pub changed: usize,
    pub errors: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListMatesParams {
    pub status: Option<Vec<String>>,
    pub mate_type: Option<String>,
    pub fit_result: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMateInput {
    pub title: String,
    pub author: String,
    pub feature_a: String,
    pub feature_b: String,
    pub mate_type: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateMateInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub mate_type: Option<String>,
    pub status: Option<String>,
}

// ============================================================================
// Conversion helpers
// ============================================================================

fn parse_status(s: &str) -> Option<Status> {
    match s.to_lowercase().as_str() {
        "draft" => Some(Status::Draft),
        "review" => Some(Status::Review),
        "approved" => Some(Status::Approved),
        "released" => Some(Status::Released),
        "obsolete" => Some(Status::Obsolete),
        _ => None,
    }
}

fn parse_mate_type(s: &str) -> Option<MateType> {
    match s.to_lowercase().as_str() {
        "clearance" => Some(MateType::Clearance),
        "transition" => Some(MateType::Transition),
        "interference" => Some(MateType::Interference),
        _ => None,
    }
}

fn parse_fit_result(s: &str) -> Option<FitResult> {
    match s.to_lowercase().as_str() {
        "clearance" => Some(FitResult::Clearance),
        "transition" => Some(FitResult::Transition),
        "interference" => Some(FitResult::Interference),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> MateSortField {
    match s.to_lowercase().as_str() {
        "id" => MateSortField::Id,
        "title" => MateSortField::Title,
        "mate_type" | "type" => MateSortField::MateType,
        "fit_result" | "fit" => MateSortField::FitResult,
        "status" => MateSortField::Status,
        "author" => MateSortField::Author,
        _ => MateSortField::Created,
    }
}

fn build_mate_filter(params: &ListMatesParams) -> MateFilter {
    use tdt_core::services::common::CommonFilter;

    let common = CommonFilter {
        status: params.status.as_ref().and_then(|v| {
            let statuses: Vec<Status> = v.iter().filter_map(|s| parse_status(s)).collect();
            if statuses.is_empty() { None } else { Some(statuses) }
        }),
        search: params.search.clone(),
        limit: params.limit,
        ..Default::default()
    };

    let sort = params.sort_by.as_ref().map(|s| parse_sort_field(s)).unwrap_or_default();
    let sort_direction = if params.sort_desc.unwrap_or(false) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    MateFilter {
        common,
        mate_type: params.mate_type.as_ref().and_then(|t| parse_mate_type(t)),
        fit_result: params.fit_result.as_ref().and_then(|f| parse_fit_result(f)),
        recent_days: None,
        sort,
        sort_direction,
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_mates(
    params: Option<ListMatesParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListMatesResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = MateService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_mate_filter(&params);
    let mates = service.list(&filter)?;

    Ok(ListMatesResult {
        total_count: mates.len(),
        items: mates.iter().map(MateSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_mate(id: String, state: State<'_, AppState>) -> CommandResult<Option<Mate>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = MateService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn create_mate(input: CreateMateInput, state: State<'_, AppState>) -> CommandResult<Mate> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mate = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = MateService::new(project, cache);

        let create = CreateMate {
            title: input.title,
            author: input.author,
            feature_a: EntityId::parse(&input.feature_a)
                .map_err(|e| CommandError::InvalidInput(format!("Invalid feature_a ID: {}", e)))?,
            feature_b: EntityId::parse(&input.feature_b)
                .map_err(|e| CommandError::InvalidInput(format!("Invalid feature_b ID: {}", e)))?,
            mate_type: parse_mate_type(&input.mate_type).unwrap_or(MateType::Clearance),
            description: input.description,
            notes: input.notes,
            tags: input.tags.unwrap_or_default(),
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(mate)
}

#[tauri::command]
pub async fn update_mate(id: String, input: UpdateMateInput, state: State<'_, AppState>) -> CommandResult<Mate> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mate = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = MateService::new(project, cache);

        let update = UpdateMate {
            title: input.title,
            description: input.description.map(Some),
            notes: input.notes.map(Some),
            mate_type: input.mate_type.and_then(|t| parse_mate_type(&t)),
            status: input.status.and_then(|s| parse_status(&s)),
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(mate)
}

#[tauri::command]
pub async fn delete_mate(id: String, force: Option<bool>, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = MateService::new(project, cache);
        service.delete(&id, force.unwrap_or(false))?;
    }

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(())
}

#[tauri::command]
pub async fn recalc_mate(id: String, state: State<'_, AppState>) -> CommandResult<RecalcMateResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let result = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = MateService::new(project, cache);
        service.recalculate(&id)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(RecalcMateResult {
        mate: result.mate,
        changed: result.changed,
        error: result.error,
    })
}

#[tauri::command]
pub async fn recalc_all_mates(state: State<'_, AppState>) -> CommandResult<RecalcAllResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let results = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = MateService::new(project, cache);
        service.recalculate_all()?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    let changed = results.iter().filter(|r| r.changed).count();
    let errors = 0; // Errors would be returned as Err from the service

    Ok(RecalcAllResult {
        processed: results.len(),
        changed,
        errors,
    })
}

#[tauri::command]
pub async fn get_mate_stats(state: State<'_, AppState>) -> CommandResult<MateStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = MateService::new(project, cache);
    Ok(service.stats()?)
}
