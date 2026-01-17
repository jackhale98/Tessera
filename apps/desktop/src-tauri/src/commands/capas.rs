//! CAPA (Corrective and Preventive Action) entity commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::cache::CachedCapa;
use tdt_core::core::entity::Status;
use tdt_core::core::identity::EntityId;
use tdt_core::entities::capa::{
    ActionStatus, ActionType, Capa, CapaStatus, CapaType, EffectivenessResult, RcaMethod,
};
use tdt_core::services::{
    AddActionInput, CapaFilter, CapaService, CapaSortField, CapaStats, CreateCapa, SortDirection,
    UpdateCapa,
};

/// List parameters for CAPAs
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListCapasParams {
    pub status: Option<Vec<String>>,
    pub capa_type: Option<String>,
    pub capa_status: Option<String>,
    pub overdue_only: Option<bool>,
    pub open_only: Option<bool>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// CAPA summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct CapaSummary {
    pub id: String,
    pub title: String,
    pub capa_number: Option<String>,
    pub capa_type: String,
    pub capa_status: String,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Capa> for CapaSummary {
    fn from(capa: &Capa) -> Self {
        Self {
            id: capa.id.to_string(),
            title: capa.title.clone(),
            capa_number: capa.capa_number.clone(),
            capa_type: capa.capa_type.to_string(),
            capa_status: capa.capa_status.to_string(),
            status: format!("{:?}", capa.status).to_lowercase(),
            author: capa.author.clone(),
            created: capa.created.to_rfc3339(),
        }
    }
}

impl From<&CachedCapa> for CapaSummary {
    fn from(cached: &CachedCapa) -> Self {
        Self {
            id: cached.id.clone(),
            title: cached.title.clone(),
            capa_number: None, // Not in cache
            capa_type: cached.capa_type.clone().unwrap_or_default(),
            capa_status: cached.capa_status.clone().unwrap_or_default(),
            status: format!("{:?}", cached.status).to_lowercase(),
            author: cached.author.clone(),
            created: cached.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListCapasResult {
    pub items: Vec<CapaSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Input for creating a CAPA
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCapaInput {
    pub title: String,
    pub capa_number: Option<String>,
    pub capa_type: Option<String>,
    pub problem_statement: Option<String>,
    pub author: String,
}

/// Input for updating a CAPA
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCapaInput {
    pub title: Option<String>,
    pub capa_number: Option<String>,
    pub capa_type: Option<String>,
    pub problem_statement: Option<String>,
    pub status: Option<String>,
    pub target_date: Option<String>,
}

/// Input for setting root cause analysis
#[derive(Debug, Clone, Deserialize)]
pub struct SetRootCauseInput {
    pub method: String,
    pub root_cause: Option<String>,
    pub contributing_factors: Option<Vec<String>>,
}

/// Input for adding an action
#[derive(Debug, Clone, Deserialize)]
pub struct AddActionInputDto {
    pub description: String,
    pub action_type: Option<String>,
    pub owner: Option<String>,
    pub due_date: Option<String>,
}

/// Input for updating action status
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateActionStatusInput {
    pub action_number: u32,
    pub status: String,
}

/// Input for verifying effectiveness
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyEffectivenessInput {
    pub result: String,
    pub evidence: Option<String>,
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

fn parse_capa_type(s: &str) -> Option<CapaType> {
    match s.to_lowercase().as_str() {
        "corrective" | "corr" => Some(CapaType::Corrective),
        "preventive" | "prev" => Some(CapaType::Preventive),
        _ => None,
    }
}

fn parse_capa_status(s: &str) -> Option<CapaStatus> {
    match s.to_lowercase().as_str() {
        "initiation" | "init" => Some(CapaStatus::Initiation),
        "investigation" | "invest" => Some(CapaStatus::Investigation),
        "implementation" | "impl" => Some(CapaStatus::Implementation),
        "verification" | "verify" => Some(CapaStatus::Verification),
        "closed" => Some(CapaStatus::Closed),
        _ => None,
    }
}

fn parse_rca_method(s: &str) -> Option<RcaMethod> {
    match s.to_lowercase().as_str() {
        "five_why" | "5why" | "5_why" | "fivewhy" => Some(RcaMethod::FiveWhy),
        "fishbone" | "ishikawa" => Some(RcaMethod::Fishbone),
        "fault_tree" | "faulttree" | "fta" => Some(RcaMethod::FaultTree),
        "eight_d" | "8d" | "eightd" => Some(RcaMethod::EightD),
        _ => None,
    }
}

fn parse_action_type(s: &str) -> Option<ActionType> {
    match s.to_lowercase().as_str() {
        "corrective" | "corr" => Some(ActionType::Corrective),
        "preventive" | "prev" => Some(ActionType::Preventive),
        _ => None,
    }
}

fn parse_action_status(s: &str) -> Option<ActionStatus> {
    match s.to_lowercase().as_str() {
        "open" => Some(ActionStatus::Open),
        "in_progress" | "inprogress" => Some(ActionStatus::InProgress),
        "completed" | "complete" => Some(ActionStatus::Completed),
        "verified" | "verify" => Some(ActionStatus::Verified),
        _ => None,
    }
}

fn parse_effectiveness_result(s: &str) -> Option<EffectivenessResult> {
    match s.to_lowercase().as_str() {
        "effective" | "eff" => Some(EffectivenessResult::Effective),
        "partially_effective" | "partial" => Some(EffectivenessResult::PartiallyEffective),
        "ineffective" | "ineff" => Some(EffectivenessResult::Ineffective),
        _ => None,
    }
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// List CAPAs with optional filters
#[tauri::command]
pub async fn list_capas(
    params: Option<ListCapasParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListCapasResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = CapaService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = CapaFilter {
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
        capa_type: params.capa_type.and_then(|s| parse_capa_type(&s)),
        capa_status: params.capa_status.and_then(|s| parse_capa_status(&s)),
        overdue_only: params.overdue_only.unwrap_or(false),
        open_only: params.open_only.unwrap_or(false),
        recent_days: None,
        sort: params
            .sort_by
            .map(|s| match s.as_str() {
                "title" => CapaSortField::Title,
                "capa_type" => CapaSortField::CapaType,
                "capa_status" => CapaSortField::CapaStatus,
                "author" => CapaSortField::Author,
                "created" => CapaSortField::Created,
                _ => CapaSortField::Created,
            })
            .unwrap_or(CapaSortField::Created),
        sort_direction: if params.sort_desc.unwrap_or(true) {
            SortDirection::Descending
        } else {
            SortDirection::Ascending
        },
    };

    // Use cache for fast list views
    let cached_capas = service.list_cached(&filter)?;
    let total_count = cached_capas.len();

    Ok(ListCapasResult {
        items: cached_capas.iter().map(CapaSummary::from).collect(),
        total_count,
        has_more: false,
    })
}

/// Get a single CAPA by ID
#[tauri::command]
pub async fn get_capa(id: String, state: State<'_, AppState>) -> CommandResult<Option<Capa>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = CapaService::new(project, cache);
    let capa = service.get(&id)?;

    Ok(capa)
}

/// Create a new CAPA
#[tauri::command]
pub async fn create_capa(
    input: CreateCapaInput,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let create = CreateCapa {
            title: input.title,
            capa_number: input.capa_number,
            capa_type: input
                .capa_type
                .and_then(|s| parse_capa_type(&s))
                .unwrap_or(CapaType::Corrective),
            problem_statement: input.problem_statement,
            source_type: None,
            source_reference: None,
            target_date: None,
            tags: Vec::new(),
            author: input.author,
        };

        service.create(create)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Update an existing CAPA
#[tauri::command]
pub async fn update_capa(
    id: String,
    input: UpdateCapaInput,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let update = UpdateCapa {
            title: input.title,
            capa_number: input.capa_number.map(Some),
            capa_type: input.capa_type.and_then(|s| parse_capa_type(&s)),
            problem_statement: input.problem_statement.map(Some),
            status: input.status.and_then(|s| parse_status(&s)),
            target_date: input.target_date.map(|s| parse_date(&s)),
        };

        service.update(&id, update)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Delete a CAPA
#[tauri::command]
pub async fn delete_capa(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);
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

/// Advance CAPA workflow status to next stage
#[tauri::command]
pub async fn advance_capa_status(id: String, state: State<'_, AppState>) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);
        service.advance_status(&id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Set root cause analysis
#[tauri::command]
pub async fn set_capa_root_cause(
    id: String,
    input: SetRootCauseInput,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let method = parse_rca_method(&input.method).ok_or_else(|| {
            CommandError::InvalidInput(format!("Invalid RCA method: {}", input.method))
        })?;

        service.set_root_cause(
            &id,
            method,
            input.root_cause,
            input.contributing_factors.unwrap_or_default(),
        )?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Add an action item
#[tauri::command]
pub async fn add_capa_action(
    id: String,
    input: AddActionInputDto,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let action_input = AddActionInput {
            description: input.description,
            action_type: input
                .action_type
                .and_then(|s| parse_action_type(&s))
                .unwrap_or(ActionType::Corrective),
            owner: input.owner,
            due_date: input.due_date.and_then(|s| parse_date(&s)),
        };

        service.add_action(&id, action_input)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Update action status
#[tauri::command]
pub async fn update_capa_action_status(
    id: String,
    input: UpdateActionStatusInput,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let status = parse_action_status(&input.status).ok_or_else(|| {
            CommandError::InvalidInput(format!("Invalid action status: {}", input.status))
        })?;

        service.update_action_status(&id, input.action_number, status)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Verify CAPA effectiveness
#[tauri::command]
pub async fn verify_capa_effectiveness(
    id: String,
    input: VerifyEffectivenessInput,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let result = parse_effectiveness_result(&input.result).ok_or_else(|| {
            CommandError::InvalidInput(format!("Invalid effectiveness result: {}", input.result))
        })?;

        service.verify_effectiveness(&id, result, input.evidence)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Close a CAPA
#[tauri::command]
pub async fn close_capa(
    id: String,
    closed_by: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);
        service.close(&id, closed_by)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Link CAPA to an NCR
#[tauri::command]
pub async fn add_capa_ncr_link(
    id: String,
    ncr_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let entity_id: EntityId = ncr_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid NCR ID: {}", e)))?;

        service.add_ncr_link(&id, &entity_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Link CAPA to a Risk
#[tauri::command]
pub async fn add_capa_risk_link(
    id: String,
    risk_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Capa> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let capa = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = CapaService::new(project, cache);

        let entity_id: EntityId = risk_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid Risk ID: {}", e)))?;

        service.add_risk_link(&id, &entity_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(capa)
}

/// Get CAPA statistics
#[tauri::command]
pub async fn get_capa_stats(state: State<'_, AppState>) -> CommandResult<CapaStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = CapaService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}
