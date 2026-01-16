//! Risk entity commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::entity::{Priority, Status};
use tdt_core::entities::risk::{Mitigation, MitigationStatus, MitigationType, Risk, RiskLevel, RiskType};
use tdt_core::services::{
    CreateRisk, RiskFilter, RiskMatrix, RiskService, RiskSortField, RiskStats, SortDirection,
    UpdateRisk,
};

/// List parameters for risks
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListRisksParams {
    pub status: Option<Vec<String>>,
    pub priority: Option<Vec<String>>,
    pub risk_type: Option<String>,
    pub risk_level: Option<String>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub min_rpn: Option<u32>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// Risk summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct RiskSummary {
    pub id: String,
    pub title: String,
    pub risk_type: String,
    pub failure_mode: String,
    pub severity: Option<u8>,
    pub occurrence: Option<u8>,
    pub detection: Option<u8>,
    pub rpn: Option<u32>,
    pub risk_level: Option<String>,
    pub status: String,
    pub author: String,
    pub created: String,
    pub tags: Vec<String>,
    pub mitigation_count: usize,
}

impl From<&Risk> for RiskSummary {
    fn from(risk: &Risk) -> Self {
        Self {
            id: risk.id.to_string(),
            title: risk.title.clone(),
            risk_type: format!("{:?}", risk.risk_type).to_lowercase(),
            failure_mode: risk.failure_mode.clone().unwrap_or_default(),
            severity: risk.severity,
            occurrence: risk.occurrence,
            detection: risk.detection,
            rpn: risk.get_rpn().map(|r| r as u32),
            risk_level: risk.get_risk_level().map(|l| format!("{:?}", l).to_lowercase()),
            status: format!("{:?}", risk.status).to_lowercase(),
            author: risk.author.clone(),
            created: risk.created.to_rfc3339(),
            tags: risk.tags.clone(),
            mitigation_count: risk.mitigations.len(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListRisksResult {
    pub items: Vec<RiskSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Input for creating a risk
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRiskInput {
    pub title: String,
    pub description: String,
    pub author: String,
    pub risk_type: Option<String>,
    pub category: Option<String>,
    pub failure_mode: Option<String>,
    pub cause: Option<String>,
    pub effect: Option<String>,
    pub severity: Option<u8>,
    pub occurrence: Option<u8>,
    pub detection: Option<u8>,
    pub tags: Option<Vec<String>>,
}

/// Input for updating a risk
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRiskInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub risk_type: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    pub failure_mode: Option<String>,
    pub cause: Option<String>,
    pub effect: Option<String>,
    pub severity: Option<u8>,
    pub occurrence: Option<u8>,
    pub detection: Option<u8>,
    pub tags: Option<Vec<String>>,
}

/// Input for adding a mitigation
#[derive(Debug, Clone, Deserialize)]
pub struct AddMitigationInput {
    pub action: String,
    pub mitigation_type: Option<String>,
    pub owner: Option<String>,
    pub due_date: Option<String>,
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

fn parse_priority(s: &str) -> Option<Priority> {
    match s.to_lowercase().as_str() {
        "low" => Some(Priority::Low),
        "medium" => Some(Priority::Medium),
        "high" => Some(Priority::High),
        "critical" => Some(Priority::Critical),
        _ => None,
    }
}

fn parse_risk_type(s: &str) -> Option<RiskType> {
    match s.to_lowercase().as_str() {
        "design" => Some(RiskType::Design),
        "process" => Some(RiskType::Process),
        "use" => Some(RiskType::Use),
        "software" => Some(RiskType::Software),
        _ => None,
    }
}

fn parse_risk_level(s: &str) -> Option<RiskLevel> {
    match s.to_lowercase().as_str() {
        "low" => Some(RiskLevel::Low),
        "medium" => Some(RiskLevel::Medium),
        "high" => Some(RiskLevel::High),
        "critical" => Some(RiskLevel::Critical),
        _ => None,
    }
}

fn parse_mitigation_type(s: &str) -> Option<MitigationType> {
    match s.to_lowercase().as_str() {
        "prevention" => Some(MitigationType::Prevention),
        "detection" => Some(MitigationType::Detection),
        _ => None,
    }
}

/// List risks with optional filters
#[tauri::command]
pub async fn list_risks(
    params: Option<ListRisksParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListRisksResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RiskService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = RiskFilter {
        common: tdt_core::services::CommonFilter {
            status: params.status.map(|v| v.iter().filter_map(|s| parse_status(s)).collect()),
            priority: params.priority.map(|v| v.iter().filter_map(|s| parse_priority(s)).collect()),
            search: params.search,
            tags: params.tags,
            limit: params.limit,
            offset: params.offset,
            ..Default::default()
        },
        risk_type: params.risk_type.and_then(|s| parse_risk_type(&s)),
        risk_level: params.risk_level.and_then(|s| parse_risk_level(&s)).map(|l| vec![l]),
        min_rpn: params.min_rpn.map(|r| r as u16),
        max_rpn: None,
        min_severity: None,
        min_occurrence: None,
        min_detection: None,
        unmitigated_only: false,
        needs_mitigation: false,
        needs_verification: false,
        category: None,
    };

    let sort_field = params
        .sort_by
        .map(|s| match s.as_str() {
            "title" => RiskSortField::Title,
            "status" => RiskSortField::Status,
            "author" => RiskSortField::Author,
            "created" => RiskSortField::Created,
            "rpn" => RiskSortField::Rpn,
            "severity" => RiskSortField::Severity,
            "risk_level" => RiskSortField::RiskLevel,
            _ => RiskSortField::Created,
        })
        .unwrap_or(RiskSortField::Created);

    let sort_dir = if params.sort_desc.unwrap_or(true) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    let result = service.list(&filter, sort_field, sort_dir)?;

    Ok(ListRisksResult {
        items: result.items.iter().map(RiskSummary::from).collect(),
        total_count: result.total_count,
        has_more: result.has_more,
    })
}

/// Get a single risk by ID
#[tauri::command]
pub async fn get_risk(id: String, state: State<'_, AppState>) -> CommandResult<Option<Risk>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RiskService::new(project, cache);
    let risk = service.get(&id)?;

    Ok(risk)
}

/// Create a new risk
#[tauri::command]
pub async fn create_risk(
    input: CreateRiskInput,
    state: State<'_, AppState>,
) -> CommandResult<Risk> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let risk = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = RiskService::new(project, cache);

        let create = CreateRisk {
            title: input.title,
            description: input.description,
            author: input.author,
            risk_type: input.risk_type.and_then(|s| parse_risk_type(&s)).unwrap_or(RiskType::Design),
            category: input.category,
            failure_mode: input.failure_mode,
            cause: input.cause,
            effect: input.effect,
            severity: input.severity,
            occurrence: input.occurrence,
            detection: input.detection,
            tags: input.tags.unwrap_or_default(),
        };

        service.create(create)?
    };

    // Sync cache to pick up the new entity
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(risk)
}

/// Update an existing risk
#[tauri::command]
pub async fn update_risk(
    id: String,
    input: UpdateRiskInput,
    state: State<'_, AppState>,
) -> CommandResult<Risk> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let risk = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = RiskService::new(project, cache);

        let update = UpdateRisk {
            title: input.title,
            description: input.description,
            risk_type: input.risk_type.and_then(|s| parse_risk_type(&s)),
            status: input.status.and_then(|s| parse_status(&s)),
            category: input.category,
            tags: input.tags,
            failure_mode: input.failure_mode,
            cause: input.cause,
            effect: input.effect,
            severity: input.severity,
            occurrence: input.occurrence,
            detection: input.detection,
            mitigations: None,
            risk_level: None,
        };

        service.update(&id, update)?
    };

    // Sync cache to pick up the changes
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(risk)
}

/// Delete a risk
#[tauri::command]
pub async fn delete_risk(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = RiskService::new(project, cache);
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

/// Add a mitigation to a risk
#[tauri::command]
pub async fn add_risk_mitigation(
    id: String,
    input: AddMitigationInput,
    state: State<'_, AppState>,
) -> CommandResult<Risk> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RiskService::new(project, cache);

    let mitigation = Mitigation {
        action: input.action,
        mitigation_type: input.mitigation_type.and_then(|s| parse_mitigation_type(&s)),
        status: Some(MitigationStatus::Proposed),
        owner: input.owner,
        due_date: input.due_date.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
    };

    let risk = service.add_mitigation(&id, mitigation)?;
    Ok(risk)
}

/// Get risk statistics
#[tauri::command]
pub async fn get_risk_stats(state: State<'_, AppState>) -> CommandResult<RiskStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RiskService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}

/// Get risk matrix
#[tauri::command]
pub async fn get_risk_matrix(state: State<'_, AppState>) -> CommandResult<RiskMatrix> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = RiskService::new(project, cache);
    let matrix = service.get_risk_matrix()?;

    Ok(matrix)
}
