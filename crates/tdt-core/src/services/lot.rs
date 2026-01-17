//! Lot service for production lot / batch (DHR) management
//!
//! Provides CRUD operations and workflow management for manufacturing lots.

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core::cache::{CachedEntity, EntityCache, EntityFilter};
use crate::core::entity::{Entity, Status};
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::lot::{
    ExecutionStatus, ExecutionStep, Lot, LotStatus, MaterialUsed, WorkInstructionRef,
};
use crate::services::base::ServiceBase;
use crate::services::common::{
    CommonFilter, NoneLast, ServiceError, ServiceResult, SortDirection, SortKey, Sortable,
};

/// Filter options for listing lots
#[derive(Debug, Clone, Default)]
pub struct LotFilter {
    /// Common filters (status, author, tags, search)
    pub common: CommonFilter,
    /// Filter by lot status
    pub lot_status: Option<LotStatus>,
    /// Filter by product (ASM or CMP ID)
    pub product: Option<String>,
    /// Show only active lots (in_progress or on_hold)
    pub active_only: bool,
    /// Show recent lots (last N days)
    pub recent_days: Option<u32>,
    /// Sort field
    pub sort: LotSortField,
    /// Sort direction
    pub sort_direction: SortDirection,
}

/// Fields available for sorting lots
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotSortField {
    Id,
    Title,
    LotNumber,
    Quantity,
    LotStatus,
    Author,
    #[default]
    Created,
}

impl Sortable for Lot {
    type SortField = LotSortField;

    fn sort_key(&self, field: &Self::SortField) -> SortKey {
        match field {
            LotSortField::Id => SortKey::String(self.id.to_string()),
            LotSortField::Title => SortKey::String(self.title.clone()),
            LotSortField::LotNumber => {
                SortKey::OptionalString(NoneLast(self.lot_number.clone()))
            }
            LotSortField::Quantity => {
                SortKey::OptionalNumber(NoneLast(self.quantity.map(|q| q as i64)))
            }
            LotSortField::LotStatus => SortKey::String(self.lot_status.to_string()),
            LotSortField::Author => SortKey::String(self.author.clone()),
            LotSortField::Created => SortKey::DateTime(self.created.timestamp()),
        }
    }
}

/// Input for creating a new lot
#[derive(Debug, Clone)]
pub struct CreateLot {
    /// Title
    pub title: String,
    /// Lot number
    pub lot_number: Option<String>,
    /// Quantity
    pub quantity: Option<u32>,
    /// Product ID (ASM or CMP)
    pub product: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Start date
    pub start_date: Option<NaiveDate>,
    /// Tags
    pub tags: Vec<String>,
    /// Initial status
    pub status: Option<Status>,
    /// Author
    pub author: String,
}

/// Input for updating an existing lot
#[derive(Debug, Clone, Default)]
pub struct UpdateLot {
    /// Update title
    pub title: Option<String>,
    /// Update lot number
    pub lot_number: Option<Option<String>>,
    /// Update quantity
    pub quantity: Option<Option<u32>>,
    /// Update notes
    pub notes: Option<Option<String>>,
    /// Update document status
    pub status: Option<Status>,
    /// Update lot status (workflow)
    pub lot_status: Option<LotStatus>,
    /// Update start date
    pub start_date: Option<Option<NaiveDate>>,
    /// Update completion date
    pub completion_date: Option<Option<NaiveDate>>,
}

/// Statistics about lots
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LotStats {
    /// Total number of lots
    pub total: usize,
    /// Counts by lot status
    pub by_status: LotStatusCounts,
    /// Total quantity across all lots
    pub total_quantity: u64,
    /// Average quantity per lot
    pub avg_quantity: f64,
    /// Number of lots with git branches
    pub with_git_branch: usize,
    /// Number of merged branches
    pub merged_branches: usize,
}

/// Lot status counts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LotStatusCounts {
    pub in_progress: usize,
    pub on_hold: usize,
    pub completed: usize,
    pub scrapped: usize,
}

