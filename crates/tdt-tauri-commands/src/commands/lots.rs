//! Lot (Manufacturing Lot) entity commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::cache::CachedEntity;
use tdt_core::core::entity::Status;
use tdt_core::entities::lot::{
    ExecutionStatus, ExecutionStep, Lot, LotStatus, MaterialUsed, WiStepExecution,
};
use tdt_core::core::{Config, Git, LotWorkflowConfig};
use tdt_core::services::{
    ApproveWiStepInput, CreateLot, ExecuteWiStepInput, LotFilter, LotService, LotSortField,
    LotStats, SortDirection, UpdateLot, UpdateStepInput, WiStepExecutionResult,
};

/// Try to auto-commit a lot change. Returns the commit SHA on success, or None if
/// auto-commit is disabled or the commit fails (non-fatal).
fn try_auto_commit(
    root: &std::path::Path,
    lot: &Lot,
    message: &str,
) -> Option<String> {
    let config = Config::load();
    let wf_config = LotWorkflowConfig::from_config(&config);

    if !wf_config.auto_commit {
        return None;
    }

    let git = Git::new(root);
    if !git.is_repo() {
        return None;
    }

    // Stage the lot file
    let lot_file = root
        .join("manufacturing/lots")
        .join(format!("{}.tdt.yaml", lot.id));

    if let Err(e) = git.stage_file(&lot_file) {
        log::warn!("Auto-commit: failed to stage lot file: {}", e);
        return None;
    }

    // Commit
    let result = if wf_config.sign_commits {
        git.commit_signed(message)
    } else {
        git.commit(message)
    };

    match result {
        Ok(sha) => Some(sha),
        Err(e) => {
            log::warn!("Auto-commit: failed to commit: {}", e);
            None
        }
    }
}

