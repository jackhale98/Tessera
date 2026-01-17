//! Assembly entity commands
//!
//! Provides commands for managing assemblies and BOM hierarchy.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::assembly::{Assembly, BomItem};
use tdt_core::services::assembly::{
    AssemblyFilter, AssemblyService, AssemblySortField, AssemblyStats, BomCostResult,
    BomMassResult, BomNode, CreateAssembly, UpdateAssembly,
};
use tdt_core::services::common::SortDirection;

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Assembly summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct AssemblySummary {
    pub id: String,
    pub part_number: String,
    pub title: String,
    pub bom_count: usize,
    pub subassembly_count: usize,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Assembly> for AssemblySummary {
    fn from(asm: &Assembly) -> Self {
        Self {
            id: asm.id.to_string(),
            part_number: asm.part_number.clone(),
            title: asm.title.clone(),
            bom_count: asm.bom.len(),
            subassembly_count: asm.subassemblies.len(),
            status: format!("{:?}", asm.status).to_lowercase(),
            author: asm.author.clone(),
            created: asm.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListAssembliesResult {
    pub items: Vec<AssemblySummary>,
    pub total_count: usize,
    pub has_more: bool,
}

// ============================================================================
// List Params & Input DTOs
// ============================================================================

/// Parameters for listing assemblies
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListAssembliesParams {
    /// Filter by status
    pub status: Option<Vec<String>>,
    /// Filter by part number (substring match)
    pub part_number: Option<String>,
    /// Show only assemblies with no components
    pub empty_bom: Option<bool>,
    /// Show only assemblies with subassemblies
    pub has_subassemblies: Option<bool>,
    /// Show only top-level assemblies (no parent)
    pub top_level_only: Option<bool>,
    /// Show only sub-assemblies (have parent)
    pub sub_only: Option<bool>,
    /// Search in title/part_number
    pub search: Option<String>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort descending
    pub sort_desc: Option<bool>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Input for creating an assembly
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssemblyInput {
    pub part_number: String,
    pub title: String,
    pub author: String,
    pub revision: Option<String>,
    pub description: Option<String>,
    pub bom: Option<Vec<BomItemInput>>,
    pub subassemblies: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Input for updating an assembly
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateAssemblyInput {
    pub part_number: Option<String>,
    pub title: Option<String>,
    pub revision: Option<String>,
    pub description: Option<String>,
    pub bom: Option<Vec<BomItemInput>>,
    pub subassemblies: Option<Vec<String>>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// BOM item input
#[derive(Debug, Clone, Deserialize)]
pub struct BomItemInput {
    pub component_id: String,
    pub quantity: u32,
    pub reference_designators: Option<Vec<String>>,
    pub notes: Option<String>,
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

fn parse_sort_field(s: &str) -> AssemblySortField {
    match s.to_lowercase().as_str() {
        "id" => AssemblySortField::Id,
        "part_number" | "partnumber" => AssemblySortField::PartNumber,
        "title" => AssemblySortField::Title,
        "bom_count" | "bomcount" => AssemblySortField::BomCount,
        "status" => AssemblySortField::Status,
        "author" => AssemblySortField::Author,
        _ => AssemblySortField::Created,
    }
}

fn build_assembly_filter(params: &ListAssembliesParams) -> AssemblyFilter {
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
        tags: params.tags.clone(),
        limit: params.limit,
        offset: params.offset,
        ..Default::default()
    };

    AssemblyFilter {
        common,
        part_number: params.part_number.clone(),
        empty_bom: params.empty_bom.unwrap_or(false),
        has_subassemblies: params.has_subassemblies.unwrap_or(false),
        top_level_only: params.top_level_only.unwrap_or(false),
        sub_only: params.sub_only.unwrap_or(false),
    }
}

fn bom_item_from_input(input: BomItemInput) -> BomItem {
    BomItem {
        component_id: input.component_id,
        quantity: input.quantity,
        reference_designators: input.reference_designators.unwrap_or_default(),
        notes: input.notes,
    }
}

// ============================================================================
// Commands
// ============================================================================

/// List assemblies
#[tauri::command]
pub async fn list_assemblies(
    params: Option<ListAssembliesParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListAssembliesResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = build_assembly_filter(&params);
    let sort_field = params
        .sort_by
        .as_ref()
        .map(|s| parse_sort_field(s))
        .unwrap_or_default();
    let sort_dir = if params.sort_desc.unwrap_or(false) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    let result = service.list(&filter, sort_field, sort_dir)?;
    let total_count = result.total_count;

    Ok(ListAssembliesResult {
        items: result.items.iter().map(AssemblySummary::from).collect(),
        total_count,
        has_more: result.has_more,
    })
}

/// Get a single assembly by ID
#[tauri::command]
pub async fn get_assembly(id: String, state: State<'_, AppState>) -> CommandResult<Option<Assembly>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let assembly = service.get(&id)?;

    Ok(assembly)
}

/// Get an assembly by part number
#[tauri::command]
pub async fn get_assembly_by_part_number(
    part_number: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<Assembly>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let assembly = service.get_by_part_number(&part_number)?;

    Ok(assembly)
}

/// Create a new assembly
#[tauri::command]
pub async fn create_assembly(
    input: CreateAssemblyInput,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);

        let create = CreateAssembly {
            part_number: input.part_number,
            title: input.title,
            author: input.author,
            revision: input.revision,
            description: input.description,
            bom: input
                .bom
                .map(|b| b.into_iter().map(bom_item_from_input).collect())
                .unwrap_or_default(),
            subassemblies: input.subassemblies.unwrap_or_default(),
            tags: input.tags.unwrap_or_default(),
        };

        service.create(create)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Update an existing assembly
#[tauri::command]
pub async fn update_assembly(
    id: String,
    input: UpdateAssemblyInput,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);

        let update = UpdateAssembly {
            part_number: input.part_number,
            title: input.title,
            revision: input.revision,
            description: input.description,
            bom: input
                .bom
                .map(|b| b.into_iter().map(bom_item_from_input).collect()),
            subassemblies: input.subassemblies,
            status: input.status.and_then(|s| parse_status(&s)),
            tags: input.tags,
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

    Ok(assembly)
}

/// Delete an assembly
#[tauri::command]
pub async fn delete_assembly(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.delete(&id, force.unwrap_or(false))?;
    }

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Add a component to an assembly's BOM
#[tauri::command]
pub async fn add_assembly_component(
    assembly_id: String,
    component_id: String,
    quantity: u32,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.add_component(&assembly_id, &component_id, quantity)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Remove a component from an assembly's BOM
#[tauri::command]
pub async fn remove_assembly_component(
    assembly_id: String,
    component_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.remove_component(&assembly_id, &component_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Add a subassembly reference
#[tauri::command]
pub async fn add_subassembly(
    assembly_id: String,
    subassembly_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.add_subassembly(&assembly_id, &subassembly_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Remove a subassembly reference
#[tauri::command]
pub async fn remove_subassembly(
    assembly_id: String,
    subassembly_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.remove_subassembly(&assembly_id, &subassembly_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Get the full BOM tree for an assembly
#[tauri::command]
pub async fn get_bom_tree(id: String, state: State<'_, AppState>) -> CommandResult<BomNode> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let tree = service.get_bom_tree(&id)?;

    Ok(tree)
}

/// Calculate total cost for an assembly
#[tauri::command]
pub async fn calculate_assembly_cost(
    id: String,
    quantity: Option<u32>,
    state: State<'_, AppState>,
) -> CommandResult<BomCostResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let result = service.calculate_cost(&id, quantity.unwrap_or(1))?;

    Ok(result)
}

/// Calculate total mass for an assembly
#[tauri::command]
pub async fn calculate_assembly_mass(
    id: String,
    quantity: Option<u32>,
    state: State<'_, AppState>,
) -> CommandResult<BomMassResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let result = service.calculate_mass(&id, quantity.unwrap_or(1))?;

    Ok(result)
}

/// Get manufacturing routing for an assembly
#[tauri::command]
pub async fn get_assembly_routing(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<String>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let routing = service.get_routing(&id)?;

    Ok(routing)
}

/// Set manufacturing routing for an assembly
#[tauri::command]
pub async fn set_assembly_routing(
    id: String,
    routing: Vec<String>,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.set_routing(&id, routing)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Add a process to manufacturing routing
#[tauri::command]
pub async fn add_assembly_routing_process(
    id: String,
    process_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.add_routing_process(&id, &process_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Remove a process from manufacturing routing
#[tauri::command]
pub async fn remove_assembly_routing_process(
    id: String,
    process_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Assembly> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let assembly = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = AssemblyService::new(project, cache);
        service.remove_routing_process(&id, &process_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(assembly)
}

/// Get assembly statistics
#[tauri::command]
pub async fn get_assembly_stats(state: State<'_, AppState>) -> CommandResult<AssemblyStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = AssemblyService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}
