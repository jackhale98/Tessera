//! Control entity commands
//!
//! Provides commands for managing process controls.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::control::{Control, ControlCategory, ControlType};
use tdt_core::services::common::SortDirection;
use tdt_core::services::control::{
    ControlFilter, ControlService, ControlSortField, ControlStats, CreateControl, UpdateControl,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Control summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct ControlSummary {
    pub id: String,
    pub title: String,
    pub control_type: String,
    pub control_category: String,
    pub process: Option<String>,
    pub feature: Option<String>,
    pub critical: bool,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Control> for ControlSummary {
    fn from(c: &Control) -> Self {
        Self {
            id: c.id.to_string(),
            title: c.title.clone(),
            control_type: c.control_type.to_string(),
            control_category: c.control_category.to_string(),
            process: c.links.process.as_ref().map(|p| p.to_string()),
            feature: c.links.feature.as_ref().map(|f| f.to_string()),
            critical: c.characteristic.critical,
            status: format!("{:?}", c.status).to_lowercase(),
            author: c.author.clone(),
            created: c.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListControlsResult {
    pub items: Vec<ControlSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListControlsParams {
    pub status: Option<Vec<String>>,
    pub control_type: Option<String>,
    pub control_category: Option<String>,
    pub process: Option<String>,
    pub critical_only: Option<bool>,
    pub has_limits: Option<bool>,
    pub has_measurement: Option<bool>,
    pub has_sampling: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateControlInput {
    pub title: String,
    pub author: String,
    pub control_type: Option<String>,
    pub control_category: Option<String>,
    pub description: Option<String>,
    pub process: Option<String>,
    pub feature: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateControlInput {
    pub title: Option<String>,
    pub control_type: Option<String>,
    pub control_category: Option<String>,
    pub description: Option<String>,
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

fn parse_control_type(s: &str) -> Option<ControlType> {
    match s.to_lowercase().as_str() {
        "spc" => Some(ControlType::Spc),
        "inspection" => Some(ControlType::Inspection),
        "poka_yoke" | "pokayoke" => Some(ControlType::PokaYoke),
        "visual" => Some(ControlType::Visual),
        "functional" | "functional_test" => Some(ControlType::FunctionalTest),
        "attribute" => Some(ControlType::Attribute),
        _ => None,
    }
}

fn parse_control_category(s: &str) -> Option<ControlCategory> {
    match s.to_lowercase().as_str() {
        "variable" => Some(ControlCategory::Variable),
        "attribute" => Some(ControlCategory::Attribute),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> ControlSortField {
    match s.to_lowercase().as_str() {
        "id" => ControlSortField::Id,
        "title" => ControlSortField::Title,
        "control_type" | "type" => ControlSortField::ControlType,
        "status" => ControlSortField::Status,
        "author" => ControlSortField::Author,
        "created" => ControlSortField::Created,
        _ => ControlSortField::Title,
    }
}

fn build_control_filter(params: &ListControlsParams) -> ControlFilter {
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

    ControlFilter {
        common,
        control_type: params
            .control_type
            .as_ref()
            .and_then(|t| parse_control_type(t)),
        control_category: params
            .control_category
            .as_ref()
            .and_then(|c| parse_control_category(c)),
        process: params.process.clone(),
        critical_only: params.critical_only.unwrap_or(false),
        has_limits: params.has_limits.unwrap_or(false),
        has_measurement: params.has_measurement.unwrap_or(false),
        has_sampling: params.has_sampling.unwrap_or(false),
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_controls(
    params: Option<ListControlsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListControlsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ControlService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_control_filter(&params);

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

    let controls = service.list(&filter, sort, sort_direction)?;

    Ok(ListControlsResult {
        total_count: controls.items.len(),
        items: controls.items.iter().map(ControlSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_control(id: String, state: State<'_, AppState>) -> CommandResult<Option<Control>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ControlService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn create_control(
    input: CreateControlInput,
    state: State<'_, AppState>,
) -> CommandResult<Control> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let control = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ControlService::new(project, cache);

        let create = CreateControl {
            title: input.title,
            author: input.author,
            control_type: input
                .control_type
                .and_then(|t| parse_control_type(&t))
                .unwrap_or_default(),
            control_category: input
                .control_category
                .and_then(|c| parse_control_category(&c))
                .unwrap_or_default(),
            description: input.description,
            characteristic: None,
            process: input.process,
            feature: input.feature,
            tags: input.tags.unwrap_or_default(),
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(control)
}

#[tauri::command]
pub async fn update_control(
    id: String,
    input: UpdateControlInput,
    state: State<'_, AppState>,
) -> CommandResult<Control> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let control = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ControlService::new(project, cache);

        let update = UpdateControl {
            title: input.title,
            control_type: input.control_type.and_then(|t| parse_control_type(&t)),
            control_category: input
                .control_category
                .and_then(|c| parse_control_category(&c)),
            description: input.description,
            characteristic: None,
            measurement: None,
            sampling: None,
            control_limits: None,
            reaction_plan: None,
            status: input.status.and_then(|s| parse_status(&s)),
            tags: input.tags,
            process: None,
            feature: None,
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(control)
}

#[tauri::command]
pub async fn delete_control(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = ControlService::new(project, cache);
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
pub async fn get_control_stats(state: State<'_, AppState>) -> CommandResult<ControlStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ControlService::new(project, cache);
    Ok(service.stats()?)
}
