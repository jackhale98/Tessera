//! Requirement-specific commands
//!
//! Provides commands for managing requirements with filtering and stats.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;

use tdt_core::core::entity::{Priority, Status};
use tdt_core::core::identity::EntityPrefix;
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
            if statuses.is_empty() {
                None
            } else {
                Some(statuses)
            }
        }),
        priority: params
            .priority
            .as_ref()
            .and_then(|p| parse_priority(p))
            .map(|p| vec![p]),
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

    let requirements = service.list(&filter, sort, sort_direction)?;

    Ok(ListRequirementsResult {
        total_count: requirements.items.len(),
        items: requirements
            .items
            .iter()
            .map(RequirementSummary::from)
            .collect(),
    })
}

#[tauri::command]
pub async fn get_requirement(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<Requirement>> {
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

// ============================================================================
// Verification Matrix Types
// ============================================================================

/// A linked entity summary for the verification matrix
#[derive(Debug, Clone, Serialize)]
pub struct LinkedEntitySummary {
    pub id: String,
    pub title: String,
    pub status: String,
}

/// A test result summary
#[derive(Debug, Clone, Serialize)]
pub struct TestResultSummary {
    pub id: String,
    pub verdict: String,
    pub executed_date: Option<String>,
    pub executor: Option<String>,
}

/// A test with its results for the verification matrix
#[derive(Debug, Clone, Serialize)]
pub struct TestWithResults {
    pub id: String,
    pub title: String,
    pub status: String,
    pub test_type: String,
    pub level: String,
    pub results: Vec<TestResultSummary>,
    /// Best result verdict (Pass > Fail > No Results)
    pub latest_verdict: Option<String>,
}

/// A single row in the verification matrix
#[derive(Debug, Clone, Serialize)]
pub struct VerificationMatrixRow {
    pub requirement: LinkedEntitySummary,
    pub req_type: String,
    pub level: String,
    pub priority: String,
    /// Output requirements that satisfy this input requirement
    pub derived_requirements: Vec<LinkedEntitySummary>,
    /// Tests that verify this requirement (directly or via derived requirements)
    pub tests: Vec<TestWithResults>,
    /// Overall verification status
    pub verification_status: String,
    /// Count of tests with passing results
    pub pass_count: usize,
    /// Count of tests with failing results
    pub fail_count: usize,
    /// Count of tests without results
    pub not_run_count: usize,
}

/// The full verification matrix response
#[derive(Debug, Clone, Serialize)]
pub struct VerificationMatrixResponse {
    pub rows: Vec<VerificationMatrixRow>,
    pub summary: VerificationMatrixSummary,
}

/// Summary statistics for the verification matrix
#[derive(Debug, Clone, Serialize)]
pub struct VerificationMatrixSummary {
    pub total_requirements: usize,
    pub fully_verified: usize,
    pub partially_verified: usize,
    pub not_tested: usize,
    pub failed: usize,
    pub verification_coverage: f64,
}

/// Get the full requirement verification matrix
#[tauri::command]
pub async fn get_verification_matrix(
    state: State<'_, AppState>,
) -> CommandResult<VerificationMatrixResponse> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    // Get all requirements
    let service = RequirementService::new(project, cache);
    let all_requirements = service.list(
        &RequirementFilter::default(),
        RequirementSortField::Created,
        SortDirection::Ascending,
    )?;

    // Build caches for quick lookup
    let mut req_map: HashMap<String, &Requirement> = HashMap::new();
    for req in &all_requirements.items {
        req_map.insert(req.id.to_string(), req);
    }

    // Get all test and result info from cache
    let tests_filter = tdt_core::core::cache::EntityFilter {
        prefix: Some(EntityPrefix::Test),
        ..Default::default()
    };
    let results_filter = tdt_core::core::cache::EntityFilter {
        prefix: Some(EntityPrefix::Rslt),
        ..Default::default()
    };

    let cached_tests = cache.list_entities(&tests_filter);
    let cached_results = cache.list_entities(&results_filter);

    // Build test info map
    let test_info: HashMap<String, (String, String, String)> = cached_tests
        .iter()
        .map(|t| {
            (
                t.id.clone(),
                (
                    t.title.clone(),
                    format!("{:?}", t.status).to_lowercase(),
                    String::new(), // We'll get test_type from the test entity if needed
                ),
            )
        })
        .collect();

    // Build result -> test mapping and result info
    let mut results_by_test: HashMap<String, Vec<TestResultSummary>> = HashMap::new();
    for result in &cached_results {
        // Get the test_id from links (verified_by or test link)
        let links = cache.get_links_to(&result.id);
        for link in &links {
            // Find which test this result belongs to
            if link.source_id.starts_with("TEST-") {
                results_by_test
                    .entry(link.source_id.clone())
                    .or_default()
                    .push(TestResultSummary {
                        id: result.id.clone(),
                        verdict: format!("{:?}", result.status).to_lowercase(),
                        executed_date: Some(result.created.to_rfc3339()),
                        executor: None,
                    });
            }
        }
    }

    // Also check links from test to result
    for test in &cached_tests {
        let links_from = cache.get_links_from(&test.id);
        for link in &links_from {
            if link.target_id.starts_with("RSLT-") {
                // Find the result
                if let Some(result) = cached_results.iter().find(|r| r.id == link.target_id) {
                    let entry = results_by_test.entry(test.id.clone()).or_default();
                    // Avoid duplicates
                    if !entry.iter().any(|r| r.id == result.id) {
                        entry.push(TestResultSummary {
                            id: result.id.clone(),
                            verdict: format!("{:?}", result.status).to_lowercase(),
                            executed_date: Some(result.created.to_rfc3339()),
                            executor: None,
                        });
                    }
                }
            }
        }
    }

    // Build the verification matrix rows
    let mut rows = Vec::new();
    let mut summary = VerificationMatrixSummary {
        total_requirements: 0,
        fully_verified: 0,
        partially_verified: 0,
        not_tested: 0,
        failed: 0,
        verification_coverage: 0.0,
    };

    // Process input requirements first (they may have derived output requirements)
    let mut processed_reqs: HashSet<String> = HashSet::new();

    for req in &all_requirements.items {
        let req_id = req.id.to_string();
        if processed_reqs.contains(&req_id) {
            continue;
        }
        processed_reqs.insert(req_id.clone());

        // Get links from this requirement
        let links_from = cache.get_links_from(&req_id);

        // Find derived/output requirements (satisfied_by links)
        let mut derived_requirements: Vec<LinkedEntitySummary> = Vec::new();
        let mut all_test_ids: Vec<String> = Vec::new();

        // Collect tests that directly verify this requirement
        for link in &links_from {
            if link.link_type == "verified_by" && link.target_id.starts_with("TEST-") {
                all_test_ids.push(link.target_id.clone());
            }
            if link.link_type == "satisfied_by" && link.target_id.starts_with("REQ-") {
                if let Some(derived) = cache.get_entity(&link.target_id) {
                    derived_requirements.push(LinkedEntitySummary {
                        id: derived.id.clone(),
                        title: derived.title.clone(),
                        status: format!("{:?}", derived.status).to_lowercase(),
                    });
                    // Also collect tests that verify the derived requirement
                    let derived_links = cache.get_links_from(&derived.id);
                    for dl in &derived_links {
                        if dl.link_type == "verified_by" && dl.target_id.starts_with("TEST-") {
                            if !all_test_ids.contains(&dl.target_id) {
                                all_test_ids.push(dl.target_id.clone());
                            }
                        }
                    }
                    // Mark as processed
                    processed_reqs.insert(derived.id.clone());
                }
            }
        }

        // Also check incoming links (tests that verify this requirement)
        let links_to = cache.get_links_to(&req_id);
        for link in &links_to {
            if link.link_type == "verifies" && link.source_id.starts_with("TEST-") {
                if !all_test_ids.contains(&link.source_id) {
                    all_test_ids.push(link.source_id.clone());
                }
            }
        }

        // Build test info with results
        let mut tests: Vec<TestWithResults> = Vec::new();
        let mut pass_count = 0;
        let mut fail_count = 0;
        let mut not_run_count = 0;

        for test_id in &all_test_ids {
            let (title, status, _) = test_info
                .get(test_id)
                .cloned()
                .unwrap_or_else(|| (test_id.clone(), "unknown".to_string(), String::new()));

            let results = results_by_test.get(test_id).cloned().unwrap_or_default();

            // Determine latest verdict
            let latest_verdict = if results.is_empty() {
                not_run_count += 1;
                None
            } else {
                // Find the most recent result and check if any passed
                let has_pass = results
                    .iter()
                    .any(|r| r.verdict == "pass" || r.verdict == "approved");
                let has_fail = results
                    .iter()
                    .any(|r| r.verdict == "fail" || r.verdict == "rejected");
                if has_pass && !has_fail {
                    pass_count += 1;
                    Some("pass".to_string())
                } else if has_fail {
                    fail_count += 1;
                    Some("fail".to_string())
                } else {
                    // Partial or inconclusive
                    pass_count += 1;
                    Some("partial".to_string())
                }
            };

            tests.push(TestWithResults {
                id: test_id.clone(),
                title,
                status,
                test_type: "verification".to_string(),
                level: "system".to_string(),
                results,
                latest_verdict,
            });
        }

        // Determine overall verification status
        let verification_status = if tests.is_empty() {
            summary.not_tested += 1;
            "not_tested"
        } else if fail_count > 0 {
            summary.failed += 1;
            "failed"
        } else if not_run_count > 0 {
            summary.partially_verified += 1;
            "partial"
        } else {
            summary.fully_verified += 1;
            "verified"
        };

        rows.push(VerificationMatrixRow {
            requirement: LinkedEntitySummary {
                id: req_id,
                title: req.title.clone(),
                status: format!("{:?}", req.status).to_lowercase(),
            },
            req_type: format!("{:?}", req.req_type).to_lowercase(),
            level: format!("{:?}", req.level).to_lowercase(),
            priority: format!("{:?}", req.priority).to_lowercase(),
            derived_requirements,
            tests,
            verification_status: verification_status.to_string(),
            pass_count,
            fail_count,
            not_run_count,
        });

        summary.total_requirements += 1;
    }

    // Calculate coverage percentage
    if summary.total_requirements > 0 {
        summary.verification_coverage =
            (summary.fully_verified as f64 / summary.total_requirements as f64) * 100.0;
    }

    Ok(VerificationMatrixResponse { rows, summary })
}
