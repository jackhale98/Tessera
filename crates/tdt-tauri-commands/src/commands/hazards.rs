//! Hazard entity commands
//!
//! Provides commands for managing safety hazards.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::hazard::{Hazard, HazardCategory, HazardSeverity};
use tdt_core::services::common::SortDirection;
use tdt_core::services::hazard::{
    CreateHazard, HazardFilter, HazardService, HazardSortField, HazardStats, UpdateHazard,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Hazard summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct HazardSummary {
    pub id: String,
    pub title: String,
    pub category: String,
    pub severity: String,
    pub risk_count: usize,
    pub control_count: usize,
    pub is_controlled: bool,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Hazard> for HazardSummary {
    fn from(h: &Hazard) -> Self {
        Self {
            id: h.id.to_string(),
            title: h.title.clone(),
            category: h.category.to_string(),
            severity: h.severity.to_string(),
            risk_count: h.risk_count(),
            control_count: h.control_count(),
            is_controlled: h.is_controlled(),
            status: format!("{:?}", h.status).to_lowercase(),
            author: h.author.clone(),
            created: h.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListHazardsResult {
    pub items: Vec<HazardSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListHazardsParams {
    pub status: Option<Vec<String>>,
    pub category: Option<String>,
    pub severity: Option<String>,
    pub uncontrolled_only: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateHazardInput {
    pub title: String,
    pub author: String,
    pub category: String,
    pub description: String,
    pub potential_harms: Option<Vec<String>>,
    pub energy_level: Option<String>,
    pub severity: String,
    pub exposure_scenario: Option<String>,
    pub affected_populations: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateHazardInput {
    pub title: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub energy_level: Option<String>,
    pub severity: Option<String>,
    pub exposure_scenario: Option<String>,
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

fn parse_hazard_category(s: &str) -> Option<HazardCategory> {
    match s.to_lowercase().as_str() {
        "electrical" => Some(HazardCategory::Electrical),
        "mechanical" => Some(HazardCategory::Mechanical),
        "thermal" => Some(HazardCategory::Thermal),
        "chemical" => Some(HazardCategory::Chemical),
        "biological" => Some(HazardCategory::Biological),
        "radiation" => Some(HazardCategory::Radiation),
        "ergonomic" => Some(HazardCategory::Ergonomic),
        "software" => Some(HazardCategory::Software),
        "environmental" => Some(HazardCategory::Environmental),
        _ => None,
    }
}

fn parse_hazard_severity(s: &str) -> Option<HazardSeverity> {
    match s.to_lowercase().as_str() {
        "negligible" => Some(HazardSeverity::Negligible),
        "minor" => Some(HazardSeverity::Minor),
        "serious" => Some(HazardSeverity::Serious),
        "severe" => Some(HazardSeverity::Severe),
        "catastrophic" => Some(HazardSeverity::Catastrophic),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> HazardSortField {
    match s.to_lowercase().as_str() {
        "id" => HazardSortField::Id,
        "title" => HazardSortField::Title,
        "category" => HazardSortField::Category,
        "severity" => HazardSortField::Severity,
        "risk_count" | "risks" => HazardSortField::RiskCount,
        "control_count" | "controls" => HazardSortField::ControlCount,
        "status" => HazardSortField::Status,
        "author" => HazardSortField::Author,
        "created" => HazardSortField::Created,
        _ => HazardSortField::Created,
    }
}

fn build_hazard_filter(params: &ListHazardsParams) -> HazardFilter {
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

    HazardFilter {
        common,
        category: params
            .category
            .as_ref()
            .and_then(|c| parse_hazard_category(c)),
        severity: params
            .severity
            .as_ref()
            .and_then(|s| parse_hazard_severity(s)),
        uncontrolled_only: params.uncontrolled_only.unwrap_or(false),
        recent_days: None,
        sort,
        sort_direction,
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_hazards(
    params: Option<ListHazardsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListHazardsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = HazardService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_hazard_filter(&params);
    let hazards = service.list(&filter)?;

    Ok(ListHazardsResult {
        total_count: hazards.len(),
        items: hazards.iter().map(HazardSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_hazard(id: String, state: State<'_, AppState>) -> CommandResult<Option<Hazard>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = HazardService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn create_hazard(
    input: CreateHazardInput,
    state: State<'_, AppState>,
) -> CommandResult<Hazard> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let hazard = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = HazardService::new(project, cache);

        let create = CreateHazard {
            title: input.title,
            author: input.author,
            category: parse_hazard_category(&input.category).unwrap_or(HazardCategory::Electrical),
            description: input.description,
            potential_harms: input.potential_harms.unwrap_or_default(),
            energy_level: input.energy_level,
            severity: parse_hazard_severity(&input.severity).unwrap_or(HazardSeverity::Minor),
            exposure_scenario: input.exposure_scenario,
            affected_populations: input.affected_populations.unwrap_or_default(),
            tags: input.tags.unwrap_or_default(),
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(hazard)
}

#[tauri::command]
pub async fn update_hazard(
    id: String,
    input: UpdateHazardInput,
    state: State<'_, AppState>,
) -> CommandResult<Hazard> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let hazard = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = HazardService::new(project, cache);

        let update = UpdateHazard {
            title: input.title,
            category: input.category.and_then(|c| parse_hazard_category(&c)),
            description: input.description,
            energy_level: input.energy_level.map(Some),
            severity: input.severity.and_then(|s| parse_hazard_severity(&s)),
            exposure_scenario: input.exposure_scenario.map(Some),
            status: input.status.and_then(|s| parse_status(&s)),
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(hazard)
}

#[tauri::command]
pub async fn delete_hazard(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = HazardService::new(project, cache);
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
pub async fn get_hazard_stats(state: State<'_, AppState>) -> CommandResult<HazardStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = HazardService::new(project, cache);
    Ok(service.stats()?)
}
