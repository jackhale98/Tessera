//! Component entity commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::entity::Status;
use tdt_core::entities::component::{Component, ComponentCategory, MakeBuy};
use tdt_core::services::{
    BomCostSummary, ComponentFilter, ComponentService, ComponentSortField, ComponentStats,
    CreateComponent, SortDirection, UpdateComponent,
};

/// List parameters for components
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListComponentsParams {
    pub status: Option<Vec<String>>,
    pub category: Option<String>,
    pub make_buy: Option<String>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// Component summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct ComponentSummary {
    pub id: String,
    pub title: String,
    pub part_number: String,
    pub revision: Option<String>,
    pub category: String,
    pub make_buy: String,
    pub unit_cost: Option<f64>,
    pub mass_kg: Option<f64>,
    pub status: String,
    pub author: String,
    pub created: String,
    pub tags: Vec<String>,
}

impl From<&Component> for ComponentSummary {
    fn from(cmp: &Component) -> Self {
        Self {
            id: cmp.id.to_string(),
            title: cmp.title.clone(),
            part_number: cmp.part_number.clone(),
            revision: cmp.revision.clone(),
            category: format!("{:?}", cmp.category).to_lowercase(),
            make_buy: format!("{:?}", cmp.make_buy).to_lowercase(),
            unit_cost: cmp.unit_cost,
            mass_kg: cmp.mass_kg,
            status: format!("{:?}", cmp.status).to_lowercase(),
            author: cmp.author.clone(),
            created: cmp.created.to_rfc3339(),
            tags: cmp.tags.clone(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListComponentsResult {
    pub items: Vec<ComponentSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Input for creating a component
#[derive(Debug, Clone, Deserialize)]
pub struct CreateComponentInput {
    pub title: String,
    pub part_number: String,
    pub author: String,
    pub revision: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub make_buy: Option<String>,
    pub unit_cost: Option<f64>,
    pub mass_kg: Option<f64>,
    pub material: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Input for updating a component
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateComponentInput {
    pub title: Option<String>,
    pub part_number: Option<String>,
    pub revision: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub make_buy: Option<String>,
    pub unit_cost: Option<f64>,
    pub mass_kg: Option<f64>,
    pub material: Option<String>,
    pub status: Option<String>,
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

fn parse_category(s: &str) -> Option<ComponentCategory> {
    match s.to_lowercase().as_str() {
        "mechanical" => Some(ComponentCategory::Mechanical),
        "electrical" => Some(ComponentCategory::Electrical),
        "software" => Some(ComponentCategory::Software),
        "fastener" => Some(ComponentCategory::Fastener),
        "consumable" => Some(ComponentCategory::Consumable),
        _ => None,
    }
}

fn parse_make_buy(s: &str) -> Option<MakeBuy> {
    match s.to_lowercase().as_str() {
        "make" => Some(MakeBuy::Make),
        "buy" => Some(MakeBuy::Buy),
        _ => None,
    }
}

/// List components with optional filters
#[tauri::command]
pub async fn list_components(
    params: Option<ListComponentsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListComponentsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ComponentService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = ComponentFilter {
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
        category: params.category.and_then(|s| parse_category(&s)),
        make_buy: params.make_buy.and_then(|s| parse_make_buy(&s)),
        ..Default::default()
    };

    let sort_field = params
        .sort_by
        .map(|s| match s.as_str() {
            "title" => ComponentSortField::Title,
            "part_number" => ComponentSortField::PartNumber,
            "status" => ComponentSortField::Status,
            "created" => ComponentSortField::Created,
            "category" => ComponentSortField::Category,
            "cost" => ComponentSortField::UnitCost,
            _ => ComponentSortField::Created,
        })
        .unwrap_or(ComponentSortField::Created);

    let sort_dir = if params.sort_desc.unwrap_or(true) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    let result = service.list(&filter, sort_field, sort_dir)?;

    Ok(ListComponentsResult {
        items: result.items.iter().map(ComponentSummary::from).collect(),
        total_count: result.total_count,
        has_more: result.has_more,
    })
}

/// Get a single component by ID
#[tauri::command]
pub async fn get_component(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<Component>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ComponentService::new(project, cache);
    let cmp = service.get(&id)?;

    Ok(cmp)
}

/// Get a component by part number
#[tauri::command]
pub async fn get_component_by_part_number(
    part_number: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<Component>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ComponentService::new(project, cache);
    let cmp = service.get_by_part_number(&part_number)?;

    Ok(cmp)
}

/// Create a new component
#[tauri::command]
pub async fn create_component(
    input: CreateComponentInput,
    state: State<'_, AppState>,
) -> CommandResult<Component> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    // Create component using service
    let cmp = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ComponentService::new(project, cache);

        let create = CreateComponent {
            title: input.title,
            part_number: input.part_number,
            author: input.author,
            revision: input.revision,
            description: input.description,
            category: input
                .category
                .and_then(|s| parse_category(&s))
                .unwrap_or_default(),
            make_buy: input
                .make_buy
                .and_then(|s| parse_make_buy(&s))
                .unwrap_or_default(),
            unit_cost: input.unit_cost,
            mass_kg: input.mass_kg,
            material: input.material,
            tags: input.tags.unwrap_or_default(),
            suppliers: vec![],
        };

        service.create(create)?
    };

    // Sync cache to pick up the new entity
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(cmp)
}

/// Update an existing component
#[tauri::command]
pub async fn update_component(
    id: String,
    input: UpdateComponentInput,
    state: State<'_, AppState>,
) -> CommandResult<Component> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let cmp = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ComponentService::new(project, cache);

        let update = UpdateComponent {
            title: input.title,
            part_number: input.part_number,
            revision: input.revision,
            description: input.description,
            category: input.category.and_then(|s| parse_category(&s)),
            make_buy: input.make_buy.and_then(|s| parse_make_buy(&s)),
            unit_cost: input.unit_cost,
            mass_kg: input.mass_kg,
            material: input.material,
            status: input.status.and_then(|s| parse_status(&s)),
            tags: None,
            suppliers: None,
            documents: None,
        };

        service.update(&id, update)?
    };

    // Sync cache to pick up the changes
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(cmp)
}

/// Delete a component
#[tauri::command]
pub async fn delete_component(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ComponentService::new(project, cache);
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

/// Get component statistics
#[tauri::command]
pub async fn get_component_stats(state: State<'_, AppState>) -> CommandResult<ComponentStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ComponentService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}

/// Get BOM cost summary
#[tauri::command]
pub async fn get_bom_cost_summary(state: State<'_, AppState>) -> CommandResult<BomCostSummary> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ComponentService::new(project, cache);
    let summary = service.get_cost_summary()?;

    Ok(summary)
}
