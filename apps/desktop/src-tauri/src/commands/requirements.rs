//! Requirement-specific commands
//!
//! Provides commands for managing requirements with filtering and stats.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::{Priority, Status};
use tdt_core::entities::requirement::{Level, Requirement, RequirementType};
use tdt_core::services::common::SortDirection;
use tdt_core::services::requirement::{
    RequirementFilter, RequirementService, RequirementSortField,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Requirement summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct RequirementSummary {
    pub id: String,
    pub title: String,
    pub req_type: String,
    pub level: String,
    pub priority: String,
    pub category: Option<String>,
    pub status: String,
    pub author: String,
    pub created: String,
    pub tags: Vec<String>,
}

impl From<&Requirement> for RequirementSummary {
    fn from(r: &Requirement) -> Self {
        Self {
            id: r.id.to_string(),
            title: r.title.clone(),
            req_type: format!("{:?}", r.req_type).to_lowercase(),
            level: format!("{:?}", r.level).to_lowercase(),
            priority: format!("{:?}", r.priority).to_lowercase(),
            category: r.category.clone(),
            status: format!("{:?}", r.status).to_lowercase(),
            author: r.author.clone(),
            created: r.created.to_rfc3339(),
            tags: r.tags.clone(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListRequirementsResult {
    pub items: Vec<RequirementSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListRequirementsParams {
    pub status: Option<Vec<String>>,
    pub req_type: Option<String>,
    pub level: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub orphans_only: Option<bool>,
    pub unverified_only: Option<bool>,
    pub needs_review: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
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

fn parse_req_type(s: &str) -> Option<RequirementType> {
    match s.to_lowercase().as_str() {
        "input" => Some(RequirementType::Input),
        "output" => Some(RequirementType::Output),
        _ => None,
    }
}

fn parse_level(s: &str) -> Option<Level> {
    match s.to_lowercase().as_str() {
        "stakeholder" => Some(Level::Stakeholder),
        "system" => Some(Level::System),
        "subsystem" => Some(Level::Subsystem),
        "component" => Some(Level::Component),
        "detail" => Some(Level::Detail),
        _ => None,
    }
}

fn parse_priority(s: &str) -> Option<Priority> {
    match s.to_lowercase().as_str() {
        "low" => Some(Priority::Low),
        "medium" => Some(Priority::Medium),
        "high" => Some(Priority::High),
        "critical" => Some(Priority::Critical),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> RequirementSortField {
    match s.to_lowercase().as_str() {
        "id" => RequirementSortField::Id,
        "title" => RequirementSortField::Title,
        "type" | "req_type" => RequirementSortField::Type,
        "level" => RequirementSortField::Level,
        "priority" => RequirementSortField::Priority,
        "category" => RequirementSortField::Category,
        "status" => RequirementSortField::Status,
        "author" => RequirementSortField::Author,
        "created" => RequirementSortField::Created,
        _ => RequirementSortField::Created,
    }
}

fn build_requirement_filter(params: &ListRequirementsParams) -> RequirementFilter {
    use tdt_core::services::common::CommonFilter;

    let common = CommonFilter {
        status: params.status.as_ref().and_then(|v| {
            let statuses: Vec<Status> = v.iter().filter_map(|s| parse_status(s)).collect();
            if statuses.is_empty() { None } else { Some(statuses) }
        }),
        priority: params.priority.as_ref().and_then(|p| parse_priority(p)).map(|p| vec![p]),
        search: params.search.clone(),
        limit: params.limit,
        ..Default::default()
    };

    RequirementFilter {
        common,
        req_type: params.req_type.as_ref().and_then(|t| parse_req_type(t)),
        level: params.level.as_ref().and_then(|l| parse_level(l)),
        category: params.category.clone(),
        orphans_only: params.orphans_only.unwrap_or(false),
        unverified_only: params.unverified_only.unwrap_or(false),
        needs_review: params.needs_review.unwrap_or(false),
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_requirements(
    params: Option<ListRequirementsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListRequirementsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RequirementService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_requirement_filter(&params);

    let sort = params.sort_by.as_ref().map(|s| parse_sort_field(s)).unwrap_or_default();
    let sort_direction = if params.sort_desc.unwrap_or(false) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    let requirements = service.list(&filter, sort, sort_direction)?;

    Ok(ListRequirementsResult {
        total_count: requirements.items.len(),
        items: requirements.items.iter().map(RequirementSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_requirement(id: String, state: State<'_, AppState>) -> CommandResult<Option<Requirement>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RequirementService::new(project, cache);
    Ok(service.get(&id)?)
}

/// Requirement statistics
#[derive(Debug, Clone, Serialize)]
pub struct RequirementStatsResponse {
    pub total: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub unverified: usize,
    pub orphaned: usize,
    pub by_status: StatusCountsResponse,
}

/// Status counts
#[derive(Debug, Clone, Serialize)]
pub struct StatusCountsResponse {
    pub draft: usize,
    pub review: usize,
    pub approved: usize,
    pub released: usize,
    pub obsolete: usize,
}

/// Get requirement statistics
#[tauri::command]
pub async fn get_requirement_stats(
    state: State<'_, AppState>,
) -> CommandResult<RequirementStatsResponse> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RequirementService::new(project, cache);
    let stats = service.stats()?;

    Ok(RequirementStatsResponse {
        total: stats.total,
        inputs: stats.inputs,
        outputs: stats.outputs,
        unverified: stats.unverified,
        orphaned: stats.orphaned,
        by_status: StatusCountsResponse {
            draft: stats.by_status.draft,
            review: stats.by_status.review,
            approved: stats.by_status.approved,
            released: stats.by_status.released,
            obsolete: stats.by_status.obsolete,
        },
    })
}