/// Input for updating an execution step
#[derive(Debug, Clone)]
pub struct UpdateStepInput {
    /// New status
    pub status: Option<ExecutionStatus>,
    /// Operator name
    pub operator: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Work instructions used
    pub work_instructions_used: Option<Vec<String>>,
    /// Mark as signed
    pub signed: bool,
}

/// Service for managing lots
pub struct LotService<'a> {
    project: &'a Project,
    base: ServiceBase<'a>,
    cache: &'a EntityCache,
}

impl<'a> LotService<'a> {
    /// Create a new LotService
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

    /// Get the lots directory
    fn lot_dir(&self) -> PathBuf {
        self.project().root().join("manufacturing/lots")
    }

    /// Get the file path for a lot
    fn get_file_path(&self, id: &EntityId) -> PathBuf {
        self.lot_dir().join(format!("{}.tdt.yaml", id))
    }

    /// Load all lots
    fn load_all(&self) -> ServiceResult<Vec<Lot>> {
        let dir = self.lot_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        loader::load_all(&dir).map_err(ServiceError::from)
    }

    /// List lots using the cache (fast path)
    ///
    /// Returns cached entity data without loading full entities from disk.
    /// Use this for list views and simple queries.
    pub fn list_cached(&self, filter: &LotFilter) -> ServiceResult<Vec<CachedEntity>> {
        // Build cache filter
        let status = filter
            .common
            .status
            .as_ref()
            .and_then(|s| s.first())
            .copied();

        let entity_filter = EntityFilter {
            prefix: Some(EntityPrefix::Lot),
            status,
            author: filter.common.author.clone(),
            search: filter.common.search.clone(),
            limit: None, // Apply limit after all filters
            priority: filter.common.priority.as_ref().and_then(|p| p.first()).copied(),
            entity_type: None,
            category: None,
        };

        let mut cached = self.cache.list_entities(&entity_filter);

        // Apply additional filters not supported by cache query
        if let Some(days) = filter.recent_days {
            let cutoff = Utc::now() - chrono::Duration::days(days as i64);
            cached.retain(|e| e.created >= cutoff);
        }

        // Note: lot_status, product, active_only require full entity load
        // These are handled in the regular list() method

        // Apply limit
        if let Some(limit) = filter.common.limit {
            cached.truncate(limit);
        }

        Ok(cached)
    }

