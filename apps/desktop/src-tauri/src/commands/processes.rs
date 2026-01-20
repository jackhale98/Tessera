//! Process entity commands
//!
//! Provides commands for managing manufacturing processes.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::process::{Process, ProcessType, SkillLevel};
use tdt_core::services::common::SortDirection;
use tdt_core::services::process::{
    CreateProcess, ProcessFilter, ProcessService, ProcessSortField, ProcessStats, UpdateProcess,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Process summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct ProcessSummary {
    pub id: String,
    pub title: String,
    pub process_type: String,
    pub operation_number: Option<String>,
    pub cycle_time_minutes: Option<f64>,
    pub setup_time_minutes: Option<f64>,
    pub operator_skill: String,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Process> for ProcessSummary {
    fn from(p: &Process) -> Self {
        Self {
            id: p.id.to_string(),
            title: p.title.clone(),
            process_type: p.process_type.to_string(),
            operation_number: p.operation_number.clone(),
            cycle_time_minutes: p.cycle_time_minutes,
            setup_time_minutes: p.setup_time_minutes,
            operator_skill: p.operator_skill.to_string(),
            status: format!("{:?}", p.status).to_lowercase(),
            author: p.author.clone(),
            created: p.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListProcessesResult {
    pub items: Vec<ProcessSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListProcessesParams {
    pub status: Option<Vec<String>>,
    pub process_type: Option<String>,
    pub operation_number: Option<String>,
    pub has_equipment: Option<bool>,
    pub has_capability: Option<bool>,
    pub requires_signature: Option<bool>,
    pub skill_level: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProcessInput {
    pub title: String,
    pub author: String,
    pub process_type: Option<String>,
    pub operation_number: Option<String>,
    pub description: Option<String>,
    pub cycle_time_minutes: Option<f64>,
    pub setup_time_minutes: Option<f64>,
    pub operator_skill: Option<String>,
    pub require_signature: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateProcessInput {
    pub title: Option<String>,
    pub process_type: Option<String>,
    pub operation_number: Option<String>,
    pub description: Option<String>,
    pub cycle_time_minutes: Option<f64>,
    pub setup_time_minutes: Option<f64>,
    pub operator_skill: Option<String>,
    pub require_signature: Option<bool>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
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

fn parse_process_type(s: &str) -> Option<ProcessType> {
    match s.to_lowercase().as_str() {
        "machining" => Some(ProcessType::Machining),
        "assembly" => Some(ProcessType::Assembly),
        "inspection" => Some(ProcessType::Inspection),
        "test" => Some(ProcessType::Test),
        "finishing" => Some(ProcessType::Finishing),
        "packaging" => Some(ProcessType::Packaging),
        "handling" => Some(ProcessType::Handling),
        "heat_treat" | "heattreat" => Some(ProcessType::HeatTreat),
        "welding" => Some(ProcessType::Welding),
        "coating" => Some(ProcessType::Coating),
        _ => None,
    }
}

fn parse_skill_level(s: &str) -> Option<SkillLevel> {
    match s.to_lowercase().as_str() {
        "entry" => Some(SkillLevel::Entry),
        "intermediate" => Some(SkillLevel::Intermediate),
        "advanced" => Some(SkillLevel::Advanced),
        "expert" => Some(SkillLevel::Expert),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> ProcessSortField {
    match s.to_lowercase().as_str() {
        "id" => ProcessSortField::Id,
        "title" => ProcessSortField::Title,
        "process_type" | "type" => ProcessSortField::ProcessType,
        "operation_number" | "op" => ProcessSortField::OperationNumber,
        "cycle_time" => ProcessSortField::CycleTime,
        "status" => ProcessSortField::Status,
        "author" => ProcessSortField::Author,
        "created" => ProcessSortField::Created,
        _ => ProcessSortField::Title,
    }
}

fn build_process_filter(params: &ListProcessesParams) -> ProcessFilter {
    use tdt_core::services::common::CommonFilter;

    let common = CommonFilter {
        status: params.status.as_ref().and_then(|v| {
            let statuses: Vec<Status> = v.iter().filter_map(|s| parse_status(s)).collect();
            if statuses.is_empty() {
                None
            } else {
                Some(statuses)
            }
        }),
        search: params.search.clone(),
        limit: params.limit,
        ..Default::default()
    };

    ProcessFilter {
        common,
        process_type: params
            .process_type
            .as_ref()
            .and_then(|t| parse_process_type(t)),
        operation_number: params.operation_number.clone(),
        has_equipment: params.has_equipment.unwrap_or(false),
        has_capability: params.has_capability.unwrap_or(false),
        requires_signature: params.requires_signature.unwrap_or(false),
        skill_level: params
            .skill_level
            .as_ref()
            .and_then(|s| parse_skill_level(s)),
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_processes(
    params: Option<ListProcessesParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListProcessesResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ProcessService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_process_filter(&params);

    let sort = params
        .sort_by
        .as_ref()
        .map(|s| parse_sort_field(s))
        .unwrap_or_default();
    let sort_direction = if params.sort_desc.unwrap_or(false) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    let processes = service.list(&filter, sort, sort_direction)?;

    Ok(ListProcessesResult {
        total_count: processes.items.len(),
        items: processes.items.iter().map(ProcessSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_process(id: String, state: State<'_, AppState>) -> CommandResult<Option<Process>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ProcessService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn create_process(
    input: CreateProcessInput,
    state: State<'_, AppState>,
) -> CommandResult<Process> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let process = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ProcessService::new(project, cache);

        let create = CreateProcess {
            title: input.title,
            author: input.author,
            process_type: input
                .process_type
                .and_then(|t| parse_process_type(&t))
                .unwrap_or_default(),
            operation_number: input.operation_number,
            description: input.description,
            cycle_time_minutes: input.cycle_time_minutes,
            setup_time_minutes: input.setup_time_minutes,
            operator_skill: input
                .operator_skill
                .and_then(|s| parse_skill_level(&s))
                .unwrap_or_default(),
            require_signature: input.require_signature.unwrap_or(false),
            tags: input.tags.unwrap_or_default(),
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(process)
}

#[tauri::command]
pub async fn update_process(
    id: String,
    input: UpdateProcessInput,
    state: State<'_, AppState>,
) -> CommandResult<Process> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let process = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ProcessService::new(project, cache);

        let update = UpdateProcess {
            title: input.title,
            process_type: input.process_type.and_then(|t| parse_process_type(&t)),
            operation_number: input.operation_number,
            description: input.description,
            cycle_time_minutes: input.cycle_time_minutes,
            setup_time_minutes: input.setup_time_minutes,
            operator_skill: input.operator_skill.and_then(|s| parse_skill_level(&s)),
            require_signature: input.require_signature,
            status: input.status.and_then(|s| parse_status(&s)),
            tags: input.tags,
            equipment: None,
            parameters: None,
            capability: None,
            safety: None,
            step_approval: None,
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(process)
}

#[tauri::command]
pub async fn delete_process(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ProcessService::new(project, cache);
        service.delete(&id, force.unwrap_or(false))?;
    }

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

#[tauri::command]
pub async fn get_process_stats(state: State<'_, AppState>) -> CommandResult<ProcessStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ProcessService::new(project, cache);
    Ok(service.stats()?)
}
