//! NCR (Non-Conformance Report) entity commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::cache::CachedNcr;
use tdt_core::core::entity::Status;
use tdt_core::core::identity::EntityId;
use tdt_core::entities::ncr::{
    DispositionDecision, Ncr, NcrCategory, NcrSeverity, NcrStatus, NcrType,
};
use tdt_core::services::{
    CreateNcr, NcrFilter, NcrService, NcrSortField, NcrStats, SortDirection, UpdateNcr,
};

/// List parameters for NCRs
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListNcrsParams {
    pub status: Option<Vec<String>>,
    pub ncr_type: Option<String>,
    pub severity: Option<String>,
    pub ncr_status: Option<String>,
    pub category: Option<String>,
    pub open_only: Option<bool>,
    pub recent_days: Option<u32>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// NCR summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct NcrSummary {
    pub id: String,
    pub title: String,
    pub ncr_number: Option<String>,
    pub ncr_type: String,
    pub severity: String,
    pub ncr_status: String,
    pub category: String,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Ncr> for NcrSummary {
    fn from(ncr: &Ncr) -> Self {
        Self {
            id: ncr.id.to_string(),
            title: ncr.title.clone(),
            ncr_number: ncr.ncr_number.clone(),
            ncr_type: ncr.ncr_type.to_string(),
            severity: ncr.severity.to_string(),
            ncr_status: ncr.ncr_status.to_string(),
            category: ncr.category.to_string(),
            status: format!("{:?}", ncr.status).to_lowercase(),
            author: ncr.author.clone(),
            created: ncr.created.to_rfc3339(),
        }
    }
}

impl From<&CachedNcr> for NcrSummary {
    fn from(cached: &CachedNcr) -> Self {
        Self {
            id: cached.id.clone(),
            title: cached.title.clone(),
            ncr_number: None, // Not in cache
            ncr_type: cached.ncr_type.clone().unwrap_or_default(),
            severity: cached.severity.clone().unwrap_or_default(),
            ncr_status: cached.ncr_status.clone().unwrap_or_default(),
            category: cached.category.clone().unwrap_or_default(),
            status: format!("{:?}", cached.status).to_lowercase(),
            author: cached.author.clone(),
            created: cached.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListNcrsResult {
    pub items: Vec<NcrSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Input for creating an NCR
#[derive(Debug, Clone, Deserialize)]
pub struct CreateNcrInput {
    pub title: String,
    pub ncr_number: Option<String>,
    pub description: Option<String>,
    pub ncr_type: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub author: String,
    pub lot_ids: Option<Vec<String>>,
}

/// Input for updating an NCR
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNcrInput {
    pub title: Option<String>,
    pub ncr_number: Option<String>,
    pub description: Option<String>,
    pub ncr_type: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub ncr_status: Option<String>,
}

/// Input for closing an NCR
#[derive(Debug, Clone, Deserialize)]
pub struct CloseNcrInput {
    pub decision: String,
    pub decision_maker: String,
    pub justification: Option<String>,
    pub mrb_required: Option<bool>,
}

/// Input for detection info
#[derive(Debug, Clone, Deserialize)]
pub struct SetDetectionInput {
    pub found_at: String,
    pub found_by: Option<String>,
    pub found_date: Option<String>,
    pub operation: Option<String>,
}

/// Input for affected items
#[derive(Debug, Clone, Deserialize)]
pub struct SetAffectedItemsInput {
    pub part_number: Option<String>,
    pub lot_number: Option<String>,
    pub serial_numbers: Option<Vec<String>>,
    pub quantity: Option<u32>,
}

/// Input for defect info
#[derive(Debug, Clone, Deserialize)]
pub struct SetDefectInput {
    pub characteristic: Option<String>,
    pub specification: Option<String>,
    pub actual: Option<String>,
    pub deviation: Option<f64>,
}

/// Input for cost impact
#[derive(Debug, Clone, Deserialize)]
pub struct SetCostInput {
    pub rework_cost: Option<f64>,
    pub scrap_cost: Option<f64>,
    pub currency: Option<String>,
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

fn parse_ncr_type(s: &str) -> Option<NcrType> {
    match s.to_lowercase().as_str() {
        "internal" | "int" => Some(NcrType::Internal),
        "supplier" | "sup" => Some(NcrType::Supplier),
        "customer" | "cust" => Some(NcrType::Customer),
        _ => None,
    }
}

fn parse_severity(s: &str) -> Option<NcrSeverity> {
    match s.to_lowercase().as_str() {
        "minor" => Some(NcrSeverity::Minor),
        "major" => Some(NcrSeverity::Major),
        "critical" | "crit" => Some(NcrSeverity::Critical),
        _ => None,
    }
}

fn parse_ncr_status(s: &str) -> Option<NcrStatus> {
    match s.to_lowercase().as_str() {
        "open" => Some(NcrStatus::Open),
        "containment" => Some(NcrStatus::Containment),
        "investigation" => Some(NcrStatus::Investigation),
        "disposition" => Some(NcrStatus::Disposition),
        "closed" => Some(NcrStatus::Closed),
        _ => None,
    }
}

fn parse_category(s: &str) -> Option<NcrCategory> {
    match s.to_lowercase().as_str() {
        "dimensional" | "dim" => Some(NcrCategory::Dimensional),
        "cosmetic" | "cos" => Some(NcrCategory::Cosmetic),
        "material" | "mat" => Some(NcrCategory::Material),
        "functional" | "func" => Some(NcrCategory::Functional),
        "documentation" | "doc" => Some(NcrCategory::Documentation),
        "process" | "proc" => Some(NcrCategory::Process),
        "packaging" | "pkg" => Some(NcrCategory::Packaging),
        _ => None,
    }
}

fn parse_disposition_decision(s: &str) -> Option<DispositionDecision> {
    match s.to_lowercase().as_str() {
        "use_as_is" | "useaseis" | "use-as-is" => Some(DispositionDecision::UseAsIs),
        "rework" => Some(DispositionDecision::Rework),
        "scrap" => Some(DispositionDecision::Scrap),
        "return_to_supplier" | "returntosupplier" | "return-to-supplier" | "return" => {
            Some(DispositionDecision::ReturnToSupplier)
        }
        _ => None,
    }
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// List NCRs with optional filters
#[tauri::command]
pub async fn list_ncrs(
    params: Option<ListNcrsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListNcrsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = NcrService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = NcrFilter {
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
        ncr_type: params.ncr_type.and_then(|s| parse_ncr_type(&s)),
        severity: params.severity.and_then(|s| parse_severity(&s)),
        ncr_status: params.ncr_status.and_then(|s| parse_ncr_status(&s)),
        category: params.category.and_then(|s| parse_category(&s)),
        open_only: params.open_only.unwrap_or(false),
        recent_days: params.recent_days,
        sort: params
            .sort_by
            .map(|s| match s.as_str() {
                "title" => NcrSortField::Title,
                "ncr_type" => NcrSortField::NcrType,
                "severity" => NcrSortField::Severity,
                "ncr_status" => NcrSortField::NcrStatus,
                "category" => NcrSortField::Category,
                "author" => NcrSortField::Author,
                "created" => NcrSortField::Created,
                _ => NcrSortField::Created,
            })
            .unwrap_or(NcrSortField::Created),
        sort_direction: if params.sort_desc.unwrap_or(true) {
            SortDirection::Descending
        } else {
            SortDirection::Ascending
        },
    };

    // Use cache for fast list views
    let cached_ncrs = service.list_cached(&filter)?;
    let total_count = cached_ncrs.len();

    Ok(ListNcrsResult {
        items: cached_ncrs.iter().map(NcrSummary::from).collect(),
        total_count,
        has_more: false,
    })
}

/// Get a single NCR by ID
#[tauri::command]
pub async fn get_ncr(id: String, state: State<'_, AppState>) -> CommandResult<Option<Ncr>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = NcrService::new(project, cache);
    let ncr = service.get(&id)?;

    Ok(ncr)
}

/// Create a new NCR
#[tauri::command]
pub async fn create_ncr(input: CreateNcrInput, state: State<'_, AppState>) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        let create = CreateNcr {
            title: input.title,
            ncr_number: input.ncr_number,
            description: input.description,
            ncr_type: input
                .ncr_type
                .and_then(|s| parse_ncr_type(&s))
                .unwrap_or(NcrType::Internal),
            severity: input
                .severity
                .and_then(|s| parse_severity(&s))
                .unwrap_or(NcrSeverity::Minor),
            category: input
                .category
                .and_then(|s| parse_category(&s))
                .unwrap_or(NcrCategory::Dimensional),
            report_date: None,
            tags: Vec::new(),
            status: None,
            author: input.author,
            lot_ids: input.lot_ids.unwrap_or_default(),
        };

        service.create(create)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Update an existing NCR
#[tauri::command]
pub async fn update_ncr(
    id: String,
    input: UpdateNcrInput,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        let update = UpdateNcr {
            title: input.title,
            ncr_number: input.ncr_number.map(Some),
            description: input.description.map(Some),
            ncr_type: input.ncr_type.and_then(|s| parse_ncr_type(&s)),
            severity: input.severity.and_then(|s| parse_severity(&s)),
            category: input.category.and_then(|s| parse_category(&s)),
            status: input.status.and_then(|s| parse_status(&s)),
            ncr_status: input.ncr_status.and_then(|s| parse_ncr_status(&s)),
            tags: None,
        };

        service.update(&id, update)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Delete an NCR
#[tauri::command]
pub async fn delete_ncr(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);
        service.delete(&id, false)?;
    }

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Close an NCR with disposition
#[tauri::command]
pub async fn close_ncr(
    id: String,
    input: CloseNcrInput,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        let decision =
            parse_disposition_decision(&input.decision).unwrap_or(DispositionDecision::UseAsIs);

        service.close(&id, decision, input.justification, input.decision_maker)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Advance NCR status to next stage
#[tauri::command]
pub async fn advance_ncr_status(id: String, state: State<'_, AppState>) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);
        service.advance_status(&id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Add a containment action
#[tauri::command]
pub async fn add_ncr_containment(
    id: String,
    action: String,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);
        service.add_containment(&id, action)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Complete a containment action
#[tauri::command]
pub async fn complete_ncr_containment(
    id: String,
    action_index: usize,
    completed_by: String,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);
        service.complete_containment(&id, action_index, completed_by)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Set detection information
#[tauri::command]
pub async fn set_ncr_detection(
    id: String,
    input: SetDetectionInput,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        // Parse detection stage (required)
        let found_at = match input.found_at.to_lowercase().as_str() {
            "incoming" => tdt_core::entities::ncr::DetectionStage::Incoming,
            "in_process" | "inprocess" => tdt_core::entities::ncr::DetectionStage::InProcess,
            "final" => tdt_core::entities::ncr::DetectionStage::Final,
            "customer" => tdt_core::entities::ncr::DetectionStage::Customer,
            "field" => tdt_core::entities::ncr::DetectionStage::Field,
            _ => {
                return Err(CommandError::InvalidInput(format!(
                    "Invalid detection stage: {}",
                    input.found_at
                )))
            }
        };

        let found_date = input.found_date.and_then(|s| parse_date(&s));

        service.set_detection(&id, found_at, input.found_by, found_date, input.operation)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Set affected items
#[tauri::command]
pub async fn set_ncr_affected_items(
    id: String,
    input: SetAffectedItemsInput,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);
        service.set_affected_items(
            &id,
            input.part_number,
            input.lot_number,
            input.serial_numbers.unwrap_or_default(),
            input.quantity,
        )?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Set defect information
#[tauri::command]
pub async fn set_ncr_defect(
    id: String,
    input: SetDefectInput,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);
        service.set_defect(
            &id,
            input.characteristic,
            input.specification,
            input.actual,
            input.deviation,
        )?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Set cost impact
#[tauri::command]
pub async fn set_ncr_cost(
    id: String,
    input: SetCostInput,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);
        service.set_cost(&id, input.rework_cost, input.scrap_cost, input.currency)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Link NCR to a component
#[tauri::command]
pub async fn set_ncr_component_link(
    id: String,
    component_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        let entity_id: EntityId = component_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid component ID: {}", e)))?;

        service.set_component_link(&id, entity_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Link NCR to a CAPA
#[tauri::command]
pub async fn set_ncr_capa_link(
    id: String,
    capa_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        let entity_id: EntityId = capa_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid CAPA ID: {}", e)))?;

        service.set_capa_link(&id, entity_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Add a lot link to an NCR
#[tauri::command]
pub async fn add_ncr_lot_link(
    id: String,
    lot_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        let entity_id: EntityId = lot_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid Lot ID: {}", e)))?;

        service.add_lot_link(&id, entity_id)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Remove a lot link from an NCR
#[tauri::command]
pub async fn remove_ncr_lot_link(
    id: String,
    lot_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Ncr> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let ncr = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = NcrService::new(project, cache);

        let entity_id: EntityId = lot_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid Lot ID: {}", e)))?;

        service.remove_lot_link(&id, &entity_id)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(ncr)
}

/// Get NCR statistics
#[tauri::command]
pub async fn get_ncr_stats(state: State<'_, AppState>) -> CommandResult<NcrStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = NcrService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}
