//! Result service for test execution record management
//!
//! Provides CRUD operations and business logic for test results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core::cache::{CachedResult, EntityCache};
use crate::core::entity::Status;
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::result::{
    Attachment, AttachmentType, Deviation, EquipmentUsed, Failure, Measurement, Result,
    ResultEnvironment, SampleInfo, StepResult, StepResultRecord, Verdict,
};
use crate::services::base::ServiceBase;
use crate::services::common::{
    CommonFilter, NoneLast, ServiceError, ServiceResult, SortDirection, SortKey, Sortable,
};

/// Filter options for listing results
#[derive(Debug, Clone, Default)]
pub struct ResultFilter {
    /// Common filters (status, author, tags, search)
    pub common: CommonFilter,
    /// Filter by verdict
    pub verdict: Option<Verdict>,
    /// Filter by test ID
    pub test_id: Option<String>,
    /// Filter by category
    pub category: Option<String>,
    /// Filter by executed_by (substring match)
    pub executed_by: Option<String>,
    /// Show only results with failures
    pub with_failures: bool,
    /// Show only results with deviations
    pub with_deviations: bool,
    /// Show results from last N days
    pub recent_days: Option<u32>,
    /// Sort field
    pub sort: ResultSortField,
    /// Sort direction
    pub sort_direction: SortDirection,
}

/// Fields available for sorting results
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultSortField {
    Id,
    Title,
    Test,
    Verdict,
    Status,
    Author,
    #[default]
    Created,
    ExecutedDate,
}

impl Sortable for Result {
    type SortField = ResultSortField;

    fn sort_key(&self, field: &Self::SortField) -> SortKey {
        match field {
            ResultSortField::Id => SortKey::String(self.id.to_string()),
            ResultSortField::Title => SortKey::OptionalString(NoneLast(self.title.clone())),
            ResultSortField::Test => SortKey::String(self.test_id.to_string()),
            ResultSortField::Verdict => {
                // Sort failures first, then conditional, incomplete, pass, N/A
                let order = match self.verdict {
                    Verdict::Fail => 0,
                    Verdict::Conditional => 1,
                    Verdict::Incomplete => 2,
                    Verdict::Pass => 3,
                    Verdict::NotApplicable => 4,
                };
                SortKey::Number(order as i64)
            }
            ResultSortField::Status => SortKey::String(self.status.to_string()),
            ResultSortField::Author => SortKey::String(self.author.clone()),
            ResultSortField::Created => SortKey::DateTime(self.created.timestamp()),
            ResultSortField::ExecutedDate => SortKey::DateTime(self.executed_date.timestamp()),
        }
    }
}

/// Input for creating a new result
#[derive(Debug, Clone)]
pub struct CreateResult {
    /// Test ID that was executed
    pub test_id: EntityId,
    /// Test revision
    pub test_revision: Option<u32>,
    /// Optional title
    pub title: Option<String>,
    /// Overall verdict
    pub verdict: Verdict,
    /// Verdict rationale
    pub verdict_rationale: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Who executed the test
    pub executed_by: String,
    /// When test was executed (defaults to now)
    pub executed_date: Option<DateTime<Utc>>,
    /// Sample info
    pub sample_info: Option<SampleInfo>,
    /// Environment during test
    pub environment: Option<ResultEnvironment>,
    /// Duration
    pub duration: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Initial status
    pub status: Option<Status>,
    /// Author
    pub author: String,
}

/// Input for updating an existing result
#[derive(Debug, Clone, Default)]
pub struct UpdateResult {
    /// Update title
    pub title: Option<Option<String>>,
    /// Update verdict
    pub verdict: Option<Verdict>,
    /// Update verdict rationale
    pub verdict_rationale: Option<Option<String>>,
    /// Update category
    pub category: Option<Option<String>>,
    /// Update tags
    pub tags: Option<Vec<String>>,
    /// Update reviewed_by
    pub reviewed_by: Option<Option<String>>,
    /// Update reviewed_date
    pub reviewed_date: Option<Option<DateTime<Utc>>>,
    /// Update sample info
    pub sample_info: Option<Option<SampleInfo>>,
    /// Update environment
    pub environment: Option<Option<ResultEnvironment>>,
    /// Update duration
    pub duration: Option<Option<String>>,
    /// Update notes
    pub notes: Option<Option<String>>,
    /// Update status
    pub status: Option<Status>,
}

