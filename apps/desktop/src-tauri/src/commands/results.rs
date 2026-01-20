//! Result (test execution record) Tauri commands
//!
//! Provides commands for managing test execution results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::cache::CachedResult;
use tdt_core::core::entity::Status;
use tdt_core::core::identity::EntityId;
use tdt_core::entities::result::{
    AttachmentType, Measurement, Result as TestResult, ResultEnvironment, SampleInfo,
    StepResult as StepResultEnum, Verdict,
};
use tdt_core::services::common::{CommonFilter, SortDirection};
use tdt_core::services::result::{
    CreateResult, ResultFilter, ResultService, ResultSortField, ResultStats, UpdateResult,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Result summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct ResultSummary {
    pub id: String,
    pub title: String,
    pub test_id: Option<String>,
    pub verdict: Option<String>,
    pub status: String,
    pub author: String,
    pub executed_by: Option<String>,
    pub executed_date: Option<String>,
    pub created: String,
}

impl From<&CachedResult> for ResultSummary {
    fn from(cached: &CachedResult) -> Self {
        Self {
            id: cached.id.clone(),
            title: cached.title.clone(),
            test_id: cached.test_id.clone(),
            verdict: cached.verdict.clone(),
            status: format!("{:?}", cached.status).to_lowercase(),
            author: cached.author.clone(),
            executed_by: cached.executed_by.clone(),
            executed_date: cached.executed_date.clone(),
            created: cached.created.to_rfc3339(),
        }
    }
}

impl From<&TestResult> for ResultSummary {
    fn from(result: &TestResult) -> Self {
        Self {
            id: result.id.to_string(),
            title: result
                .title
                .clone()
                .unwrap_or_else(|| "Untitled Result".to_string()),
            test_id: Some(result.test_id.to_string()),
            verdict: Some(result.verdict.to_string()),
            status: format!("{:?}", result.status).to_lowercase(),
            author: result.author.clone(),
            executed_by: Some(result.executed_by.clone()),
            executed_date: Some(result.executed_date.to_rfc3339()),
            created: result.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListResultsResult {
    pub items: Vec<ResultSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

// ============================================================================
// List Params & Input DTOs
// ============================================================================

/// Parameters for listing results
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListResultsParams {
    /// Filter by status
    pub status: Option<String>,
    /// Filter by verdict
    pub verdict: Option<String>,
    /// Filter by test ID
    pub test_id: Option<String>,
    /// Filter by category
    pub category: Option<String>,
    /// Filter by executor (substring match)
    pub executed_by: Option<String>,
    /// Show only results with failures
    pub with_failures: Option<bool>,
    /// Show only results with deviations
    pub with_deviations: Option<bool>,
    /// Show results from last N days
    pub recent_days: Option<u32>,
    /// Filter by author
    pub author: Option<String>,
    /// Search in title/notes
    pub search: Option<String>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Sort field
    pub sort: Option<String>,
    /// Sort direction
    pub sort_direction: Option<String>,
    /// Limit number of results
    pub limit: Option<usize>,
}

/// Input for creating a result
#[derive(Debug, Clone, Deserialize)]
pub struct CreateResultInput {
    /// Test ID that was executed
    pub test_id: String,
    /// Test revision
    pub test_revision: Option<u32>,
    /// Optional title
    pub title: Option<String>,
    /// Overall verdict
    pub verdict: String,
    /// Verdict rationale
    pub verdict_rationale: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Who executed the test
    pub executed_by: String,
    /// When test was executed
    pub executed_date: Option<DateTime<Utc>>,
    /// Sample info
    pub sample_info: Option<SampleInfoInput>,
    /// Environment during test
    pub environment: Option<EnvironmentInput>,
    /// Duration
    pub duration: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Initial status
    pub status: Option<String>,
    /// Author
    pub author: String,
}

/// Input for updating a result
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateResultInput {
    /// Update title
    pub title: Option<String>,
    /// Update verdict
    pub verdict: Option<String>,
    /// Update verdict rationale
    pub verdict_rationale: Option<String>,
    /// Update category
    pub category: Option<String>,
    /// Update tags
    pub tags: Option<Vec<String>>,
    /// Update reviewed_by
    pub reviewed_by: Option<String>,
    /// Update reviewed_date
    pub reviewed_date: Option<DateTime<Utc>>,
    /// Update sample info
    pub sample_info: Option<SampleInfoInput>,
    /// Update environment
    pub environment: Option<EnvironmentInput>,
    /// Update duration
    pub duration: Option<String>,
    /// Update notes
    pub notes: Option<String>,
    /// Update status
    pub status: Option<String>,
}

/// Sample info input
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SampleInfoInput {
    pub sample_id: Option<String>,
    pub serial_number: Option<String>,
    pub lot_number: Option<String>,
    pub configuration: Option<String>,
}

/// Environment input
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EnvironmentInput {
    pub temperature: Option<String>,
    pub humidity: Option<String>,
    pub location: Option<String>,
    pub other: Option<String>,
}

/// Equipment used input
#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentUsedInput {
    pub name: String,
    pub asset_id: Option<String>,
    pub calibration_date: Option<String>,
    pub calibration_due: Option<String>,
}

/// Step result input
#[derive(Debug, Clone, Deserialize)]
pub struct AddStepResultInput {
    pub step: u32,
    pub result: String,
    pub observed: Option<String>,
    pub measurement: Option<MeasurementInput>,
    pub notes: Option<String>,
}

/// Measurement input
#[derive(Debug, Clone, Deserialize)]
pub struct MeasurementInput {
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// Failure input
#[derive(Debug, Clone, Deserialize)]
pub struct RecordFailureInput {
    pub description: String,
    pub step: Option<u32>,
    pub root_cause: Option<String>,
    pub corrective_action: Option<String>,
}

/// Deviation input
#[derive(Debug, Clone, Deserialize)]
pub struct RecordDeviationInput {
    pub description: String,
    pub impact: Option<String>,
    pub justification: Option<String>,
}

/// Attachment input
#[derive(Debug, Clone, Deserialize)]
pub struct AddAttachmentInput {
    pub filename: String,
    pub path: Option<String>,
    pub attachment_type: Option<String>,
    pub description: Option<String>,
}

// ============================================================================
// Conversion helpers
// ============================================================================

fn parse_verdict(s: &str) -> Option<Verdict> {
    match s.to_lowercase().as_str() {
        "pass" => Some(Verdict::Pass),
        "fail" => Some(Verdict::Fail),
        "conditional" => Some(Verdict::Conditional),
        "incomplete" => Some(Verdict::Incomplete),
        "not_applicable" | "na" => Some(Verdict::NotApplicable),
        _ => None,
    }
}

fn parse_step_result(s: &str) -> Option<StepResultEnum> {
    match s.to_lowercase().as_str() {
        "pass" => Some(StepResultEnum::Pass),
        "fail" => Some(StepResultEnum::Fail),
        "skip" => Some(StepResultEnum::Skip),
        "not_applicable" | "na" => Some(StepResultEnum::NotApplicable),
        _ => None,
    }
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

fn parse_sort_field(s: &str) -> ResultSortField {
    match s.to_lowercase().as_str() {
        "id" => ResultSortField::Id,
        "title" => ResultSortField::Title,
        "test" => ResultSortField::Test,
        "verdict" => ResultSortField::Verdict,
        "status" => ResultSortField::Status,
        "author" => ResultSortField::Author,
        "executed_date" | "executeddate" | "executed" => ResultSortField::ExecutedDate,
        _ => ResultSortField::Created,
    }
}

fn parse_sort_direction(s: &str) -> SortDirection {
    match s.to_lowercase().as_str() {
        "desc" | "descending" => SortDirection::Descending,
        _ => SortDirection::Ascending,
    }
}

fn parse_attachment_type(s: &str) -> Option<AttachmentType> {
    match s.to_lowercase().as_str() {
        "data" => Some(AttachmentType::Data),
        "photo" => Some(AttachmentType::Photo),
        "screenshot" => Some(AttachmentType::Screenshot),
        "log" => Some(AttachmentType::Log),
        "report" => Some(AttachmentType::Report),
        "other" => Some(AttachmentType::Other),
        _ => None,
    }
}

fn build_result_filter(params: &ListResultsParams) -> ResultFilter {
    let mut common = CommonFilter::default();

    if let Some(ref status) = params.status {
        if let Some(s) = parse_status(status) {
            common.status = Some(vec![s]);
        }
    }
    if let Some(ref author) = params.author {
        common.author = Some(author.clone());
    }
    if let Some(ref search) = params.search {
        common.search = Some(search.clone());
    }
    if let Some(ref tags) = params.tags {
        common.tags = Some(tags.clone());
    }
    if let Some(limit) = params.limit {
        common.limit = Some(limit);
    }

    let sort = params
        .sort
        .as_ref()
        .map(|s| parse_sort_field(s))
        .unwrap_or_default();

    let sort_direction = params
        .sort_direction
        .as_ref()
        .map(|s| parse_sort_direction(s))
        .unwrap_or(SortDirection::Descending);

    ResultFilter {
        common,
        verdict: params.verdict.as_ref().and_then(|v| parse_verdict(v)),
        test_id: params.test_id.clone(),
        category: params.category.clone(),
        executed_by: params.executed_by.clone(),
        with_failures: params.with_failures.unwrap_or(false),
        with_deviations: params.with_deviations.unwrap_or(false),
        recent_days: params.recent_days,
        sort,
        sort_direction,
    }
}

fn sample_info_from_input(input: SampleInfoInput) -> SampleInfo {
    SampleInfo {
        sample_id: input.sample_id,
        serial_number: input.serial_number,
        lot_number: input.lot_number,
        configuration: input.configuration,
    }
}

fn environment_from_input(input: EnvironmentInput) -> ResultEnvironment {
    ResultEnvironment {
        temperature: input.temperature,
        humidity: input.humidity,
        location: input.location,
        other: input.other,
    }
}

fn measurement_from_input(input: MeasurementInput) -> Measurement {
    Measurement {
        value: input.value,
        unit: input.unit,
        min: input.min,
        max: input.max,
    }
}

// ============================================================================
// Commands
// ============================================================================

/// List results
#[tauri::command]
pub async fn list_results(
    params: ListResultsParams,
    state: State<'_, AppState>,
) -> CommandResult<ListResultsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let filter = build_result_filter(&params);
    let results = service.list_cached(&filter)?;
    let total_count = results.len();

    Ok(ListResultsResult {
        items: results.iter().map(ResultSummary::from).collect(),
        total_count,
        has_more: false,
    })
}

/// List results (full entity load for JSON/YAML output)
#[tauri::command]
pub async fn list_results_full(
    params: ListResultsParams,
    state: State<'_, AppState>,
) -> CommandResult<Vec<TestResult>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let filter = build_result_filter(&params);
    let results = service.list(&filter)?;

    Ok(results)
}

/// Get a result by ID
#[tauri::command]
pub async fn get_result(id: String, state: State<'_, AppState>) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.get_required(&id)?;

    Ok(result)
}

/// Create a new result
#[tauri::command]
pub async fn create_result(
    input: CreateResultInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let test_id: EntityId = input
        .test_id
        .parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid test ID: {}", input.test_id)))?;

    let verdict = parse_verdict(&input.verdict)
        .ok_or_else(|| CommandError::InvalidInput(format!("Invalid verdict: {}", input.verdict)))?;

    let create_input = CreateResult {
        test_id,
        test_revision: input.test_revision,
        title: input.title,
        verdict,
        verdict_rationale: input.verdict_rationale,
        category: input.category,
        tags: input.tags.unwrap_or_default(),
        executed_by: input.executed_by,
        executed_date: input.executed_date,
        sample_info: input.sample_info.map(sample_info_from_input),
        environment: input.environment.map(environment_from_input),
        duration: input.duration,
        notes: input.notes,
        status: input.status.and_then(|s| parse_status(&s)),
        author: input.author,
    };

    let service = ResultService::new(project, cache);
    let result = service.create(create_input)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Update an existing result
#[tauri::command]
pub async fn update_result(
    id: String,
    input: UpdateResultInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let update_input = UpdateResult {
        title: input.title.map(Some),
        verdict: input.verdict.and_then(|v| parse_verdict(&v)),
        verdict_rationale: input.verdict_rationale.map(Some),
        category: input.category.map(Some),
        tags: input.tags,
        reviewed_by: input.reviewed_by.map(Some),
        reviewed_date: input.reviewed_date.map(Some),
        sample_info: input.sample_info.map(|s| Some(sample_info_from_input(s))),
        environment: input.environment.map(|e| Some(environment_from_input(e))),
        duration: input.duration.map(Some),
        notes: input.notes.map(Some),
        status: input.status.and_then(|s| parse_status(&s)),
    };

    let service = ResultService::new(project, cache);
    let result = service.update(&id, update_input)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Delete a result
#[tauri::command]
pub async fn delete_result(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    service.delete(&id, force.unwrap_or(false))?;

    // Sync cache
    let _ = cache.sync();

    Ok(())
}

/// Add a step result
#[tauri::command]
pub async fn add_result_step(
    id: String,
    input: AddStepResultInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let step_result = parse_step_result(&input.result).ok_or_else(|| {
        CommandError::InvalidInput(format!("Invalid step result: {}", input.result))
    })?;

    let measurement = input.measurement.map(measurement_from_input);

    let service = ResultService::new(project, cache);
    let result = service.add_step_result(
        &id,
        input.step,
        step_result,
        input.observed,
        measurement,
        input.notes,
    )?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Update a step result
#[tauri::command]
pub async fn update_result_step(
    id: String,
    step: u32,
    result: Option<String>,
    observed: Option<String>,
    measurement: Option<MeasurementInput>,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let step_result = result.as_ref().and_then(|r| parse_step_result(r));
    let measurement_opt = measurement.map(|m| Some(measurement_from_input(m)));

    let service = ResultService::new(project, cache);
    let test_result = service.update_step_result(
        &id,
        step,
        step_result,
        observed.map(Some),
        measurement_opt,
        notes.map(Some),
    )?;

    // Sync cache
    let _ = cache.sync();

    Ok(test_result)
}

/// Remove a step result
#[tauri::command]
pub async fn remove_result_step(
    id: String,
    step: u32,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.remove_step_result(&id, step)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Record a failure
#[tauri::command]
pub async fn record_result_failure(
    id: String,
    input: RecordFailureInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.record_failure(
        &id,
        input.description,
        input.step,
        input.root_cause,
        input.corrective_action,
    )?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Remove a failure by index
#[tauri::command]
pub async fn remove_result_failure(
    id: String,
    index: usize,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.remove_failure(&id, index)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Record a deviation
#[tauri::command]
pub async fn record_result_deviation(
    id: String,
    input: RecordDeviationInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result =
        service.record_deviation(&id, input.description, input.impact, input.justification)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Remove a deviation by index
#[tauri::command]
pub async fn remove_result_deviation(
    id: String,
    index: usize,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.remove_deviation(&id, index)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Add an attachment
#[tauri::command]
pub async fn add_result_attachment(
    id: String,
    input: AddAttachmentInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let attachment_type = input
        .attachment_type
        .as_ref()
        .and_then(|t| parse_attachment_type(t));

    let service = ResultService::new(project, cache);
    let result = service.add_attachment(
        &id,
        input.filename,
        input.path,
        attachment_type,
        input.description,
    )?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Remove an attachment by filename
#[tauri::command]
pub async fn remove_result_attachment(
    id: String,
    filename: String,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.remove_attachment(&id, &filename)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Add equipment used
#[tauri::command]
pub async fn add_result_equipment(
    id: String,
    input: EquipmentUsedInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.add_equipment(
        &id,
        input.name,
        input.asset_id,
        input.calibration_date,
        input.calibration_due,
    )?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Remove equipment used by name
#[tauri::command]
pub async fn remove_result_equipment(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.remove_equipment(&id, &name)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Set sample info
#[tauri::command]
pub async fn set_result_sample_info(
    id: String,
    sample_info: SampleInfoInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.set_sample_info(&id, sample_info_from_input(sample_info))?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Set environment
#[tauri::command]
pub async fn set_result_environment(
    id: String,
    environment: EnvironmentInput,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.set_environment(&id, environment_from_input(environment))?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Mark result as reviewed
#[tauri::command]
pub async fn mark_result_reviewed(
    id: String,
    reviewer: String,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let mut cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_mut()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.mark_reviewed(&id, reviewer)?;

    // Sync cache
    let _ = cache.sync();

    Ok(result)
}

/// Get results for a specific test
#[tauri::command]
pub async fn get_results_by_test(
    test_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<TestResult>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let results = service.get_results_for_test(&test_id)?;

    Ok(results)
}

/// Get the latest result for a test
#[tauri::command]
pub async fn get_latest_result_for_test(
    test_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<TestResult>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let result = service.get_latest_result_for_test(&test_id)?;

    Ok(result)
}

/// Get result statistics
#[tauri::command]
pub async fn get_result_stats(state: State<'_, AppState>) -> CommandResult<ResultStats> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard
        .as_ref()
        .ok_or_else(|| CommandError::NoProject)?;

    let service = ResultService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}
