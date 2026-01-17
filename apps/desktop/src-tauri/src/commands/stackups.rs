//! Stackup (tolerance analysis) entity commands
//!
//! Provides commands for managing tolerance stackups and running analyses.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::core::identity::EntityId;
use tdt_core::entities::stackup::{
    AnalysisResult, Direction, Disposition, Distribution, MonteCarloResult, RssResult, Stackup,
    WorstCaseResult,
};
use tdt_core::services::common::SortDirection;
use tdt_core::services::stackup::{
    AddContributorInput, CreateStackup, StackupFilter, StackupService, StackupSortField,
    StackupStats, UpdateStackup,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Stackup summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct StackupSummary {
    pub id: String,
    pub title: String,
    pub target_name: String,
    pub critical: bool,
    pub result: Option<String>,
    pub disposition: String,
    pub cpk: Option<f64>,
    pub yield_percent: Option<f64>,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Stackup> for StackupSummary {
    fn from(s: &Stackup) -> Self {
        let (result, cpk, yield_percent) = if let Some(ref wc) = s.analysis_results.worst_case {
            let cpk = s.analysis_results.rss.as_ref().map(|r| r.cpk);
            let yield_pct = s.analysis_results.rss.as_ref().map(|r| r.yield_percent);
            (Some(wc.result.to_string()), cpk, yield_pct)
        } else {
            (None, None, None)
        };

        Self {
            id: s.id.to_string(),
            title: s.title.clone(),
            target_name: s.target.name.clone(),
            critical: s.target.critical,
            result,
            disposition: s.disposition.to_string(),
            cpk,
            yield_percent,
            status: format!("{:?}", s.status).to_lowercase(),
            author: s.author.clone(),
            created: s.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListStackupsResult {
    pub items: Vec<StackupSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

// ============================================================================
// List Params & Input DTOs
// ============================================================================

/// Parameters for listing stackups
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListStackupsParams {
    /// Filter by status
    pub status: Option<Vec<String>>,
    /// Filter by disposition
    pub disposition: Option<String>,
    /// Filter by worst-case result
    pub result: Option<String>,
    /// Show only critical stackups
    pub critical_only: Option<bool>,
    /// Show recent stackups (last N days)
    pub recent_days: Option<u32>,
    /// Search in title
    pub search: Option<String>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort descending
    pub sort_desc: Option<bool>,
    /// Limit number of results
    pub limit: Option<usize>,
}

/// Input for creating a stackup
#[derive(Debug, Clone, Deserialize)]
pub struct CreateStackupInput {
    pub title: String,
    pub target_name: String,
    pub target_nominal: f64,
    pub target_upper: f64,
    pub target_lower: f64,
    pub author: String,
    pub units: Option<String>,
    pub critical: Option<bool>,
    pub description: Option<String>,
    pub sigma_level: Option<f64>,
    pub mean_shift_k: Option<f64>,
    pub include_gdt: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// Input for updating a stackup
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateStackupInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub target_name: Option<String>,
    pub target_nominal: Option<f64>,
    pub target_upper: Option<f64>,
    pub target_lower: Option<f64>,
    pub critical: Option<bool>,
    pub sigma_level: Option<f64>,
    pub mean_shift_k: Option<f64>,
    pub include_gdt: Option<bool>,
    pub disposition: Option<String>,
    pub status: Option<String>,
}

/// Input for adding a contributor
#[derive(Debug, Clone, Deserialize)]
pub struct AddContributorInputDto {
    pub name: String,
    pub direction: String,
    pub nominal: f64,
    pub plus_tol: f64,
    pub minus_tol: f64,
    pub distribution: Option<String>,
    pub source: Option<String>,
    pub feature_id: Option<String>,
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

fn parse_disposition(s: &str) -> Option<Disposition> {
    match s.to_lowercase().as_str() {
        "under_review" | "underreview" => Some(Disposition::UnderReview),
        "approved" => Some(Disposition::Approved),
        "rejected" => Some(Disposition::Rejected),
        _ => None,
    }
}

fn parse_analysis_result(s: &str) -> Option<AnalysisResult> {
    match s.to_lowercase().as_str() {
        "pass" => Some(AnalysisResult::Pass),
        "marginal" => Some(AnalysisResult::Marginal),
        "fail" => Some(AnalysisResult::Fail),
        _ => None,
    }
}

fn parse_direction(s: &str) -> Option<Direction> {
    match s.to_lowercase().as_str() {
        "positive" | "+" => Some(Direction::Positive),
        "negative" | "-" => Some(Direction::Negative),
        _ => None,
    }
}

fn parse_distribution(s: &str) -> Option<Distribution> {
    match s.to_lowercase().as_str() {
        "normal" => Some(Distribution::Normal),
        "uniform" => Some(Distribution::Uniform),
        "triangular" => Some(Distribution::Triangular),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> StackupSortField {
    match s.to_lowercase().as_str() {
        "id" => StackupSortField::Id,
        "title" => StackupSortField::Title,
        "result" => StackupSortField::Result,
        "cpk" => StackupSortField::Cpk,
        "yield" => StackupSortField::Yield,
        "disposition" => StackupSortField::Disposition,
        "status" => StackupSortField::Status,
        "author" => StackupSortField::Author,
        _ => StackupSortField::Created,
    }
}

fn build_stackup_filter(params: &ListStackupsParams) -> StackupFilter {
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

    StackupFilter {
        common,
        disposition: params.disposition.as_ref().and_then(|d| parse_disposition(d)),
        result: params.result.as_ref().and_then(|r| parse_analysis_result(r)),
        critical_only: params.critical_only.unwrap_or(false),
        recent_days: params.recent_days,
        sort,
        sort_direction,
    }
}

// ============================================================================
// Commands
// ============================================================================

/// List stackups
#[tauri::command]
pub async fn list_stackups(
    params: Option<ListStackupsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListStackupsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = StackupService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = build_stackup_filter(&params);
    let stackups = service.list(&filter)?;
    let total_count = stackups.len();

    Ok(ListStackupsResult {
        items: stackups.iter().map(StackupSummary::from).collect(),
        total_count,
        has_more: false,
    })
}

/// Get a single stackup by ID
#[tauri::command]
pub async fn get_stackup(id: String, state: State<'_, AppState>) -> CommandResult<Option<Stackup>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = StackupService::new(project, cache);
    let stackup = service.get(&id)?;

    Ok(stackup)
}

/// Create a new stackup
#[tauri::command]
pub async fn create_stackup(
    input: CreateStackupInput,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);

        let create = CreateStackup {
            title: input.title,
            target_name: input.target_name,
            target_nominal: input.target_nominal,
            target_upper: input.target_upper,
            target_lower: input.target_lower,
            author: input.author,
            units: input.units,
            critical: input.critical.unwrap_or(false),
            description: input.description,
            sigma_level: input.sigma_level,
            mean_shift_k: input.mean_shift_k,
            include_gdt: input.include_gdt.unwrap_or(false),
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

    Ok(stackup)
}

/// Update an existing stackup
#[tauri::command]
pub async fn update_stackup(
    id: String,
    input: UpdateStackupInput,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);

        let update = UpdateStackup {
            title: input.title,
            description: input.description.map(Some),
            target_name: input.target_name,
            target_nominal: input.target_nominal,
            target_upper: input.target_upper,
            target_lower: input.target_lower,
            critical: input.critical,
            sigma_level: input.sigma_level,
            mean_shift_k: input.mean_shift_k,
            include_gdt: input.include_gdt,
            disposition: input.disposition.and_then(|d| parse_disposition(&d)),
            status: input.status.and_then(|s| parse_status(&s)),
        };

        service.update(&id, update)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Delete a stackup
#[tauri::command]
pub async fn delete_stackup(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);
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

/// Add a contributor to a stackup
#[tauri::command]
pub async fn add_stackup_contributor(
    id: String,
    input: AddContributorInputDto,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let direction = parse_direction(&input.direction).ok_or_else(|| {
        CommandError::InvalidInput(format!("Invalid direction: {}", input.direction))
    })?;

    let feature_id = if let Some(ref fid) = input.feature_id {
        Some(fid.parse::<EntityId>().map_err(|_| {
            CommandError::InvalidInput(format!("Invalid feature ID: {}", fid))
        })?)
    } else {
        None
    };

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);

        let contributor_input = AddContributorInput {
            name: input.name,
            direction,
            nominal: input.nominal,
            plus_tol: input.plus_tol,
            minus_tol: input.minus_tol,
            distribution: input.distribution.and_then(|d| parse_distribution(&d)),
            source: input.source,
            feature_id,
        };

        service.add_contributor(&id, contributor_input)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Remove a contributor from a stackup by index
#[tauri::command]
pub async fn remove_stackup_contributor(
    id: String,
    index: usize,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);
        service.remove_contributor(&id, index)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Remove a contributor by feature ID
#[tauri::command]
pub async fn remove_stackup_contributor_by_feature(
    id: String,
    feature_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);
        service.remove_contributor_by_feature(&id, &feature_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Run all analyses on a stackup
#[tauri::command]
pub async fn analyze_stackup(
    id: String,
    monte_carlo_iterations: Option<u32>,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);
        service.analyze(&id, monte_carlo_iterations)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Calculate worst-case analysis only
#[tauri::command]
pub async fn calculate_stackup_worst_case(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<WorstCaseResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = StackupService::new(project, cache);
    let result = service.calculate_worst_case(&id)?;

    Ok(result)
}

/// Calculate RSS analysis only
#[tauri::command]
pub async fn calculate_stackup_rss(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<RssResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = StackupService::new(project, cache);
    let result = service.calculate_rss(&id)?;

    Ok(result)
}

/// Calculate Monte Carlo analysis only
#[tauri::command]
pub async fn calculate_stackup_monte_carlo(
    id: String,
    iterations: Option<u32>,
    state: State<'_, AppState>,
) -> CommandResult<MonteCarloResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = StackupService::new(project, cache);
    let result = service.calculate_monte_carlo(&id, iterations.unwrap_or(10000))?;

    Ok(result)
}

/// Set disposition for a stackup
#[tauri::command]
pub async fn set_stackup_disposition(
    id: String,
    disposition: String,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let disp = parse_disposition(&disposition).ok_or_else(|| {
        CommandError::InvalidInput(format!("Invalid disposition: {}", disposition))
    })?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);
        service.set_disposition(&id, disp)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Add a verifies link to a requirement
#[tauri::command]
pub async fn add_stackup_verifies_link(
    id: String,
    requirement_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);
        service.add_verifies_link(&id, &requirement_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Remove a verifies link
#[tauri::command]
pub async fn remove_stackup_verifies_link(
    id: String,
    requirement_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Stackup> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let stackup = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = StackupService::new(project, cache);
        service.remove_verifies_link(&id, &requirement_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(stackup)
}

/// Get stackup statistics
#[tauri::command]
pub async fn get_stackup_stats(state: State<'_, AppState>) -> CommandResult<StackupStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = StackupService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}