    /// Find a lot by ID
    ///
    /// Uses the cache to find the file path for faster lookup.
    fn find_lot(&self, id: &str) -> ServiceResult<(PathBuf, Lot)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(lot) = crate::yaml::parse_yaml_file::<Lot>(&path) {
                    return Ok((path, lot));
                }
            }
        }

        // Fall back to directory scan
        let dir = self.lot_dir();
        if let Some((path, lot)) = loader::load_entity::<Lot>(&dir, id)? {
            return Ok((path, lot));
        }
        Err(ServiceError::NotFound(format!("Lot: {}", id)))
    }

    /// List lots with filtering and sorting
    pub fn list(&self, filter: &LotFilter) -> ServiceResult<Vec<Lot>> {
        let mut lots = self.load_all()?;

        // Apply filters
        lots.retain(|lot| {
            // Common filter
            if !filter.common.matches_status_str(lot.status()) {
                return false;
            }
            if !filter.common.matches_author(&lot.author) {
                return false;
            }
            // Search in title and lot number
            let mut search_fields: Vec<&str> = vec![&lot.title];
            if let Some(ref ln) = lot.lot_number {
                search_fields.push(ln);
            }
            if !filter.common.matches_search(&search_fields) {
                return false;
            }

            // Lot status filter
            if let Some(ref status) = filter.lot_status {
                if &lot.lot_status != status {
                    return false;
                }
            }

            // Product filter
            if let Some(ref product) = filter.product {
                if let Some(ref lot_product) = lot.links.product {
                    if !lot_product.contains(product) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Active only filter
            if filter.active_only {
                if lot.lot_status != LotStatus::InProgress
                    && lot.lot_status != LotStatus::OnHold
                {
                    return false;
                }
            }

            // Recent filter
            if let Some(days) = filter.recent_days {
                let cutoff = Utc::now() - chrono::Duration::days(days as i64);
                if lot.created < cutoff {
                    return false;
                }
            }

            true
        });

        // Sort
        crate::services::common::sort_entities(&mut lots, filter.sort, filter.sort_direction);

        Ok(lots)
    }

    /// Get a lot by ID
    pub fn get(&self, id: &str) -> ServiceResult<Option<Lot>> {
        match self.find_lot(id) {
            Ok((_, lot)) => Ok(Some(lot)),
            Err(ServiceError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get a lot by ID, returning error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Lot> {
        let (_, lot) = self.find_lot(id)?;
        Ok(lot)
    }

    /// Create a new lot
    pub fn create(&self, input: CreateLot) -> ServiceResult<Lot> {
        let id = EntityId::new(EntityPrefix::Lot);

        let mut lot = Lot::new(input.title, input.author);
        lot.id = id;
        lot.lot_number = input.lot_number;
        lot.quantity = input.quantity;
        lot.notes = input.notes;
        lot.start_date = input.start_date;

        if let Some(product) = input.product {
            lot.links.product = Some(product);
        }

        if let Some(status) = input.status {
            lot.status = status;
        }

        // Ensure directory exists
        let dir = self.lot_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        // Save
        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Update an existing lot
    pub fn update(&self, id: &str, input: UpdateLot) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if let Some(title) = input.title {
            lot.title = title;
        }
        if let Some(lot_number) = input.lot_number {
            lot.lot_number = lot_number;
        }
        if let Some(quantity) = input.quantity {
            lot.quantity = quantity;
        }
        if let Some(notes) = input.notes {
            lot.notes = notes;
        }
        if let Some(status) = input.status {
            lot.status = status;
        }
        if let Some(lot_status) = input.lot_status {
            lot.lot_status = lot_status;
        }
        if let Some(start_date) = input.start_date {
            lot.start_date = start_date;
        }
        if let Some(completion_date) = input.completion_date {
            lot.completion_date = completion_date;
        }

        // Increment revision
        lot.entity_revision += 1;

        // Save
        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Delete a lot
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, _) = self.find_lot(id)?;

        if !force {
            // Check for references (could check NCRs, results, etc.)
            // For now, allow deletion
        }

        fs::remove_file(&path)?;
        Ok(())
    }

    /// Set the product link for a lot
    pub fn set_product(&self, id: &str, product_id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        lot.links.product = Some(product_id.to_string());
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Add a material to the lot
    pub fn add_material(&self, id: &str, material: MaterialUsed) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        lot.materials_used.push(material);
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Remove a material from the lot
    pub fn remove_material(&self, id: &str, component_id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        lot.materials_used.retain(|m| {
            m.component.as_ref().map(|c| c != component_id).unwrap_or(true)
        });
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Add an execution step
    pub fn add_step(&self, id: &str, step: ExecutionStep) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        lot.execution.push(step);
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Update an execution step by index
    pub fn update_step(
        &self,
        id: &str,
        step_index: usize,
        input: UpdateStepInput,
    ) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if step_index >= lot.execution.len() {
            return Err(ServiceError::ValidationFailed(format!(
                "Step index {} out of range (lot has {} steps)",
                step_index,
                lot.execution.len()
            )));
        }

        let step = &mut lot.execution[step_index];

        if let Some(status) = input.status {
            // Set started_date if transitioning to in_progress
            if status == ExecutionStatus::InProgress && step.started_date.is_none() {
                step.started_date = Some(chrono::Local::now().date_naive());
            }
            // Set completed_date if transitioning to completed
            if status == ExecutionStatus::Completed {
                step.completed_date = Some(chrono::Local::now().date_naive());
            }
            step.status = status;
        }

        if let Some(operator) = input.operator {
            step.operator = Some(operator);
        }

        if let Some(notes) = input.notes {
            step.notes = Some(notes);
        }

        if let Some(wis) = input.work_instructions_used {
            step.work_instructions_used = wis
                .into_iter()
                .map(|wi_id| WorkInstructionRef {
                    id: wi_id,
                    revision: None,
                })
                .collect();
        }

        if input.signed {
            step.signature_verified = Some(true);
        }

        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Remove an execution step by index
    pub fn remove_step(&self, id: &str, step_index: usize) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if step_index >= lot.execution.len() {
            return Err(ServiceError::ValidationFailed(format!(
                "Step index {} out of range (lot has {} steps)",
                step_index,
                lot.execution.len()
            )));
        }

        lot.execution.remove(step_index);
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Put a lot on hold
    pub fn put_on_hold(&self, id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if lot.lot_status == LotStatus::Completed {
            return Err(ServiceError::ValidationFailed(
                "Cannot put a completed lot on hold".to_string(),
            ));
        }
        if lot.lot_status == LotStatus::Scrapped {
            return Err(ServiceError::ValidationFailed(
                "Cannot put a scrapped lot on hold".to_string(),
            ));
        }

        lot.lot_status = LotStatus::OnHold;
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Resume a lot from hold
    pub fn resume(&self, id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if lot.lot_status != LotStatus::OnHold {
            return Err(ServiceError::ValidationFailed(
                "Can only resume lots that are on hold".to_string(),
            ));
        }

        lot.lot_status = LotStatus::InProgress;
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Complete a lot
    pub fn complete(&self, id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if lot.lot_status == LotStatus::Completed {
            return Err(ServiceError::ValidationFailed(
                "Lot is already completed".to_string(),
            ));
        }
        if lot.lot_status == LotStatus::Scrapped {
            return Err(ServiceError::ValidationFailed(
                "Cannot complete a scrapped lot".to_string(),
            ));
        }

        // Check for incomplete steps
        let incomplete_count = lot
            .execution
            .iter()
            .filter(|s| {
                s.status != ExecutionStatus::Completed && s.status != ExecutionStatus::Skipped
            })
            .count();

        if incomplete_count > 0 {
            return Err(ServiceError::ValidationFailed(format!(
                "Cannot complete lot: {} step(s) are incomplete",
                incomplete_count
            )));
        }

        lot.lot_status = LotStatus::Completed;
        lot.completion_date = Some(chrono::Local::now().date_naive());
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Complete a lot, allowing incomplete steps
    pub fn complete_force(&self, id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if lot.lot_status == LotStatus::Completed {
            return Err(ServiceError::ValidationFailed(
                "Lot is already completed".to_string(),
            ));
        }
        if lot.lot_status == LotStatus::Scrapped {
            return Err(ServiceError::ValidationFailed(
                "Cannot complete a scrapped lot".to_string(),
            ));
        }

        lot.lot_status = LotStatus::Completed;
        lot.completion_date = Some(chrono::Local::now().date_naive());
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Scrap a lot
    pub fn scrap(&self, id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if lot.lot_status == LotStatus::Scrapped {
            return Err(ServiceError::ValidationFailed(
                "Lot is already scrapped".to_string(),
            ));
        }

        lot.lot_status = LotStatus::Scrapped;
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Link an NCR to the lot
    pub fn add_ncr(&self, id: &str, ncr_id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if !lot.links.ncrs.contains(&ncr_id.to_string()) {
            lot.links.ncrs.push(ncr_id.to_string());
        }
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Remove an NCR link from the lot
    pub fn remove_ncr(&self, id: &str, ncr_id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        lot.links.ncrs.retain(|n| n != ncr_id);
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Link a result to the lot
    pub fn add_result(&self, id: &str, result_id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        if !lot.links.results.contains(&result_id.to_string()) {
            lot.links.results.push(result_id.to_string());
        }
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Set the git branch for a lot
    pub fn set_git_branch(&self, id: &str, branch: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        lot.git_branch = Some(branch.to_string());
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Mark the git branch as merged
    pub fn mark_branch_merged(&self, id: &str) -> ServiceResult<Lot> {
        let (_, mut lot) = self.find_lot(id)?;

        lot.branch_merged = true;
        lot.entity_revision += 1;

        let file_path = self.get_file_path(&lot.id);
        self.base.save(&lot, &file_path)?;

        Ok(lot)
    }

    /// Get the index of the next pending step in a lot
    pub fn get_next_step_index(&self, id: &str) -> ServiceResult<Option<usize>> {
        let (_, lot) = self.find_lot(id)?;

        // Find first pending or in-progress step
        let next_idx = lot.execution.iter().position(|s| {
            s.status == ExecutionStatus::Pending || s.status == ExecutionStatus::InProgress
        });

        Ok(next_idx)
    }

    /// Get statistics about lots
    pub fn stats(&self) -> ServiceResult<LotStats> {
        let lots = self.load_all()?;

        let mut stats = LotStats {
            total: lots.len(),
            ..Default::default()
        };

        let mut total_qty: u64 = 0;
        let mut qty_count = 0;

        for lot in &lots {
            // Count by status
            match lot.lot_status {
                LotStatus::InProgress => stats.by_status.in_progress += 1,
                LotStatus::OnHold => stats.by_status.on_hold += 1,
                LotStatus::Completed => stats.by_status.completed += 1,
                LotStatus::Scrapped => stats.by_status.scrapped += 1,
            }

            // Sum quantities
            if let Some(qty) = lot.quantity {
                total_qty += qty as u64;
                qty_count += 1;
            }

            // Git branch counts
            if lot.git_branch.is_some() {
                stats.with_git_branch += 1;
            }
            if lot.branch_merged {
                stats.merged_branches += 1;
            }
        }

        stats.total_quantity = total_qty;
        stats.avg_quantity = if qty_count > 0 {
            total_qty as f64 / qty_count as f64
        } else {
            0.0
        };

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

    fn make_create_lot(title: &str) -> CreateLot {
        CreateLot {
            title: title.to_string(),
            lot_number: None,
            quantity: None,
            product: None,
            notes: None,
            start_date: None,
            tags: Vec::new(),
            status: None,
            author: "Test Author".to_string(),
        }
    }

    #[test]
    fn test_create_lot() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let mut input = make_create_lot("Test Lot");
        input.lot_number = Some("LOT-001".to_string());
        input.quantity = Some(100);

        let lot = service.create(input).unwrap();

        assert!(lot.id.to_string().starts_with("LOT-"));
        assert_eq!(lot.title, "Test Lot");
        assert_eq!(lot.lot_number, Some("LOT-001".to_string()));
        assert_eq!(lot.quantity, Some(100));
        assert_eq!(lot.lot_status, LotStatus::InProgress);
    }

    #[test]
    fn test_get_lot() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let created = service.create(input).unwrap();

        let retrieved = service.get(&created.id.to_string()).unwrap().unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.title, "Test Lot");
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        // Create multiple lots
        service.create(make_create_lot("Lot A")).unwrap();
        let lot_b = service.create(make_create_lot("Lot B")).unwrap();
        service.create(make_create_lot("Lot C")).unwrap();

        // Put one on hold
        service.put_on_hold(&lot_b.id.to_string()).unwrap();

        // List all
        let all = service.list(&LotFilter::default()).unwrap();
        assert_eq!(all.len(), 3);

        // List on hold only
        let on_hold = service
            .list(&LotFilter {
                lot_status: Some(LotStatus::OnHold),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(on_hold.len(), 1);
        assert_eq!(on_hold[0].id, lot_b.id);
    }

    #[test]
    fn test_update_lot() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let created = service.create(input).unwrap();

        let updated = service
            .update(
                &created.id.to_string(),
                UpdateLot {
                    title: Some("Updated Lot".to_string()),
                    quantity: Some(Some(50)),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Lot");
        assert_eq!(updated.quantity, Some(50));
        assert_eq!(updated.entity_revision, 2);
    }

    #[test]
    fn test_delete_lot() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let created = service.create(input).unwrap();

        service.delete(&created.id.to_string(), false).unwrap();

        let result = service.get(&created.id.to_string()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_lot_workflow() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let lot = service.create(input).unwrap();
        assert_eq!(lot.lot_status, LotStatus::InProgress);

        // Put on hold
        let lot = service.put_on_hold(&lot.id.to_string()).unwrap();
        assert_eq!(lot.lot_status, LotStatus::OnHold);

        // Resume
        let lot = service.resume(&lot.id.to_string()).unwrap();
        assert_eq!(lot.lot_status, LotStatus::InProgress);

        // Force complete (no steps)
        let lot = service.complete_force(&lot.id.to_string()).unwrap();
        assert_eq!(lot.lot_status, LotStatus::Completed);
        assert!(lot.completion_date.is_some());
    }

    #[test]
    fn test_add_and_update_step() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let lot = service.create(input).unwrap();

        // Add step
        let step = ExecutionStep {
            process: Some("PROC-001".to_string()),
            status: ExecutionStatus::Pending,
            ..Default::default()
        };
        let lot = service.add_step(&lot.id.to_string(), step).unwrap();
        assert_eq!(lot.execution.len(), 1);

        // Update step
        let lot = service
            .update_step(
                &lot.id.to_string(),
                0,
                UpdateStepInput {
                    status: Some(ExecutionStatus::Completed),
                    operator: Some("John Doe".to_string()),
                    notes: Some("Completed successfully".to_string()),
                    work_instructions_used: None,
                    signed: true,
                },
            )
            .unwrap();

        assert_eq!(lot.execution[0].status, ExecutionStatus::Completed);
        assert_eq!(lot.execution[0].operator, Some("John Doe".to_string()));
        assert!(lot.execution[0].completed_date.is_some());
        assert_eq!(lot.execution[0].signature_verified, Some(true));
    }

    #[test]
    fn test_add_material() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let lot = service.create(input).unwrap();

        let material = MaterialUsed {
            component: Some("CMP-001".to_string()),
            supplier_lot: Some("SUPPLIER-LOT-123".to_string()),
            quantity: Some(50),
        };

        let lot = service.add_material(&lot.id.to_string(), material).unwrap();
        assert_eq!(lot.materials_used.len(), 1);
        assert_eq!(lot.materials_used[0].component, Some("CMP-001".to_string()));
    }

    #[test]
    fn test_ncr_links() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let lot = service.create(input).unwrap();

        let lot = service.add_ncr(&lot.id.to_string(), "NCR-001").unwrap();
        assert!(lot.links.ncrs.contains(&"NCR-001".to_string()));

        let lot = service.remove_ncr(&lot.id.to_string(), "NCR-001").unwrap();
        assert!(!lot.links.ncrs.contains(&"NCR-001".to_string()));
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        // Create lots with different statuses
        let mut input = make_create_lot("Lot A");
        input.quantity = Some(100);
        let lot_a = service.create(input).unwrap();

        let mut input = make_create_lot("Lot B");
        input.quantity = Some(50);
        let lot_b = service.create(input).unwrap();
        service.put_on_hold(&lot_b.id.to_string()).unwrap();

        service.set_git_branch(&lot_a.id.to_string(), "lot/lot-a").unwrap();

        let stats = service.stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.by_status.in_progress, 1);
        assert_eq!(stats.by_status.on_hold, 1);
        assert_eq!(stats.total_quantity, 150);
        assert_eq!(stats.with_git_branch, 1);
    }

    #[test]
    fn test_complete_requires_all_steps() {
        let (_tmp, project, cache) = setup();
        let service = LotService::new(&project, &cache);

        let input = make_create_lot("Test Lot");
        let lot = service.create(input).unwrap();

        // Add incomplete step
        let step = ExecutionStep {
            process: Some("PROC-001".to_string()),
            status: ExecutionStatus::Pending,
            ..Default::default()
        };
        let lot = service.add_step(&lot.id.to_string(), step).unwrap();

        // Try to complete - should fail
        let result = service.complete(&lot.id.to_string());
        assert!(result.is_err());

        // Complete the step
        service
            .update_step(
                &lot.id.to_string(),
                0,
                UpdateStepInput {
                    status: Some(ExecutionStatus::Completed),
                    operator: None,
                    notes: None,
                    work_instructions_used: None,
                    signed: false,
                },
            )
            .unwrap();

        // Now complete should work
        let lot = service.complete(&lot.id.to_string()).unwrap();
        assert_eq!(lot.lot_status, LotStatus::Completed);
    }
}