/// Statistics about results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultStats {
    /// Total number of results
    pub total: usize,
    /// Counts by verdict
    pub by_verdict: VerdictCounts,
    /// Counts by status
    pub by_status: ResultStatusCounts,
    /// Pass rate percentage (0-100)
    pub pass_rate: f64,
    /// Number with failures
    pub with_failures: usize,
    /// Number with deviations
    pub with_deviations: usize,
    /// Verification results count
    pub verification_count: usize,
    /// Validation results count
    pub validation_count: usize,
}

/// Verdict counts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerdictCounts {
    pub pass: usize,
    pub fail: usize,
    pub conditional: usize,
    pub incomplete: usize,
    pub not_applicable: usize,
}

/// Result status counts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultStatusCounts {
    pub draft: usize,
    pub review: usize,
    pub approved: usize,
    pub released: usize,
    pub obsolete: usize,
}

/// Service for managing test results
pub struct ResultService<'a> {
    project: &'a Project,
    base: ServiceBase<'a>,
    cache: &'a EntityCache,
}

impl<'a> ResultService<'a> {
    /// Create a new ResultService
    pub fn new(project: &'a Project, cache: &'a EntityCache) -> Self {
        Self {
            project,
            base: ServiceBase::new(project, cache),
            cache,
        }
    }

    /// Get a reference to the project
    fn project(&self) -> &Project {
        self.project
    }

