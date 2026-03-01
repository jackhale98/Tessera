//! Work Instruction entity commands
//!
//! Provides commands for managing work instructions and procedures.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::work_instruction::{
    Hazard, Material, PpeItem, ProcedureStep, QualityCheck, Tool, WorkInstruction, WorkSafety,
};
use tdt_core::services::common::SortDirection;
use tdt_core::services::work_instruction::{
    CreateWorkInstruction, UpdateWorkInstruction, WorkInstructionFilter, WorkInstructionService,
    WorkInstructionSortField, WorkInstructionStats,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Work instruction summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct WorkInstructionSummary {
    pub id: String,
    pub title: String,
    pub document_number: Option<String>,
    pub revision: Option<String>,
    pub process: Option<String>,
    pub step_count: usize,
    pub estimated_duration_minutes: Option<f64>,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&WorkInstruction> for WorkInstructionSummary {
    fn from(w: &WorkInstruction) -> Self {
        Self {
            id: w.id.to_string(),
            title: w.title.clone(),
            document_number: w.document_number.clone(),
            revision: w.revision.clone(),
            process: w.links.process.as_ref().map(|p| p.to_string()),
            step_count: w.procedure.len(),
            estimated_duration_minutes: w.estimated_duration_minutes,
            status: format!("{:?}", w.status).to_lowercase(),
            author: w.author.clone(),
            created: w.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListWorkInstructionsResult {
    pub items: Vec<WorkInstructionSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListWorkInstructionsParams {
    pub status: Option<Vec<String>>,
    pub process: Option<String>,
    pub has_safety: Option<bool>,
    pub has_quality_checks: Option<bool>,
    pub document_number: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkInstructionInput {
    pub title: String,
    pub author: String,
    pub document_number: Option<String>,
    pub revision: Option<String>,
    pub description: Option<String>,
    pub process: Option<String>,
    pub estimated_duration_minutes: Option<f64>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateWorkInstructionInput {
    pub title: Option<String>,
    pub document_number: Option<String>,
    pub revision: Option<String>,
    pub description: Option<String>,
    pub estimated_duration_minutes: Option<f64>,
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

fn parse_sort_field(s: &str) -> WorkInstructionSortField {
    match s.to_lowercase().as_str() {
        "id" => WorkInstructionSortField::Id,
        "title" => WorkInstructionSortField::Title,
        "document_number" | "doc" => WorkInstructionSortField::DocumentNumber,
        "status" => WorkInstructionSortField::Status,
        "author" => WorkInstructionSortField::Author,
        "created" => WorkInstructionSortField::Created,
        "step_count" | "steps" => WorkInstructionSortField::StepCount,
        _ => WorkInstructionSortField::Title,
    }
}

fn build_work_instruction_filter(params: &ListWorkInstructionsParams) -> WorkInstructionFilter {
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

    WorkInstructionFilter {
        common,
        process: params.process.clone(),
        has_safety: params.has_safety.unwrap_or(false),
        has_quality_checks: params.has_quality_checks.unwrap_or(false),
        document_number: params.document_number.clone(),
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_work_instructions(
    params: Option<ListWorkInstructionsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListWorkInstructionsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = WorkInstructionService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_work_instruction_filter(&params);

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

    let work_instructions = service.list(&filter, sort, sort_direction)?;

    Ok(ListWorkInstructionsResult {
        total_count: work_instructions.items.len(),
        items: work_instructions
            .items
            .iter()
            .map(WorkInstructionSummary::from)
            .collect(),
    })
}

#[tauri::command]
pub async fn get_work_instruction(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<WorkInstruction>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = WorkInstructionService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn create_work_instruction(
    input: CreateWorkInstructionInput,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);

        let create = CreateWorkInstruction {
            title: input.title,
            author: input.author,
            document_number: input.document_number,
            revision: input.revision,
            description: input.description,
            process: input.process,
            estimated_duration_minutes: input.estimated_duration_minutes,
            tags: input.tags.unwrap_or_default(),
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn update_work_instruction(
    id: String,
    input: UpdateWorkInstructionInput,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);

        let update = UpdateWorkInstruction {
            title: input.title,
            document_number: input.document_number,
            revision: input.revision,
            description: input.description,
            estimated_duration_minutes: input.estimated_duration_minutes,
            status: input.status.and_then(|s| parse_status(&s)),
            tags: input.tags,
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn delete_work_instruction(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);
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
pub async fn get_work_instruction_stats(
    state: State<'_, AppState>,
) -> CommandResult<WorkInstructionStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = WorkInstructionService::new(project, cache);
    Ok(service.stats()?)
}

// ============================================================================
// Step Management Input DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct WiAddStepInput {
    pub action: String,
    pub verification: Option<String>,
    pub caution: Option<String>,
    pub image: Option<String>,
    pub estimated_time_minutes: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddToolInput {
    pub name: String,
    pub part_number: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WiAddMaterialInput {
    pub name: String,
    pub specification: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddQualityCheckInput {
    pub at_step: u32,
    pub characteristic: String,
    pub specification: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PpeItemInput {
    pub item: String,
    pub standard: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HazardInput {
    pub hazard: String,
    pub control: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetSafetyInput {
    pub ppe_required: Vec<PpeItemInput>,
    pub hazards: Vec<HazardInput>,
}

// ============================================================================
// Step Management Commands
// ============================================================================

#[tauri::command]
pub async fn add_work_instruction_step(
    id: String,
    input: WiAddStepInput,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);

        let step = ProcedureStep {
            step: 0, // Will be renumbered by add_step
            action: input.action,
            verification: input.verification,
            caution: input.caution,
            image: input.image,
            estimated_time_minutes: input.estimated_time_minutes,
            approval: None,
            data_fields: Vec::new(),
            equipment: Vec::new(),
        };
        service.add_step(&id, step)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn remove_work_instruction_step(
    id: String,
    step_number: u32,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);
        service.remove_step(&id, step_number)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn add_work_instruction_tool(
    id: String,
    input: AddToolInput,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);

        let tool = Tool {
            name: input.name,
            part_number: input.part_number,
        };
        service.add_tool(&id, tool)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn remove_work_instruction_tool(
    id: String,
    tool_name: String,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);
        service.remove_tool(&id, &tool_name)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn add_work_instruction_material(
    id: String,
    input: WiAddMaterialInput,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);

        let material = Material {
            name: input.name,
            specification: input.specification,
        };
        service.add_material(&id, material)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn remove_work_instruction_material(
    id: String,
    material_name: String,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);
        service.remove_material(&id, &material_name)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn add_work_instruction_quality_check(
    id: String,
    input: AddQualityCheckInput,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);

        let check = QualityCheck {
            at_step: input.at_step,
            characteristic: input.characteristic,
            specification: input.specification,
        };
        service.add_quality_check(&id, check)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn remove_work_instruction_quality_check(
    id: String,
    at_step: u32,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);
        service.remove_quality_check(&id, at_step)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn set_work_instruction_safety(
    id: String,
    input: SetSafetyInput,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);

        let safety = WorkSafety {
            ppe_required: input
                .ppe_required
                .into_iter()
                .map(|p| PpeItem {
                    item: p.item,
                    standard: p.standard,
                })
                .collect(),
            hazards: input
                .hazards
                .into_iter()
                .map(|h| Hazard {
                    hazard: h.hazard,
                    control: h.control,
                })
                .collect(),
        };
        service.set_safety(&id, safety)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}

#[tauri::command]
pub async fn clear_work_instruction_safety(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<WorkInstruction> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let work_instruction = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = WorkInstructionService::new(project, cache);
        service.clear_safety(&id)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(work_instruction)
}