/// List parameters for Lots
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListLotsParams {
    pub status: Option<Vec<String>>,
    pub lot_status: Option<String>,
    pub product: Option<String>,
    pub active_only: Option<bool>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// Lot summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct LotSummary {
    pub id: String,
    pub title: String,
    pub lot_number: Option<String>,
    pub quantity: Option<u32>,
    pub lot_status: String,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Lot> for LotSummary {
    fn from(lot: &Lot) -> Self {
        Self {
            id: lot.id.to_string(),
            title: lot.title.clone(),
            lot_number: lot.lot_number.clone(),
            quantity: lot.quantity,
            lot_status: lot.lot_status.to_string(),
            status: format!("{:?}", lot.status).to_lowercase(),
            author: lot.author.clone(),
            created: lot.created.to_rfc3339(),
        }
    }
}

impl From<&CachedEntity> for LotSummary {
    fn from(cached: &CachedEntity) -> Self {
        Self {
            id: cached.id.clone(),
            title: cached.title.clone(),
            lot_number: None, // Not in generic cache
            quantity: None,   // Not in generic cache
            lot_status: String::new(),
            status: format!("{:?}", cached.status).to_lowercase(),
            author: cached.author.clone(),
            created: cached.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListLotsResult {
    pub items: Vec<LotSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Input for creating a Lot
#[derive(Debug, Clone, Deserialize)]
pub struct CreateLotInput {
    pub title: String,
    pub lot_number: Option<String>,
    pub quantity: Option<u32>,
    pub product_id: Option<String>,
    pub author: String,
    pub from_routing: Option<bool>,
}

/// Input for updating a Lot
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLotInput {
    pub title: Option<String>,
    pub lot_number: Option<String>,
    pub quantity: Option<u32>,
    pub status: Option<String>,
}

/// Input for updating an execution step
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStepInputDto {
    pub status: Option<String>,
    pub operator: Option<String>,
    pub notes: Option<String>,
    pub work_instructions_used: Option<Vec<String>>,
    pub signed: Option<bool>,
}

/// Input for adding a material
#[derive(Debug, Clone, Deserialize)]
pub struct AddMaterialInput {
    pub component_id: Option<String>,
    pub supplier_lot: Option<String>,
    pub quantity: Option<u32>,
}

/// Input for adding a step
#[derive(Debug, Clone, Deserialize)]
pub struct AddStepInput {
    pub process_id: Option<String>,
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

fn parse_lot_status(s: &str) -> Option<LotStatus> {
    match s.to_lowercase().as_str() {
        "in_progress" | "inprogress" => Some(LotStatus::InProgress),
        "on_hold" | "onhold" => Some(LotStatus::OnHold),
        "completed" | "complete" => Some(LotStatus::Completed),
        "scrapped" | "scrap" => Some(LotStatus::Scrapped),
        _ => None,
    }
}

fn parse_execution_status(s: &str) -> Option<ExecutionStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(ExecutionStatus::Pending),
        "in_progress" | "inprogress" => Some(ExecutionStatus::InProgress),
        "completed" | "complete" => Some(ExecutionStatus::Completed),
        "skipped" | "skip" => Some(ExecutionStatus::Skipped),
        _ => None,
    }
}

/// List Lots with optional filters
#[tauri::command]
pub async fn list_lots(
    params: Option<ListLotsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListLotsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = LotService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = LotFilter {
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
        lot_status: params.lot_status.and_then(|s| parse_lot_status(&s)),
        product: params.product,
        active_only: params.active_only.unwrap_or(false),
        recent_days: None,
        sort: params
            .sort_by
            .map(|s| match s.as_str() {
                "title" => LotSortField::Title,
                "lot_number" => LotSortField::LotNumber,
                "quantity" => LotSortField::Quantity,
                "lot_status" => LotSortField::LotStatus,
                "author" => LotSortField::Author,
                "created" => LotSortField::Created,
                _ => LotSortField::Created,
            })
            .unwrap_or(LotSortField::Created),
        sort_direction: if params.sort_desc.unwrap_or(true) {
            SortDirection::Descending
        } else {
            SortDirection::Ascending
        },
    };

    // Use full list since we need lot-specific fields for summary
    let lots = service.list(&filter)?;
    let total_count = lots.len();

    Ok(ListLotsResult {
        items: lots.iter().map(LotSummary::from).collect(),
        total_count,
        has_more: false,
    })
}

/// Get a single Lot by ID
#[tauri::command]
pub async fn get_lot(id: String, state: State<'_, AppState>) -> CommandResult<Option<Lot>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = LotService::new(project, cache);
    let lot = service.get(&id)?;

    Ok(lot)
}

/// Create a new Lot
#[tauri::command]
pub async fn create_lot(input: CreateLotInput, state: State<'_, AppState>) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);

        let create = CreateLot {
            title: input.title,
            lot_number: input.lot_number,
            quantity: input.quantity,
            product: input.product_id,
            notes: None,
            start_date: None,
            status: None,
            tags: Vec::new(),
            author: input.author,
            from_routing: input.from_routing.unwrap_or(false),
        };

        service.create(create)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Update an existing Lot
#[tauri::command]
pub async fn update_lot(
    id: String,
    input: UpdateLotInput,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);

        let update = UpdateLot {
            title: input.title,
            lot_number: input.lot_number.map(Some),
            quantity: input.quantity.map(Some),
            status: input.status.and_then(|s| parse_status(&s)),
            ..Default::default()
        };

        service.update(&id, update)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Delete a Lot
#[tauri::command]
pub async fn delete_lot(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
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

/// Set lot product (ASM or CMP)
#[tauri::command]
pub async fn set_lot_product(
    id: String,
    product_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.set_product(&id, &product_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Add material to lot
#[tauri::command]
pub async fn add_lot_material(
    id: String,
    input: AddMaterialInput,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);

        let material = MaterialUsed {
            component: input.component_id,
            supplier_lot: input.supplier_lot,
            quantity: input.quantity,
        };

        service.add_material(&id, material)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Remove material from lot
#[tauri::command]
pub async fn remove_lot_material(
    id: String,
    component_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.remove_material(&id, &component_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Add execution step to lot
#[tauri::command]
pub async fn add_lot_step(
    id: String,
    input: AddStepInput,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);

        let step = ExecutionStep {
            process: input.process_id,
            ..Default::default()
        };

        service.add_step(&id, step)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Update execution step
#[tauri::command]
pub async fn update_lot_step(
    id: String,
    step_index: usize,
    input: UpdateStepInputDto,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);

        let update = UpdateStepInput {
            status: input.status.and_then(|s| parse_execution_status(&s)),
            operator: input.operator,
            notes: input.notes,
            work_instructions_used: input.work_instructions_used,
            signed: input.signed.unwrap_or(false),
        };

        service.update_step(&id, step_index, update)?
    };

    // Auto-commit if step was completed or skipped
    if let Some(step) = lot.execution.get(step_index) {
        if step.status == ExecutionStatus::Completed || step.status == ExecutionStatus::Skipped {
            let operator = step.operator.as_deref().unwrap_or("unknown");
            let lot_num = lot.lot_number.as_deref().unwrap_or(&id);
            let msg = format!(
                "lot({}): Step {} {} by {}",
                lot_num,
                step_index + 1,
                if step.status == ExecutionStatus::Completed { "completed" } else { "skipped" },
                operator
            );
            let _ = try_auto_commit(project.root(), &lot, &msg);
        }
    }

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Remove execution step
#[tauri::command]
pub async fn remove_lot_step(
    id: String,
    step_index: usize,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.remove_step(&id, step_index)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Put lot on hold
#[tauri::command]
pub async fn put_lot_on_hold(id: String, state: State<'_, AppState>) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.put_on_hold(&id)?
    };

    // Auto-commit
    let lot_num = lot.lot_number.as_deref().unwrap_or(&id);
    let msg = format!("lot({}): Put on hold", lot_num);
    let _ = try_auto_commit(project.root(), &lot, &msg);

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Resume lot from hold
#[tauri::command]
pub async fn resume_lot(id: String, state: State<'_, AppState>) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.resume(&id)?
    };

    // Auto-commit
    let lot_num = lot.lot_number.as_deref().unwrap_or(&id);
    let msg = format!("lot({}): Resumed", lot_num);
    let _ = try_auto_commit(project.root(), &lot, &msg);

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Complete lot
#[tauri::command]
pub async fn complete_lot(id: String, state: State<'_, AppState>) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.complete(&id)?
    };

    // Auto-commit
    let lot_num = lot.lot_number.as_deref().unwrap_or(&id);
    let msg = format!("lot({}): Completed", lot_num);
    let _ = try_auto_commit(project.root(), &lot, &msg);

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Force complete lot (skip incomplete step checks)
#[tauri::command]
pub async fn force_complete_lot(id: String, state: State<'_, AppState>) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.complete_force(&id)?
    };

    // Auto-commit
    let lot_num = lot.lot_number.as_deref().unwrap_or(&id);
    let msg = format!("lot({}): Force completed", lot_num);
    let _ = try_auto_commit(project.root(), &lot, &msg);

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Scrap lot
#[tauri::command]
pub async fn scrap_lot(id: String, state: State<'_, AppState>) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.scrap(&id)?
    };

    // Auto-commit
    let lot_num = lot.lot_number.as_deref().unwrap_or(&id);
    let msg = format!("lot({}): Scrapped", lot_num);
    let _ = try_auto_commit(project.root(), &lot, &msg);

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Add NCR link to lot
#[tauri::command]
pub async fn add_lot_ncr(
    id: String,
    ncr_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.add_ncr(&id, &ncr_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Remove NCR link from lot
#[tauri::command]
pub async fn remove_lot_ncr(
    id: String,
    ncr_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.remove_ncr(&id, &ncr_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Add test result link to lot
#[tauri::command]
pub async fn add_lot_result(
    id: String,
    result_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.add_result(&id, &result_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Set git branch for lot
#[tauri::command]
pub async fn set_lot_git_branch(
    id: String,
    branch: String,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.set_git_branch(&id, &branch)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Mark lot git branch as merged
#[tauri::command]
pub async fn mark_lot_branch_merged(id: String, state: State<'_, AppState>) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);
        service.mark_branch_merged(&id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

/// Get next step index for lot
#[tauri::command]
pub async fn get_lot_next_step(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<usize>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = LotService::new(project, cache);
    let next_step = service.get_next_step_index(&id)?;

    Ok(next_step)
}

/// Get Lot statistics
#[tauri::command]
pub async fn get_lot_stats(state: State<'_, AppState>) -> CommandResult<LotStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = LotService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}

// ============================================================================
// WI Step Execution Commands
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ExecuteWiStepDto {
    pub work_instruction_id: String,
    pub step_number: u32,
    pub process_index: Option<usize>,
    pub operator: String,
    pub operator_email: Option<String>,
    pub data: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub equipment: Option<std::collections::HashMap<String, String>>,
    pub notes: Option<String>,
    pub sign: Option<bool>,
    pub require_approval: Option<bool>,
    pub complete: Option<bool>,
    pub deviation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WiStepExecutionResultDto {
    pub lot: Lot,
    pub process_index: usize,
    pub step_number: u32,
    pub was_completed: bool,
    pub deviation_used: Option<String>,
}

impl From<WiStepExecutionResult> for WiStepExecutionResultDto {
    fn from(r: WiStepExecutionResult) -> Self {
        Self {
            lot: r.lot,
            process_index: r.process_index,
            step_number: r.step_number,
            was_completed: r.was_completed,
            deviation_used: r.deviation_used,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApproveWiStepDto {
    pub approver: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub comment: Option<String>,
    pub sign: Option<bool>,
    pub reject: Option<bool>,
}

#[tauri::command]
pub async fn execute_wi_step(
    id: String,
    input: ExecuteWiStepDto,
    state: State<'_, AppState>,
) -> CommandResult<WiStepExecutionResultDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let result = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = LotService::new(project, cache);

        let exec_input = ExecuteWiStepInput {
            work_instruction_id: input.work_instruction_id,
            step_number: input.step_number,
            process_index: input.process_index,
            operator: input.operator,
            operator_email: input.operator_email,
            data: input.data.unwrap_or_default(),
            equipment: input.equipment.unwrap_or_default(),
            notes: input.notes,
            sign: input.sign.unwrap_or(false),
            require_approval: input.require_approval.unwrap_or(false),
            complete: input.complete.unwrap_or(false),
            deviation_id: input.deviation_id,
        };

        service.execute_wi_step(&id, exec_input)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(WiStepExecutionResultDto::from(result))
}

#[tauri::command]
pub async fn get_wi_step_status(
    id: String,
    process_index: usize,
    wi_id: String,
    step_number: u32,
    state: State<'_, AppState>,
) -> CommandResult<Option<WiStepExecution>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = LotService::new(project, cache);
    Ok(service.get_wi_step_status(&id, process_index, &wi_id, step_number)?)
}

#[tauri::command]
pub async fn approve_wi_step(
    id: String,
    process_index: usize,
    wi_id: String,
    step_number: u32,
    input: ApproveWiStepDto,
    state: State<'_, AppState>,
) -> CommandResult<Lot> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let lot = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let guard = tdt_core::services::WorkflowGuard::load(project);
        let service = LotService::new(project, cache).with_workflow(guard);

        let approve_input = ApproveWiStepInput {
            approver: input.approver,
            email: input.email,
            role: input.role,
            comment: input.comment,
            sign: input.sign.unwrap_or(false),
            reject: input.reject.unwrap_or(false),
        };

        service.approve_wi_step(&id, process_index, &wi_id, step_number, approve_input)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(lot)
}

#[tauri::command]
pub async fn validate_lot_step_order(
    id: String,
    process_index: usize,
    wi_id: String,
    step_number: u32,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = LotService::new(project, cache);
    let lot = service
        .get(&id)?
        .ok_or_else(|| CommandError::NotFound(format!("Lot: {}", id)))?;

    service.validate_step_order(&lot, process_index, &wi_id, step_number)?;
    Ok(())
}
