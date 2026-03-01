//! Deviation entity commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::cache::CachedDeviation;
use tdt_core::core::entity::Status;
use tdt_core::entities::dev::{
    AuthorizationLevel, Dev, DevStatus, DeviationCategory, DeviationType, RiskLevel,
};
use tdt_core::services::{
    CreateDeviation, DeviationFilter, DeviationService, DeviationSortField, DeviationStats,
    SortDirection, UpdateDeviation,
};

/// List parameters for deviations
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListDeviationsParams {
    pub status: Option<Vec<String>>,
    pub dev_status: Option<String>,
    pub deviation_type: Option<String>,
    pub category: Option<String>,
    pub risk_level: Option<String>,
    pub active_only: Option<bool>,
    pub recent_days: Option<u32>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// Deviation summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct DeviationSummary {
    pub id: String,
    pub title: String,
    pub deviation_number: Option<String>,
    pub deviation_type: String,
    pub category: String,
    pub risk_level: String,
    pub dev_status: String,
    pub status: String,
    pub effective_date: Option<String>,
    pub expiration_date: Option<String>,
    pub approved_by: Option<String>,
    pub approval_date: Option<String>,
    pub author: String,
    pub created: String,
}

impl From<&Dev> for DeviationSummary {
    fn from(dev: &Dev) -> Self {
        Self {
            id: dev.id.to_string(),
            title: dev.title.clone(),
            deviation_number: dev.deviation_number.clone(),
            deviation_type: dev.deviation_type.to_string(),
            category: dev.category.to_string(),
            risk_level: format!("{:?}", dev.risk.level).to_lowercase(),
            dev_status: dev.dev_status.to_string(),
            status: format!("{:?}", dev.status).to_lowercase(),
            effective_date: dev.effective_date.map(|d| d.to_string()),
            expiration_date: dev.expiration_date.map(|d| d.to_string()),
            approved_by: dev.approval.approved_by.clone(),
            approval_date: dev.approval.approval_date.map(|d| d.to_string()),
            author: dev.author.clone(),
            created: dev.created.to_rfc3339(),
        }
    }
}

