//! Test entity commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::cache::CachedTest;
use tdt_core::core::entity::Status;
use tdt_core::core::identity::EntityId;
use tdt_core::entities::result::{Result as TestResult, Verdict};
use tdt_core::entities::test::{
    Environment, Equipment, ProcedureStep, SampleSize, Test, TestLevel, TestMethod, TestType,
};
use tdt_core::services::{CreateTest, RunTestInput, TestFilter, TestService, TestStats, UpdateTest};

/// List parameters for Tests
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListTestsParams {
    pub status: Option<Vec<String>>,
    pub test_type: Option<String>,
    pub level: Option<String>,
    pub method: Option<String>,
    pub orphans_only: Option<bool>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// Test summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct TestSummary {
    pub id: String,
    pub title: String,
    pub test_type: String,
    pub level: String,
    pub method: String,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Test> for TestSummary {
    fn from(test: &Test) -> Self {
        Self {
            id: test.id.to_string(),
            title: test.title.clone(),
            test_type: test.test_type.to_string(),
            level: test
                .test_level
                .as_ref()
                .map(|l| l.to_string())
                .unwrap_or_default(),
            method: test
                .test_method
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_default(),
            status: format!("{:?}", test.status).to_lowercase(),
            author: test.author.clone(),
            created: test.created.to_rfc3339(),
        }
    }
}

impl From<&CachedTest> for TestSummary {
    fn from(cached: &CachedTest) -> Self {
        Self {
            id: cached.id.clone(),
            title: cached.title.clone(),
            test_type: cached.test_type.clone().unwrap_or_default(),
            level: cached.level.clone().unwrap_or_default(),
            method: cached.method.clone().unwrap_or_default(),
            status: format!("{:?}", cached.status).to_lowercase(),
            author: cached.author.clone(),
            created: cached.created.to_rfc3339(),
        }
    }
}

/// List result with pagination info
#[derive(Debug, Clone, Serialize)]
pub struct ListTestsResult {
    pub items: Vec<TestSummary>,
    pub total_count: usize,
    pub has_more: bool,
}

/// Input for creating a Test
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTestInput {
    pub title: String,
    pub test_type: Option<String>,
    pub level: Option<String>,
    pub method: Option<String>,
    pub description: Option<String>,
    pub objective: Option<String>,
    pub author: String,
}

/// Input for updating a Test
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTestInput {
    pub title: Option<String>,
    pub test_type: Option<String>,
    pub level: Option<String>,
    pub method: Option<String>,
    pub description: Option<String>,
    pub objective: Option<String>,
    pub status: Option<String>,
}

/// Input for running a test
#[derive(Debug, Clone, Deserialize)]
pub struct RunTestInputDto {
    pub verdict: String,
    pub executed_by: String,
    pub notes: Option<String>,
    pub verdict_rationale: Option<String>,
}

/// Input for adding a procedure step
#[derive(Debug, Clone, Deserialize)]
pub struct TestAddStepInput {
    pub step: u32,
    pub action: String,
    pub expected: Option<String>,
    pub acceptance: Option<String>,
}

/// Input for adding equipment
#[derive(Debug, Clone, Deserialize)]
pub struct AddEquipmentInput {
    pub name: String,
    pub specification: Option<String>,
    pub calibration_required: Option<bool>,
}

/// Input for sample size
#[derive(Debug, Clone, Deserialize)]
pub struct SetSampleSizeInput {
    pub quantity: Option<u32>,
    pub rationale: Option<String>,
    pub sampling_method: Option<String>,
}

/// Input for environment
#[derive(Debug, Clone, Deserialize)]
pub struct SetEnvironmentInput {
    pub temperature: Option<String>,
    pub humidity: Option<String>,
    pub other: Option<String>,
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

fn parse_test_type(s: &str) -> Option<TestType> {
    match s.to_lowercase().as_str() {
        "verification" | "ver" => Some(TestType::Verification),
        "validation" | "val" => Some(TestType::Validation),
        _ => None,
    }
}

fn parse_test_level(s: &str) -> Option<TestLevel> {
    match s.to_lowercase().as_str() {
        "unit" => Some(TestLevel::Unit),
        "integration" | "int" => Some(TestLevel::Integration),
        "system" | "sys" => Some(TestLevel::System),
        "acceptance" | "acc" => Some(TestLevel::Acceptance),
        _ => None,
    }
}

fn parse_test_method(s: &str) -> Option<TestMethod> {
    match s.to_lowercase().as_str() {
        "inspection" | "i" => Some(TestMethod::Inspection),
        "analysis" | "a" => Some(TestMethod::Analysis),
        "demonstration" | "d" => Some(TestMethod::Demonstration),
        "test" | "t" => Some(TestMethod::Test),
        _ => None,
    }
}

fn parse_verdict(s: &str) -> Option<Verdict> {
    match s.to_lowercase().as_str() {
        "pass" | "passed" => Some(Verdict::Pass),
        "fail" | "failed" => Some(Verdict::Fail),
        "conditional" | "cond" => Some(Verdict::Conditional),
        "incomplete" | "inc" => Some(Verdict::Incomplete),
        "not_applicable" | "na" | "n/a" => Some(Verdict::NotApplicable),
        _ => None,
    }
}

/// List Tests with optional filters
#[tauri::command]
pub async fn list_tests(
    params: Option<ListTestsParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListTestsResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TestService::new(project, cache);
    let params = params.unwrap_or_default();

    let filter = TestFilter {
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
        test_type: params.test_type.and_then(|s| parse_test_type(&s)),
        test_level: params.level.and_then(|s| parse_test_level(&s)),
        test_method: params.method.and_then(|s| parse_test_method(&s)),
        orphans_only: params.orphans_only.unwrap_or(false),
        ..Default::default()
    };

    // Use cache for fast list views
    let result = service.list_cached(&filter)?;
    let total_count = result.items.len();

    Ok(ListTestsResult {
        items: result.items.iter().map(TestSummary::from).collect(),
        total_count,
        has_more: false,
    })
}

/// Get a single Test by ID
#[tauri::command]
pub async fn get_test(id: String, state: State<'_, AppState>) -> CommandResult<Option<Test>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TestService::new(project, cache);
    let test = service.get(&id)?;

    Ok(test)
}

/// Create a new Test
#[tauri::command]
pub async fn create_test(
    input: CreateTestInput,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let create = CreateTest {
            title: input.title,
            test_type: input
                .test_type
                .and_then(|s| parse_test_type(&s))
                .unwrap_or(TestType::Verification),
            test_level: input.level.and_then(|s| parse_test_level(&s)),
            test_method: input.method.and_then(|s| parse_test_method(&s)),
            description: input.description,
            objective: input.objective.unwrap_or_default(),
            tags: Vec::new(),
            author: input.author,
            ..Default::default()
        };

        service.create(create)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Update an existing Test
#[tauri::command]
pub async fn update_test(
    id: String,
    input: UpdateTestInput,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let update = UpdateTest {
            title: input.title,
            test_type: input.test_type.and_then(|s| parse_test_type(&s)),
            test_level: input.level.and_then(|s| parse_test_level(&s)),
            test_method: input.method.and_then(|s| parse_test_method(&s)),
            description: input.description,
            objective: input.objective,
            status: input.status.and_then(|s| parse_status(&s)),
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

    Ok(test)
}

/// Delete a Test
#[tauri::command]
pub async fn delete_test(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);
        service.delete(&id, false)?;
    }

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Run a test and create a result
#[tauri::command]
pub async fn run_test(
    id: String,
    input: RunTestInputDto,
    state: State<'_, AppState>,
) -> CommandResult<TestResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let result = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let verdict = parse_verdict(&input.verdict).ok_or_else(|| {
            CommandError::InvalidInput(format!("Invalid verdict: {}", input.verdict))
        })?;

        let run_input = RunTestInput {
            verdict,
            executed_by: input.executed_by,
            notes: input.notes,
            verdict_rationale: input.verdict_rationale,
        };

        service.run(&id, run_input)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(result)
}

/// Add a procedure step
#[tauri::command]
pub async fn add_test_step(
    id: String,
    input: TestAddStepInput,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let step = ProcedureStep {
            step: input.step,
            action: input.action,
            expected: input.expected,
            acceptance: input.acceptance,
        };

        service.add_step(&id, step)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Remove a procedure step
#[tauri::command]
pub async fn remove_test_step(
    id: String,
    step_number: u32,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);
        service.remove_step(&id, step_number)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Add equipment
#[tauri::command]
pub async fn add_test_equipment(
    id: String,
    input: AddEquipmentInput,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let equipment = Equipment {
            name: input.name,
            specification: input.specification,
            calibration_required: input.calibration_required,
        };

        service.add_equipment(&id, equipment)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Remove equipment
#[tauri::command]
pub async fn remove_test_equipment(
    id: String,
    equipment_name: String,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);
        service.remove_equipment(&id, &equipment_name)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Add precondition
#[tauri::command]
pub async fn add_test_precondition(
    id: String,
    precondition: String,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);
        service.add_precondition(&id, precondition)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Add acceptance criterion
#[tauri::command]
pub async fn add_test_acceptance_criterion(
    id: String,
    criterion: String,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);
        service.add_acceptance_criterion(&id, criterion)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Set sample size
#[tauri::command]
pub async fn set_test_sample_size(
    id: String,
    input: SetSampleSizeInput,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let sample_size = SampleSize {
            quantity: input.quantity,
            rationale: input.rationale,
            sampling_method: input.sampling_method,
        };

        service.set_sample_size(&id, sample_size)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Set environment
#[tauri::command]
pub async fn set_test_environment(
    id: String,
    input: SetEnvironmentInput,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let environment = Environment {
            temperature: input.temperature,
            humidity: input.humidity,
            other: input.other,
        };

        service.set_environment(&id, environment)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Add verifies link to requirement
#[tauri::command]
pub async fn add_test_verifies_link(
    id: String,
    req_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let entity_id: EntityId = req_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid requirement ID: {}", e)))?;

        service.add_verifies_link(&id, entity_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Add mitigates link to risk
#[tauri::command]
pub async fn add_test_mitigates_link(
    id: String,
    risk_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Test> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let test = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = TestService::new(project, cache);

        let entity_id: EntityId = risk_id
            .parse()
            .map_err(|e| CommandError::InvalidInput(format!("Invalid risk ID: {}", e)))?;

        service.add_mitigates_link(&id, entity_id)?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(test)
}

/// Get tests by requirement
#[tauri::command]
pub async fn get_tests_by_requirement(
    req_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<TestSummary>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TestService::new(project, cache);
    let tests = service.get_by_requirement(&req_id)?;

    Ok(tests.iter().map(TestSummary::from).collect())
}

/// Get tests by risk
#[tauri::command]
pub async fn get_tests_by_risk(
    risk_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<TestSummary>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TestService::new(project, cache);
    let tests = service.get_by_risk(&risk_id)?;

    Ok(tests.iter().map(TestSummary::from).collect())
}

/// Get Test statistics
#[tauri::command]
pub async fn get_test_stats(state: State<'_, AppState>) -> CommandResult<TestStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TestService::new(project, cache);
    let stats = service.stats()?;

    Ok(stats)
}