    /// Get the results directories
    fn result_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.project().root().join("verification/results"),
            self.project().root().join("validation/results"),
        ]
    }

    /// Load all results from all directories
    fn load_all(&self) -> ServiceResult<Vec<Result>> {
        let mut all_results = Vec::new();
        for dir in self.result_dirs() {
            if dir.exists() {
                let results: Vec<Result> = loader::load_all(&dir)?;
                all_results.extend(results);
            }
        }
        Ok(all_results)
    }

    /// List results using the cache (fast path)
    ///
    /// Returns cached result data without loading full entities from disk.
    /// Use this for list views and simple queries.
    pub fn list_cached(&self, filter: &ResultFilter) -> ServiceResult<Vec<CachedResult>> {
        // Convert filter values to strings for cache query
        let status_str = filter
            .common
            .status
            .as_ref()
            .and_then(|s| s.first())
            .map(|s| format!("{:?}", s).to_lowercase());

        let verdict_str = filter.verdict.as_ref().map(|v| match v {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::Conditional => "conditional",
            Verdict::Incomplete => "incomplete",
            Verdict::NotApplicable => "not_applicable",
        });

        // Query cache
        let mut cached = self.cache.list_results(
            status_str.as_deref(),
            filter.test_id.as_deref(),
            verdict_str,
            filter.common.author.as_deref(),
            filter.common.search.as_deref(),
            None, // Apply limit after all filters
        );

        // Apply additional filters not supported by cache query
        if let Some(days) = filter.recent_days {
            let cutoff = Utc::now() - chrono::Duration::days(days as i64);
            cached.retain(|r| r.created >= cutoff);
        }

        if let Some(ref exec) = filter.executed_by {
            cached.retain(|r| {
                r.executed_by
                    .as_ref()
                    .map(|e| e.to_lowercase().contains(&exec.to_lowercase()))
                    .unwrap_or(false)
            });
        }

        // with_failures and with_deviations require full entity load
        // They are handled in the regular list() method

        // Apply limit
        if let Some(limit) = filter.common.limit {
            cached.truncate(limit);
        }

        Ok(cached)
    }

    /// Find a result by ID, returning the path and result
    ///
    /// Uses the cache to find the file path for faster lookup.
    fn find_result(&self, id: &str) -> ServiceResult<(PathBuf, Result)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(result) = crate::yaml::parse_yaml_file::<Result>(&path) {
                    return Ok((path, result));
                }
            }
        }

        // Fall back to directory scan
        for dir in self.result_dirs() {
            if let Some((path, result)) = loader::load_entity::<Result>(&dir, id)? {
                return Ok((path, result));
            }
        }
        Err(ServiceError::NotFound(format!("Result: {}", id)))
    }

    /// List results with filtering and sorting
    pub fn list(&self, filter: &ResultFilter) -> ServiceResult<Vec<Result>> {
        let mut results = self.load_all()?;

        // Apply filters
        results.retain(|r| self.matches_filter(r, filter));

        // Sort
        crate::services::common::sort_entities(&mut results, filter.sort, filter.sort_direction);

        // Apply limit from common filter
        if let Some(limit) = filter.common.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Check if a result matches the filter
    fn matches_filter(&self, result: &Result, filter: &ResultFilter) -> bool {
        // Common filters
        if !filter.common.matches_entity(result) {
            return false;
        }

        // Verdict filter
        if let Some(ref verdict) = filter.verdict {
            if &result.verdict != verdict {
                return false;
            }
        }

        // Test ID filter
        if let Some(ref test_id) = filter.test_id {
            if !result.test_id.to_string().contains(test_id) {
                return false;
            }
        }

        // Category filter
        if let Some(ref cat) = filter.category {
            match &result.category {
                Some(result_cat) if result_cat.to_lowercase() == cat.to_lowercase() => {}
                _ => return false,
            }
        }

        // Executed by filter
        if let Some(ref exec) = filter.executed_by {
            if !result
                .executed_by
                .to_lowercase()
                .contains(&exec.to_lowercase())
            {
                return false;
            }
        }

        // With failures filter
        if filter.with_failures && result.failures.is_empty() {
            return false;
        }

        // With deviations filter
        if filter.with_deviations && result.deviations.is_empty() {
            return false;
        }

        // Recent days filter
        if let Some(days) = filter.recent_days {
            let cutoff = Utc::now() - chrono::Duration::days(days as i64);
            if result.executed_date < cutoff {
                return false;
            }
        }

        true
    }

    /// Get a result by ID (cache-first lookup)
    pub fn get(&self, id: &str) -> ServiceResult<Option<Result>> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                match crate::yaml::parse_yaml_file::<Result>(&path) {
                    Ok(result) => return Ok(Some(result)),
                    Err(_) => {} // Fall through to directory scan
                }
            }
        }

        // Fall back to directory scan
        for dir in self.result_dirs() {
            if let Some((_, result)) = loader::load_entity::<Result>(&dir, id)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    /// Get a result by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Result> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(format!("Result: {}", id)))
    }

    /// Create a new result
    pub fn create(&self, input: CreateResult) -> ServiceResult<Result> {
        let now = Utc::now();
        let id = EntityId::new(EntityPrefix::Rslt);

        let result = Result {
            id: id.clone(),
            test_id: input.test_id.clone(),
            test_revision: input.test_revision,
            title: input.title,
            verdict: input.verdict,
            verdict_rationale: input.verdict_rationale,
            category: input.category,
            tags: input.tags,
            executed_date: input.executed_date.unwrap_or(now),
            executed_by: input.executed_by,
            reviewed_by: None,
            reviewed_date: None,
            sample_info: input.sample_info,
            environment: input.environment,
            equipment_used: Vec::new(),
            step_results: Vec::new(),
            deviations: Vec::new(),
            failures: Vec::new(),
            attachments: Vec::new(),
            duration: input.duration,
            notes: input.notes,
            status: input.status.unwrap_or(Status::Draft),
            links: Default::default(),
            created: now,
            author: input.author,
            revision: 1,
        };

        // Determine directory and file path
        let file_path = self.get_file_path(&result);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Determine the result directory based on test location
    fn determine_result_dir(&self, test_id: &EntityId) -> PathBuf {
        let verification_test = self
            .project()
            .root()
            .join(format!("verification/protocols/{}.tdt.yaml", test_id));
        if verification_test.exists() {
            self.project().root().join("verification/results")
        } else {
            self.project().root().join("validation/results")
        }
    }

    /// Get the file path for a result
    pub fn get_file_path(&self, result: &Result) -> PathBuf {
        let dir = self.determine_result_dir(&result.test_id);
        dir.join(format!("{}.tdt.yaml", result.id))
    }

    /// Update an existing result
    pub fn update(&self, id: &str, input: UpdateResult) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        // Apply updates
        if let Some(title) = input.title {
            result.title = title;
        }
        if let Some(verdict) = input.verdict {
            result.verdict = verdict;
        }
        if let Some(rationale) = input.verdict_rationale {
            result.verdict_rationale = rationale;
        }
        if let Some(category) = input.category {
            result.category = category;
        }
        if let Some(tags) = input.tags {
            result.tags = tags;
        }
        if let Some(reviewed_by) = input.reviewed_by {
            result.reviewed_by = reviewed_by;
        }
        if let Some(reviewed_date) = input.reviewed_date {
            result.reviewed_date = reviewed_date;
        }
        if let Some(sample_info) = input.sample_info {
            result.sample_info = sample_info;
        }
        if let Some(environment) = input.environment {
            result.environment = environment;
        }
        if let Some(duration) = input.duration {
            result.duration = duration;
        }
        if let Some(notes) = input.notes {
            result.notes = notes;
        }
        if let Some(status) = input.status {
            result.status = status;
        }

        result.revision += 1;

        // Save to file
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Delete a result
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, _result) = self.find_result(id)?;

        // Check for references unless force is true
        if !force {
            let links_to = self.cache.get_links_to(id);
            if !links_to.is_empty() {
                return Err(ServiceError::HasReferences);
            }
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Add a step result
    pub fn add_step_result(
        &self,
        id: &str,
        step: u32,
        step_result: StepResult,
        observed: Option<String>,
        measurement: Option<Measurement>,
        notes: Option<String>,
    ) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        // Check if step already exists
        if result.step_results.iter().any(|s| s.step == step) {
            return Err(ServiceError::ValidationFailed(format!(
                "Step {} already has a result recorded",
                step
            )));
        }

        result.step_results.push(StepResultRecord {
            step,
            result: step_result,
            observed,
            measurement,
            notes,
        });

        // Sort by step number
        result.step_results.sort_by_key(|s| s.step);

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Update a step result
    pub fn update_step_result(
        &self,
        id: &str,
        step: u32,
        step_result: Option<StepResult>,
        observed: Option<Option<String>>,
        measurement: Option<Option<Measurement>>,
        notes: Option<Option<String>>,
    ) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        let step_record = result
            .step_results
            .iter_mut()
            .find(|s| s.step == step)
            .ok_or_else(|| {
                ServiceError::ValidationFailed(format!("Step {} not found in results", step))
            })?;

        if let Some(r) = step_result {
            step_record.result = r;
        }
        if let Some(o) = observed {
            step_record.observed = o;
        }
        if let Some(m) = measurement {
            step_record.measurement = m;
        }
        if let Some(n) = notes {
            step_record.notes = n;
        }

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Remove a step result
    pub fn remove_step_result(&self, id: &str, step: u32) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        let original_len = result.step_results.len();
        result.step_results.retain(|s| s.step != step);

        if result.step_results.len() == original_len {
            return Err(ServiceError::ValidationFailed(format!(
                "Step {} not found in results",
                step
            )));
        }

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Record a failure
    pub fn record_failure(
        &self,
        id: &str,
        description: String,
        step: Option<u32>,
        root_cause: Option<String>,
        corrective_action: Option<String>,
    ) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        result.failures.push(Failure {
            description,
            step,
            root_cause,
            corrective_action,
            action_id: None,
        });

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Remove a failure by index
    pub fn remove_failure(&self, id: &str, index: usize) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        if index >= result.failures.len() {
            return Err(ServiceError::ValidationFailed(format!(
                "Failure index {} out of range (0-{})",
                index,
                result.failures.len().saturating_sub(1)
            )));
        }

        result.failures.remove(index);

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Record a deviation
    pub fn record_deviation(
        &self,
        id: &str,
        description: String,
        impact: Option<String>,
        justification: Option<String>,
    ) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        result.deviations.push(Deviation {
            description,
            impact,
            justification,
        });

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Remove a deviation by index
    pub fn remove_deviation(&self, id: &str, index: usize) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        if index >= result.deviations.len() {
            return Err(ServiceError::ValidationFailed(format!(
                "Deviation index {} out of range (0-{})",
                index,
                result.deviations.len().saturating_sub(1)
            )));
        }

        result.deviations.remove(index);

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Add an attachment
    pub fn add_attachment(
        &self,
        id: &str,
        filename: String,
        path: Option<String>,
        attachment_type: Option<AttachmentType>,
        description: Option<String>,
    ) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        // Check if attachment with same filename exists
        if result.attachments.iter().any(|a| a.filename == filename) {
            return Err(ServiceError::ValidationFailed(format!(
                "Attachment with filename '{}' already exists",
                filename
            )));
        }

        result.attachments.push(Attachment {
            filename,
            path,
            attachment_type,
            description,
        });

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Remove an attachment by filename
    pub fn remove_attachment(&self, id: &str, filename: &str) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        let original_len = result.attachments.len();
        result.attachments.retain(|a| a.filename != filename);

        if result.attachments.len() == original_len {
            return Err(ServiceError::ValidationFailed(format!(
                "Attachment '{}' not found",
                filename
            )));
        }

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Add equipment used record
    pub fn add_equipment(
        &self,
        id: &str,
        name: String,
        asset_id: Option<String>,
        calibration_date: Option<String>,
        calibration_due: Option<String>,
    ) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        // Check if equipment with same name or asset_id already exists
        if result.equipment_used.iter().any(|e| e.name == name) {
            return Err(ServiceError::ValidationFailed(format!(
                "Equipment '{}' already recorded",
                name
            )));
        }

        result.equipment_used.push(EquipmentUsed {
            name,
            asset_id,
            calibration_date,
            calibration_due,
        });

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Remove equipment used by name
    pub fn remove_equipment(&self, id: &str, name: &str) -> ServiceResult<Result> {
        let mut result = self.get_required(id)?;

        let original_len = result.equipment_used.len();
        result.equipment_used.retain(|e| e.name != name);

        if result.equipment_used.len() == original_len {
            return Err(ServiceError::ValidationFailed(format!(
                "Equipment '{}' not found",
                name
            )));
        }

        result.revision += 1;
        let file_path = self.get_file_path(&result);
        self.base.save(&result, &file_path)?;

        Ok(result)
    }

    /// Set sample info
    pub fn set_sample_info(&self, id: &str, sample_info: SampleInfo) -> ServiceResult<Result> {
        self.update(
            id,
            UpdateResult {
                sample_info: Some(Some(sample_info)),
                ..Default::default()
            },
        )
    }

    /// Set environment
    pub fn set_environment(
        &self,
        id: &str,
        environment: ResultEnvironment,
    ) -> ServiceResult<Result> {
        self.update(
            id,
            UpdateResult {
                environment: Some(Some(environment)),
                ..Default::default()
            },
        )
    }

    /// Mark result as reviewed
    pub fn mark_reviewed(&self, id: &str, reviewer: String) -> ServiceResult<Result> {
        self.update(
            id,
            UpdateResult {
                reviewed_by: Some(Some(reviewer)),
                reviewed_date: Some(Some(Utc::now())),
                ..Default::default()
            },
        )
    }

    /// Get results for a specific test
    pub fn get_results_for_test(&self, test_id: &str) -> ServiceResult<Vec<Result>> {
        self.list(&ResultFilter {
            test_id: Some(test_id.to_string()),
            ..Default::default()
        })
    }

    /// Get the latest result for a test
    pub fn get_latest_result_for_test(&self, test_id: &str) -> ServiceResult<Option<Result>> {
        let mut results = self.get_results_for_test(test_id)?;
        // Sort by executed_date descending
        results.sort_by(|a, b| b.executed_date.cmp(&a.executed_date));
        Ok(results.into_iter().next())
    }

    /// Calculate statistics
    pub fn stats(&self) -> ServiceResult<ResultStats> {
        let results = self.list(&ResultFilter::default())?;

        let mut stats = ResultStats {
            total: results.len(),
            ..Default::default()
        };

        for result in &results {
            // Count by verdict
            match result.verdict {
                Verdict::Pass => stats.by_verdict.pass += 1,
                Verdict::Fail => stats.by_verdict.fail += 1,
                Verdict::Conditional => stats.by_verdict.conditional += 1,
                Verdict::Incomplete => stats.by_verdict.incomplete += 1,
                Verdict::NotApplicable => stats.by_verdict.not_applicable += 1,
            }

            // Count by status
            match result.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            // Count failures and deviations
            if !result.failures.is_empty() {
                stats.with_failures += 1;
            }
            if !result.deviations.is_empty() {
                stats.with_deviations += 1;
            }
        }

        // Calculate pass rate (excluding N/A)
        let countable = stats.total - stats.by_verdict.not_applicable;
        if countable > 0 {
            stats.pass_rate = (stats.by_verdict.pass as f64 / countable as f64) * 100.0;
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();
        let project = Project::init(tmp.path()).unwrap();
        let cache = EntityCache::open(&project).unwrap();
        (tmp, project, cache)
    }

    fn make_create_result(
        test_id: EntityId,
        title: Option<String>,
        verdict: Verdict,
    ) -> CreateResult {
        CreateResult {
            test_id,
            test_revision: None,
            title,
            verdict,
            verdict_rationale: None,
            category: None,
            tags: Vec::new(),
            executed_by: "tester".to_string(),
            executed_date: None,
            sample_info: None,
            environment: None,
            duration: None,
            notes: None,
            status: None,
            author: "author".to_string(),
        }
    }

    fn create_test_result(service: &ResultService) -> Result {
        let test_id = EntityId::new(EntityPrefix::Test);
        service
            .create(make_create_result(
                test_id,
                Some("Test Run 1".to_string()),
                Verdict::Pass,
            ))
            .unwrap()
    }

    #[test]
    fn test_create_result() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let test_id = EntityId::new(EntityPrefix::Test);
        let result = service
            .create(make_create_result(
                test_id.clone(),
                Some("Test Run".to_string()),
                Verdict::Pass,
            ))
            .unwrap();

        assert!(result.id.to_string().starts_with("RSLT-"));
        assert_eq!(result.test_id, test_id);
        assert_eq!(result.verdict, Verdict::Pass);
        assert_eq!(result.status, Status::Draft);
    }

    #[test]
    fn test_get_result() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let retrieved = service.get(&created.id.to_string()).unwrap().unwrap();

        assert_eq!(created.id, retrieved.id);
        assert_eq!(created.verdict, retrieved.verdict);
    }

    #[test]
    fn test_update_result() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let updated = service
            .update(
                &created.id.to_string(),
                UpdateResult {
                    verdict: Some(Verdict::Fail),
                    verdict_rationale: Some(Some("Device overheated".to_string())),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.verdict, Verdict::Fail);
        assert_eq!(
            updated.verdict_rationale,
            Some("Device overheated".to_string())
        );
        assert_eq!(updated.revision, 2);
    }

    #[test]
    fn test_delete_result() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        service.delete(&created.id.to_string(), false).unwrap();

        let result = service.get(&created.id.to_string()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        // Create results with different verdicts
        let test_id = EntityId::new(EntityPrefix::Test);
        service
            .create(make_create_result(test_id.clone(), None, Verdict::Pass))
            .unwrap();
        service
            .create(make_create_result(test_id.clone(), None, Verdict::Fail))
            .unwrap();

        // Filter by verdict
        let failures = service
            .list(&ResultFilter {
                verdict: Some(Verdict::Fail),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].verdict, Verdict::Fail);
    }

    #[test]
    fn test_add_step_result() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let updated = service
            .add_step_result(
                &created.id.to_string(),
                1,
                StepResult::Pass,
                Some("As expected".to_string()),
                None,
                None,
            )
            .unwrap();

        assert_eq!(updated.step_results.len(), 1);
        assert_eq!(updated.step_results[0].step, 1);
        assert_eq!(updated.step_results[0].result, StepResult::Pass);
    }

    #[test]
    fn test_remove_step_result() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let with_step = service
            .add_step_result(
                &created.id.to_string(),
                1,
                StepResult::Pass,
                None,
                None,
                None,
            )
            .unwrap();

        let removed = service
            .remove_step_result(&with_step.id.to_string(), 1)
            .unwrap();

        assert!(removed.step_results.is_empty());
    }

    #[test]
    fn test_record_failure() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let updated = service
            .record_failure(
                &created.id.to_string(),
                "Device overheated".to_string(),
                Some(3),
                Some("Insufficient cooling".to_string()),
                None,
            )
            .unwrap();

        assert_eq!(updated.failures.len(), 1);
        assert_eq!(updated.failures[0].description, "Device overheated");
        assert_eq!(updated.failures[0].step, Some(3));
    }

    #[test]
    fn test_record_deviation() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let updated = service
            .record_deviation(
                &created.id.to_string(),
                "Temperature out of spec".to_string(),
                Some("Minor impact".to_string()),
                Some("Accepted due to time constraints".to_string()),
            )
            .unwrap();

        assert_eq!(updated.deviations.len(), 1);
        assert_eq!(updated.deviations[0].description, "Temperature out of spec");
    }

    #[test]
    fn test_add_equipment() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let updated = service
            .add_equipment(
                &created.id.to_string(),
                "Multimeter".to_string(),
                Some("DMM-001".to_string()),
                Some("2024-01-15".to_string()),
                Some("2025-01-15".to_string()),
            )
            .unwrap();

        assert_eq!(updated.equipment_used.len(), 1);
        assert_eq!(updated.equipment_used[0].name, "Multimeter");
        assert_eq!(
            updated.equipment_used[0].asset_id,
            Some("DMM-001".to_string())
        );
    }

    #[test]
    fn test_mark_reviewed() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let created = create_test_result(&service);
        let reviewed = service
            .mark_reviewed(&created.id.to_string(), "reviewer".to_string())
            .unwrap();

        assert_eq!(reviewed.reviewed_by, Some("reviewer".to_string()));
        assert!(reviewed.reviewed_date.is_some());
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup();
        let service = ResultService::new(&project, &cache);

        let test_id = EntityId::new(EntityPrefix::Test);

        // Create results with different verdicts
        for verdict in [Verdict::Pass, Verdict::Pass, Verdict::Fail] {
            service
                .create(make_create_result(test_id.clone(), None, verdict))
                .unwrap();
        }

        let stats = service.stats().unwrap();

        assert_eq!(stats.total, 3);
        assert_eq!(stats.by_verdict.pass, 2);
        assert_eq!(stats.by_verdict.fail, 1);
        assert!((stats.pass_rate - 66.67).abs() < 0.1);
    }
}