impl From<&CachedDeviation> for DeviationSummary {
    fn from(cached: &CachedDeviation) -> Self {
        Self {
            id: cached.id.clone(),
            title: cached.title.clone(),
            deviation_number: cached.deviation_number.clone(),
            deviation_type: cached.deviation_type.clone().unwrap_or_default(),
            category: cached.category.clone().unwrap_or_default(),
            risk_level: cached
                .risk_level
                .clone()
                .unwrap_or_else(|| "low".to_string()),
            dev_status: cached.dev_status.clone().unwrap_or_default(),
            status: format!("{:?}", cached.status).to_lowercase(),
            effective_date: cached.effective_date.clone(),
            expiration_date: cached.expiration_date.clone(),
            approved_by: cached.approved_by.clone(),
            approval_date: cached.approval_date.clone(),
            author: cached.author.clone(),
            created: cached.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListDeviationsResult {
    pub items: Vec<DeviationSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Input for creating a deviation
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDeviationInput {
    pub title: String,
    pub deviation_number: Option<String>,
    pub deviation_type: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub risk_level: Option<String>,
    pub risk_assessment: Option<String>,
    pub effective_date: Option<String>,
    pub expiration_date: Option<String>,
    pub notes: Option<String>,
    pub author: String,
}

/// Input for updating a deviation
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDeviationInput {
    pub title: Option<String>,
    pub deviation_number: Option<String>,
    pub deviation_type: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub effective_date: Option<String>,
    pub expiration_date: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    pub dev_status: Option<String>,
}

/// Input for approval
#[derive(Debug, Clone, Deserialize)]
pub struct ApproveDeviationInput {
    pub approved_by: String,
    pub authorization_level: String,
    pub activate: Option<bool>,
}

/// Input for rejection
#[derive(Debug, Clone, Deserialize)]
pub struct RejectDeviationInput {
    pub reason: Option<String>,
}

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

fn parse_dev_status(s: &str) -> Option<DevStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(DevStatus::Pending),
        "approved" => Some(DevStatus::Approved),
        "active" => Some(DevStatus::Active),
        "expired" => Some(DevStatus::Expired),
        "closed" => Some(DevStatus::Closed),
        "rejected" => Some(DevStatus::Rejected),
        _ => None,
    }
}

fn parse_deviation_type(s: &str) -> Option<DeviationType> {
    match s.to_lowercase().as_str() {
        "temporary" | "temp" => Some(DeviationType::Temporary),
        "permanent" | "perm" => Some(DeviationType::Permanent),
        "emergency" | "emerg" => Some(DeviationType::Emergency),
        _ => None,
    }
}

fn parse_category(s: &str) -> Option<DeviationCategory> {
    match s.to_lowercase().as_str() {
        "material" | "mat" => Some(DeviationCategory::Material),
        "process" | "proc" => Some(DeviationCategory::Process),
        "equipment" | "equip" => Some(DeviationCategory::Equipment),
        "tooling" | "tool" => Some(DeviationCategory::Tooling),
        "specification" | "spec" => Some(DeviationCategory::Specification),
        "documentation" | "doc" => Some(DeviationCategory::Documentation),
        _ => None,
    }
}

fn parse_risk_level(s: &str) -> Option<RiskLevel> {
    match s.to_lowercase().as_str() {
        "low" => Some(RiskLevel::Low),
        "medium" | "med" => Some(RiskLevel::Medium),
        "high" => Some(RiskLevel::High),
        _ => None,
    }
}

fn parse_authorization_level(s: &str) -> Option<AuthorizationLevel> {
    match s.to_lowercase().as_str() {
        "engineering" | "eng" => Some(AuthorizationLevel::Engineering),
        "quality" | "qa" => Some(AuthorizationLevel::Quality),
        "management" | "mgmt" => Some(AuthorizationLevel::Management),
        _ => None,
    }
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// List deviations with optional filters
#[tauri::command]
pub async fn list_deviations(
    params: Option<ListDeviationsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListDeviationsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = DeviationService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = DeviationFilter {
        common: tdt_core::services::CommonFilter {
            status: params
                .status
                .map(|v| v.iter().filter_map(|s| parse_status(s)).collect()),
            search: params.search,
            tags: params.tags,
            limit: params.limit,
            offset: params.offset,
            ..Default::default()
        },
        dev_status: params.dev_status.and_then(|s| parse_dev_status(&s)),
        deviation_type: params.deviation_type.and_then(|s| parse_deviation_type(&s)),
        category: params.category.and_then(|s| parse_category(&s)),
        risk_level: params.risk_level.and_then(|s| parse_risk_level(&s)),
        active_only: params.active_only.unwrap_or(false),
        recent_days: params.recent_days,
        sort: params
            .sort_by
            .map(|s| match s.as_str() {
                "title" => DeviationSortField::Title,
                "deviation_number" => DeviationSortField::DeviationNumber,
                "deviation_type" => DeviationSortField::DeviationType,
                "category" => DeviationSortField::Category,
                "risk" => DeviationSortField::Risk,
                "dev_status" => DeviationSortField::DevStatus,
                "author" => DeviationSortField::Author,
                "created" => DeviationSortField::Created,
                _ => DeviationSortField::Created,
            })
            .unwrap_or(DeviationSortField::Created),
        sort_direction: if params.sort_desc.unwrap_or(true) {
            SortDirection::Descending
        } else {
            SortDirection::Ascending
        },
    };

    // Use cache for fast list views - all summary fields are in cache
    let cached_deviations = service.list_cached(&filter)?;
    let total_count = cached_deviations.len();

    Ok(ListDeviationsResult {
        items: cached_deviations
            .iter()
            .map(DeviationSummary::from)
            .collect(),
        total_count,
        has_more: false, // Limit already applied in filter
    })
}

/// Get a single deviation by ID
#[tauri::command]
pub async fn get_deviation(id: String, state: State<'_, AppState>) -> CommandResult<Option<Dev>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = DeviationService::new(project, cache);
    let dev = service.get(&id)?;

    Ok(dev)
}

/// Create a new deviation
#[tauri::command]
pub async fn create_deviation(
    input: CreateDeviationInput,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        let create = CreateDeviation {
            title: input.title,
            deviation_number: input.deviation_number,
            deviation_type: input
                .deviation_type
                .and_then(|s| parse_deviation_type(&s))
                .unwrap_or(DeviationType::Temporary),
            category: input
                .category
                .and_then(|s| parse_category(&s))
                .unwrap_or(DeviationCategory::Material),
            description: input.description,
            risk_level: input
                .risk_level
                .and_then(|s| parse_risk_level(&s))
                .unwrap_or(RiskLevel::Low),
            risk_assessment: input.risk_assessment,
            effective_date: input.effective_date.and_then(|s| parse_date(&s)),
            expiration_date: input.expiration_date.and_then(|s| parse_date(&s)),
            notes: input.notes,
            status: None,
            author: input.author,
        };

        service.create(create)?
    };

    // Sync cache to pick up the new entity
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Update an existing deviation
#[tauri::command]
pub async fn update_deviation(
    id: String,
    input: UpdateDeviationInput,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        let update = UpdateDeviation {
            title: input.title,
            deviation_number: input.deviation_number.map(Some),
            deviation_type: input.deviation_type.and_then(|s| parse_deviation_type(&s)),
            category: input.category.and_then(|s| parse_category(&s)),
            description: input.description.map(Some),
            effective_date: input.effective_date.map(|s| parse_date(&s)),
            expiration_date: input.expiration_date.map(|s| parse_date(&s)),
            notes: input.notes.map(Some),
            status: input.status.and_then(|s| parse_status(&s)),
            dev_status: input.dev_status.and_then(|s| parse_dev_status(&s)),
        };

        service.update(&id, update)?
    };

    // Sync cache to pick up the changes
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Delete a deviation
#[tauri::command]
pub async fn delete_deviation(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);
        service.delete(&id, false)?;
    }

    // Sync cache to remove the deleted entity
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Approve a deviation
#[tauri::command]
pub async fn approve_deviation(
    id: String,
    input: ApproveDeviationInput,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let guard = tdt_core::services::WorkflowGuard::load(project);
        let service = DeviationService::new(project, cache).with_workflow(guard);

        let auth_level = parse_authorization_level(&input.authorization_level)
            .unwrap_or(AuthorizationLevel::Engineering);

        service.approve(
            &id,
            input.approved_by,
            auth_level,
            input.activate.unwrap_or(false),
        )?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Reject a deviation
#[tauri::command]
pub async fn reject_deviation(
    id: String,
    input: Option<RejectDeviationInput>,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.reject(&id, input.and_then(|i| i.reason))?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Activate an approved deviation
#[tauri::command]
pub async fn activate_deviation(id: String, state: State<'_, AppState>) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.activate(&id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Close/expire a deviation
#[tauri::command]
pub async fn close_deviation(
    id: String,
    reason: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.close(&id, reason)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Set a deviation to expired status
#[tauri::command]
pub async fn expire_deviation(id: String, state: State<'_, AppState>) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.expire(&id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Add a mitigation measure
#[tauri::command]
pub async fn add_deviation_mitigation(
    id: String,
    mitigation: String,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.add_mitigation(&id, mitigation)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Get deviation statistics
#[tauri::command]
pub async fn get_deviation_stats(state: State<'_, AppState>) -> CommandResult<DeviationStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = DeviationService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}

/// Set risk level and assessment for a deviation
#[tauri::command]
pub async fn set_deviation_risk(
    id: String,
    level: String,
    assessment: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        let risk_level = parse_risk_level(&level).unwrap_or(RiskLevel::Low);
        service.set_risk(&id, risk_level, assessment)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Add a process link to a deviation
#[tauri::command]
pub async fn add_deviation_process_link(
    id: String,
    process_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.add_process_link(&id, process_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Add a lot link to a deviation
#[tauri::command]
pub async fn add_deviation_lot_link(
    id: String,
    lot_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.add_lot_link(&id, lot_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}

/// Add a component link to a deviation
#[tauri::command]
pub async fn add_deviation_component_link(
    id: String,
    component_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Dev> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let dev = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = DeviationService::new(project, cache);

        service.add_component_link(&id, component_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(dev)
}
