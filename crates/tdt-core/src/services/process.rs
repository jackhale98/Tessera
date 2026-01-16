//! Process service - business logic for manufacturing process management
//!
//! Provides CRUD operations and equipment/parameter management for processes.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core::cache::EntityCache;
use crate::core::entity::Status;
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::process::{
    Equipment, Process, ProcessCapability, ProcessLinks, ProcessParameter, ProcessSafety,
    ProcessType, SkillLevel, StepApprovalConfig,
};

use super::common::{
    apply_pagination, CommonFilter, ListResult, ServiceError, ServiceResult, SortDirection,
};

/// Filter options specific to processes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessFilter {
    /// Common filter options (status, author, search, etc.)
    #[serde(flatten)]
    pub common: CommonFilter,

    /// Filter by process type
    pub process_type: Option<ProcessType>,

    /// Filter by operation number (substring match)
    pub operation_number: Option<String>,

    /// Show only processes with equipment defined
    pub has_equipment: bool,

    /// Show only processes with capability data
    pub has_capability: bool,

    /// Show only processes requiring signature
    pub requires_signature: bool,

    /// Filter by skill level
    pub skill_level: Option<SkillLevel>,
}

impl ProcessFilter {
    /// Create a filter for machining processes
    pub fn machining() -> Self {
        Self {
            process_type: Some(ProcessType::Machining),
            ..Default::default()
        }
    }

    /// Create a filter for assembly processes
    pub fn assembly() -> Self {
        Self {
            process_type: Some(ProcessType::Assembly),
            ..Default::default()
        }
    }

    /// Create a filter for inspection processes
    pub fn inspection() -> Self {
        Self {
            process_type: Some(ProcessType::Inspection),
            ..Default::default()
        }
    }

    /// Create a filter for test processes
    pub fn test() -> Self {
        Self {
            process_type: Some(ProcessType::Test),
            ..Default::default()
        }
    }
}

/// Sort field for processes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessSortField {
    Id,
    #[default]
    Title,
    ProcessType,
    OperationNumber,
    CycleTime,
    Status,
    Author,
    Created,
}

/// Input for creating a new process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProcess {
    /// Process title
    pub title: String,

    /// Author name
    pub author: String,

    /// Process type classification
    #[serde(default)]
    pub process_type: ProcessType,

    /// Operation number (e.g., "OP-010")
    #[serde(default)]
    pub operation_number: Option<String>,

    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,

    /// Cycle time in minutes
    #[serde(default)]
    pub cycle_time_minutes: Option<f64>,

    /// Setup time in minutes
    #[serde(default)]
    pub setup_time_minutes: Option<f64>,

    /// Required operator skill level
    #[serde(default)]
    pub operator_skill: SkillLevel,

    /// Whether operator signature is required
    #[serde(default)]
    pub require_signature: bool,

    /// Classification tags
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for CreateProcess {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            process_type: ProcessType::default(),
            operation_number: None,
            description: None,
            cycle_time_minutes: None,
            setup_time_minutes: None,
            operator_skill: SkillLevel::default(),
            require_signature: false,
            tags: Vec::new(),
        }
    }
}

/// Input for updating an existing process
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateProcess {
    /// Update title
    pub title: Option<String>,

    /// Update process type
    pub process_type: Option<ProcessType>,

    /// Update operation number
    pub operation_number: Option<String>,

    /// Update description
    pub description: Option<String>,

    /// Update cycle time
    pub cycle_time_minutes: Option<f64>,

    /// Update setup time
    pub setup_time_minutes: Option<f64>,

    /// Update skill level
    pub operator_skill: Option<SkillLevel>,

    /// Update signature requirement
    pub require_signature: Option<bool>,

    /// Update status
    pub status: Option<Status>,

    /// Replace tags
    pub tags: Option<Vec<String>>,

    /// Replace equipment list
    pub equipment: Option<Vec<Equipment>>,

    /// Replace parameters list
    pub parameters: Option<Vec<ProcessParameter>>,

    /// Update capability data
    pub capability: Option<ProcessCapability>,

    /// Update safety info
    pub safety: Option<ProcessSafety>,

    /// Update step approval config
    pub step_approval: Option<StepApprovalConfig>,
}

/// Statistics about processes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessStats {
    pub total: usize,
    pub by_type: ProcessTypeCounts,
    pub by_status: StatusCounts,
    pub with_equipment: usize,
    pub with_capability: usize,
    pub require_signature: usize,
    pub avg_cycle_time: Option<f64>,
    pub avg_setup_time: Option<f64>,
}

/// Counts by process type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessTypeCounts {
    pub machining: usize,
    pub assembly: usize,
    pub inspection: usize,
    pub test: usize,
    pub finishing: usize,
    pub packaging: usize,
    pub handling: usize,
    pub heat_treat: usize,
    pub welding: usize,
    pub coating: usize,
}

/// Counts by status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusCounts {
    pub draft: usize,
    pub review: usize,
    pub approved: usize,
    pub released: usize,
    pub obsolete: usize,
}

/// Service for process management
pub struct ProcessService<'a> {
    project: &'a Project,
    cache: &'a EntityCache,
}

impl<'a> ProcessService<'a> {
    /// Create a new process service
    pub fn new(project: &'a Project, cache: &'a EntityCache) -> Self {
        Self { project, cache }
    }

    /// Get the directory for storing processes
    fn get_directory(&self) -> PathBuf {
        self.project.root().join("manufacturing/processes")
    }

    /// Get the file path for a process
    fn get_file_path(&self, id: &EntityId) -> PathBuf {
        self.get_directory().join(format!("{}.tdt.yaml", id))
    }

    /// List processes with filtering and pagination
    pub fn list(
        &self,
        filter: &ProcessFilter,
        sort_by: ProcessSortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Process>> {
        let mut processes = self.load_all()?;

        // Apply filters
        processes.retain(|proc| self.matches_filter(proc, filter));

        // Sort
        self.sort_processes(&mut processes, sort_by, sort_dir);

        // Paginate
        Ok(apply_pagination(
            processes,
            filter.common.offset,
            filter.common.limit,
        ))
    }

    /// List processes from cache (fast path for list display)
    ///
    /// Returns cached process data without loading full YAML files.
    /// Use this for list commands where full entity data isn't needed.
    pub fn list_cached(&self) -> Vec<crate::core::CachedEntity> {
        use crate::core::cache::EntityFilter;
        use crate::core::identity::EntityPrefix;

        let filter = EntityFilter {
            prefix: Some(EntityPrefix::Proc),
            ..Default::default()
        };
        self.cache.list_entities(&filter)
    }

    /// Load all processes from the filesystem
    pub fn load_all(&self) -> ServiceResult<Vec<Process>> {
        let dir = self.get_directory();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        Ok(loader::load_all(&dir)?)
    }

    /// Get a single process by ID
    pub fn get(&self, id: &str) -> ServiceResult<Option<Process>> {
        let dir = self.get_directory();
        if let Some((_, proc)) = loader::load_entity::<Process>(&dir, id)? {
            return Ok(Some(proc));
        }
        Ok(None)
    }

    /// Get a process by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Process> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(id.to_string()))
    }

    /// Get a process by operation number
    pub fn get_by_operation_number(&self, op_number: &str) -> ServiceResult<Option<Process>> {
        let processes = self.load_all()?;
        Ok(processes.into_iter().find(|p| {
            p.operation_number
                .as_ref()
                .is_some_and(|op| op.eq_ignore_ascii_case(op_number))
        }))
    }

    /// Create a new process
    pub fn create(&self, input: CreateProcess) -> ServiceResult<Process> {
        // Check for duplicate operation number if provided
        if let Some(ref op) = input.operation_number {
            if let Some(existing) = self.get_by_operation_number(op)? {
                return Err(ServiceError::AlreadyExists(format!(
                    "Process with operation number '{}' already exists ({})",
                    op, existing.id
                )));
            }
        }

        let id = EntityId::new(EntityPrefix::Proc);

        let process = Process {
            id: id.clone(),
            title: input.title,
            description: input.description,
            process_type: input.process_type,
            operation_number: input.operation_number,
            equipment: Vec::new(),
            parameters: Vec::new(),
            cycle_time_minutes: input.cycle_time_minutes,
            setup_time_minutes: input.setup_time_minutes,
            capability: None,
            operator_skill: input.operator_skill,
            safety: None,
            require_signature: input.require_signature,
            step_approval: None,
            tags: input.tags,
            status: Status::Draft,
            links: ProcessLinks::default(),
            created: Utc::now(),
            author: input.author,
            entity_revision: 1,
        };

        // Ensure directory exists
        let dir = self.get_directory();
        fs::create_dir_all(&dir)?;

        // Write to file
        let path = self.get_file_path(&id);
        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Update an existing process
    pub fn update(&self, id: &str, input: UpdateProcess) -> ServiceResult<Process> {
        let (path, mut process) = self.find_process(id)?;

        // Check for duplicate operation number if changing it
        if let Some(new_op) = &input.operation_number {
            if process.operation_number.as_ref() != Some(new_op) {
                if let Some(existing) = self.get_by_operation_number(new_op)? {
                    if existing.id != process.id {
                        return Err(ServiceError::AlreadyExists(format!(
                            "Process with operation number '{}' already exists ({})",
                            new_op, existing.id
                        )));
                    }
                }
            }
        }

        // Apply updates
        if let Some(title) = input.title {
            process.title = title;
        }
        if let Some(process_type) = input.process_type {
            process.process_type = process_type;
        }
        if let Some(operation_number) = input.operation_number {
            process.operation_number = Some(operation_number);
        }
        if let Some(description) = input.description {
            process.description = Some(description);
        }
        if let Some(cycle_time) = input.cycle_time_minutes {
            process.cycle_time_minutes = Some(cycle_time);
        }
        if let Some(setup_time) = input.setup_time_minutes {
            process.setup_time_minutes = Some(setup_time);
        }
        if let Some(skill) = input.operator_skill {
            process.operator_skill = skill;
        }
        if let Some(require_sig) = input.require_signature {
            process.require_signature = require_sig;
        }
        if let Some(status) = input.status {
            process.status = status;
        }
        if let Some(tags) = input.tags {
            process.tags = tags;
        }
        if let Some(equipment) = input.equipment {
            process.equipment = equipment;
        }
        if let Some(parameters) = input.parameters {
            process.parameters = parameters;
        }
        if let Some(capability) = input.capability {
            process.capability = Some(capability);
        }
        if let Some(safety) = input.safety {
            process.safety = Some(safety);
        }
        if let Some(step_approval) = input.step_approval {
            process.step_approval = Some(step_approval);
        }

        // Increment revision
        process.entity_revision += 1;

        // Write back
        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Delete a process
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, process) = self.find_process(id)?;

        // Check for references unless force is true
        if !force {
            // Check if referenced in links
            if !process.links.produces.is_empty()
                || !process.links.controls.is_empty()
                || !process.links.work_instructions.is_empty()
            {
                return Err(ServiceError::HasReferences);
            }
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Add equipment to a process
    pub fn add_equipment(&self, id: &str, equipment: Equipment) -> ServiceResult<Process> {
        let (path, mut process) = self.find_process(id)?;

        // Check for duplicate equipment name
        if process.equipment.iter().any(|e| e.name == equipment.name) {
            return Err(ServiceError::AlreadyExists(format!(
                "Equipment '{}' already exists in process",
                equipment.name
            )));
        }

        process.equipment.push(equipment);
        process.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Remove equipment from a process
    pub fn remove_equipment(&self, id: &str, equipment_name: &str) -> ServiceResult<Process> {
        let (path, mut process) = self.find_process(id)?;

        let original_len = process.equipment.len();
        process.equipment.retain(|e| e.name != equipment_name);

        if process.equipment.len() == original_len {
            return Err(ServiceError::NotFound(format!(
                "Equipment '{}' not found in process",
                equipment_name
            )));
        }

        process.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Add a parameter to a process
    pub fn add_parameter(&self, id: &str, parameter: ProcessParameter) -> ServiceResult<Process> {
        let (path, mut process) = self.find_process(id)?;

        // Check for duplicate parameter name
        if process.parameters.iter().any(|p| p.name == parameter.name) {
            return Err(ServiceError::AlreadyExists(format!(
                "Parameter '{}' already exists in process",
                parameter.name
            )));
        }

        process.parameters.push(parameter);
        process.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Remove a parameter from a process
    pub fn remove_parameter(&self, id: &str, parameter_name: &str) -> ServiceResult<Process> {
        let (path, mut process) = self.find_process(id)?;

        let original_len = process.parameters.len();
        process.parameters.retain(|p| p.name != parameter_name);

        if process.parameters.len() == original_len {
            return Err(ServiceError::NotFound(format!(
                "Parameter '{}' not found in process",
                parameter_name
            )));
        }

        process.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Set capability data for a process
    pub fn set_capability(
        &self,
        id: &str,
        capability: ProcessCapability,
    ) -> ServiceResult<Process> {
        let (path, mut process) = self.find_process(id)?;

        process.capability = Some(capability);
        process.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Set safety information for a process
    pub fn set_safety(&self, id: &str, safety: ProcessSafety) -> ServiceResult<Process> {
        let (path, mut process) = self.find_process(id)?;

        process.safety = Some(safety);
        process.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&process).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(process)
    }

    /// Get statistics about processes
    pub fn stats(&self) -> ServiceResult<ProcessStats> {
        let processes = self.load_all()?;

        let mut stats = ProcessStats::default();
        stats.total = processes.len();

        let mut cycle_times: Vec<f64> = Vec::new();
        let mut setup_times: Vec<f64> = Vec::new();

        for proc in &processes {
            // Count by type
            match proc.process_type {
                ProcessType::Machining => stats.by_type.machining += 1,
                ProcessType::Assembly => stats.by_type.assembly += 1,
                ProcessType::Inspection => stats.by_type.inspection += 1,
                ProcessType::Test => stats.by_type.test += 1,
                ProcessType::Finishing => stats.by_type.finishing += 1,
                ProcessType::Packaging => stats.by_type.packaging += 1,
                ProcessType::Handling => stats.by_type.handling += 1,
                ProcessType::HeatTreat => stats.by_type.heat_treat += 1,
                ProcessType::Welding => stats.by_type.welding += 1,
                ProcessType::Coating => stats.by_type.coating += 1,
            }

            // Count by status
            match proc.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            // Count features
            if !proc.equipment.is_empty() {
                stats.with_equipment += 1;
            }
            if proc.capability.is_some() {
                stats.with_capability += 1;
            }
            if proc.require_signature {
                stats.require_signature += 1;
            }

            // Collect times for averaging
            if let Some(cycle) = proc.cycle_time_minutes {
                cycle_times.push(cycle);
            }
            if let Some(setup) = proc.setup_time_minutes {
                setup_times.push(setup);
            }
        }

        // Calculate averages
        if !cycle_times.is_empty() {
            stats.avg_cycle_time = Some(cycle_times.iter().sum::<f64>() / cycle_times.len() as f64);
        }
        if !setup_times.is_empty() {
            stats.avg_setup_time = Some(setup_times.iter().sum::<f64>() / setup_times.len() as f64);
        }

        Ok(stats)
    }

    // --- Private helper methods ---

    /// Find a process and its file path (cache-first lookup)
    fn find_process(&self, id: &str) -> ServiceResult<(PathBuf, Process)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(process) = crate::yaml::parse_yaml_file::<Process>(&path) {
                    return Ok((path, process));
                }
            }
        }

        // Fall back to directory scan
        let dir = self.get_directory();
        if let Some((path, proc)) = loader::load_entity::<Process>(&dir, id)? {
            return Ok((path, proc));
        }
        Err(ServiceError::NotFound(id.to_string()))
    }

    /// Check if a process matches the given filter
    fn matches_filter(&self, proc: &Process, filter: &ProcessFilter) -> bool {
        // Process type filter
        if let Some(proc_type) = &filter.process_type {
            if proc.process_type != *proc_type {
                return false;
            }
        }

        // Operation number filter
        if let Some(op) = &filter.operation_number {
            if !proc
                .operation_number
                .as_ref()
                .is_some_and(|o| o.to_lowercase().contains(&op.to_lowercase()))
            {
                return false;
            }
        }

        // Has equipment filter
        if filter.has_equipment && proc.equipment.is_empty() {
            return false;
        }

        // Has capability filter
        if filter.has_capability && proc.capability.is_none() {
            return false;
        }

        // Requires signature filter
        if filter.requires_signature && !proc.require_signature {
            return false;
        }

        // Skill level filter
        if let Some(skill) = &filter.skill_level {
            if proc.operator_skill != *skill {
                return false;
            }
        }

        // Common filters
        if !filter.common.matches_status(&proc.status) {
            return false;
        }
        if !filter.common.matches_author(&proc.author) {
            return false;
        }
        if !filter.common.matches_tags(&proc.tags) {
            return false;
        }
        if !filter.common.matches_search(&[&proc.title]) {
            return false;
        }
        if !filter.common.matches_recent(&proc.created) {
            return false;
        }

        true
    }

    /// Sort processes by the given field
    fn sort_processes(
        &self,
        processes: &mut [Process],
        sort_by: ProcessSortField,
        sort_dir: SortDirection,
    ) {
        processes.sort_by(|a, b| {
            let cmp = match sort_by {
                ProcessSortField::Id => a.id.to_string().cmp(&b.id.to_string()),
                ProcessSortField::Title => a.title.cmp(&b.title),
                ProcessSortField::ProcessType => {
                    format!("{:?}", a.process_type).cmp(&format!("{:?}", b.process_type))
                }
                ProcessSortField::OperationNumber => a.operation_number.cmp(&b.operation_number),
                ProcessSortField::CycleTime => a
                    .cycle_time_minutes
                    .partial_cmp(&b.cycle_time_minutes)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProcessSortField::Status => {
                    format!("{:?}", a.status).cmp(&format!("{:?}", b.status))
                }
                ProcessSortField::Author => a.author.cmp(&b.author),
                ProcessSortField::Created => a.created.cmp(&b.created),
            };

            match sort_dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();

        // Initialize project structure
        fs::create_dir_all(tmp.path().join(".tdt")).unwrap();
        fs::create_dir_all(tmp.path().join("manufacturing/processes")).unwrap();

        // Create config file
        fs::write(
            tmp.path().join(".tdt/config.yaml"),
            "author: Test Author\n",
        )
        .unwrap();

        let project = Project::discover_from(tmp.path()).unwrap();
        let cache = EntityCache::open(&project).unwrap();

        (tmp, project, cache)
    }

    #[test]
    fn test_create_process() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let input = CreateProcess {
            title: "CNC Milling".into(),
            author: "Test Author".into(),
            process_type: ProcessType::Machining,
            operation_number: Some("OP-010".into()),
            cycle_time_minutes: Some(15.0),
            ..Default::default()
        };

        let proc = service.create(input).unwrap();

        assert_eq!(proc.title, "CNC Milling");
        assert_eq!(proc.process_type, ProcessType::Machining);
        assert_eq!(proc.operation_number, Some("OP-010".to_string()));
        assert_eq!(proc.cycle_time_minutes, Some(15.0));
        assert_eq!(proc.status, Status::Draft);
    }

    #[test]
    fn test_duplicate_operation_number_rejected() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        // Create first process
        service
            .create(CreateProcess {
                title: "First".into(),
                author: "Test".into(),
                operation_number: Some("OP-010".into()),
                ..Default::default()
            })
            .unwrap();

        // Try to create duplicate
        let result = service.create(CreateProcess {
            title: "Second".into(),
            author: "Test".into(),
            operation_number: Some("OP-010".into()),
            ..Default::default()
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_get_process() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "Find Me".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Find Me");
    }

    #[test]
    fn test_get_by_operation_number() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        service
            .create(CreateProcess {
                title: "Test Process".into(),
                author: "Test".into(),
                operation_number: Some("OP-020".into()),
                ..Default::default()
            })
            .unwrap();

        let found = service.get_by_operation_number("op-020").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().operation_number, Some("OP-020".to_string()));
    }

    #[test]
    fn test_update_process() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "Original".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .update(
                &created.id.to_string(),
                UpdateProcess {
                    title: Some("Updated Title".into()),
                    cycle_time_minutes: Some(20.0),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.cycle_time_minutes, Some(20.0));
        assert_eq!(updated.entity_revision, 2);
    }

    #[test]
    fn test_delete_process() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "Delete Me".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service.delete(&created.id.to_string(), false).unwrap();

        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_add_equipment() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "With Equipment".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let equipment = Equipment {
            name: "CNC Mill".into(),
            equipment_id: Some("EQ-001".into()),
            capability: Some("3-axis".into()),
        };

        let updated = service
            .add_equipment(&created.id.to_string(), equipment)
            .unwrap();

        assert_eq!(updated.equipment.len(), 1);
        assert_eq!(updated.equipment[0].name, "CNC Mill");
    }

    #[test]
    fn test_remove_equipment() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "With Equipment".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let equipment = Equipment {
            name: "Lathe".into(),
            equipment_id: None,
            capability: None,
        };

        service
            .add_equipment(&created.id.to_string(), equipment)
            .unwrap();

        let updated = service
            .remove_equipment(&created.id.to_string(), "Lathe")
            .unwrap();

        assert!(updated.equipment.is_empty());
    }

    #[test]
    fn test_add_parameter() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "With Parameters".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let param = ProcessParameter {
            name: "Spindle Speed".into(),
            value: 1500.0,
            units: Some("RPM".into()),
            min: Some(500.0),
            max: Some(3000.0),
        };

        let updated = service
            .add_parameter(&created.id.to_string(), param)
            .unwrap();

        assert_eq!(updated.parameters.len(), 1);
        assert_eq!(updated.parameters[0].name, "Spindle Speed");
        assert_eq!(updated.parameters[0].value, 1500.0);
    }

    #[test]
    fn test_set_capability() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "Capable Process".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let capability = ProcessCapability {
            cpk: Some(1.67),
            ppk: Some(1.45),
            sample_size: Some(30),
            study_date: None,
        };

        let updated = service
            .set_capability(&created.id.to_string(), capability)
            .unwrap();

        assert!(updated.capability.is_some());
        let cap = updated.capability.unwrap();
        assert_eq!(cap.cpk, Some(1.67));
        assert_eq!(cap.ppk, Some(1.45));
    }

    #[test]
    fn test_set_safety() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "Safe Process".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let safety = ProcessSafety {
            ppe: vec!["Safety glasses".into(), "Gloves".into()],
            hazards: vec!["Flying chips".into()],
        };

        let updated = service
            .set_safety(&created.id.to_string(), safety)
            .unwrap();

        assert!(updated.safety.is_some());
        let safe = updated.safety.unwrap();
        assert_eq!(safe.ppe.len(), 2);
        assert_eq!(safe.hazards.len(), 1);
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        // Create machining process
        service
            .create(CreateProcess {
                title: "Mill Part".into(),
                author: "Test".into(),
                process_type: ProcessType::Machining,
                ..Default::default()
            })
            .unwrap();

        // Create assembly process
        service
            .create(CreateProcess {
                title: "Assemble Part".into(),
                author: "Test".into(),
                process_type: ProcessType::Assembly,
                ..Default::default()
            })
            .unwrap();

        // List all
        let all = service
            .list(
                &ProcessFilter::default(),
                ProcessSortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(all.items.len(), 2);

        // List machining only
        let machining = service
            .list(
                &ProcessFilter::machining(),
                ProcessSortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(machining.items.len(), 1);
        assert_eq!(machining.items[0].title, "Mill Part");
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        // Create processes with different types
        service
            .create(CreateProcess {
                title: "Machining 1".into(),
                author: "Test".into(),
                process_type: ProcessType::Machining,
                cycle_time_minutes: Some(10.0),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateProcess {
                title: "Assembly 1".into(),
                author: "Test".into(),
                process_type: ProcessType::Assembly,
                cycle_time_minutes: Some(20.0),
                require_signature: true,
                ..Default::default()
            })
            .unwrap();

        let stats = service.stats().unwrap();

        assert_eq!(stats.total, 2);
        assert_eq!(stats.by_type.machining, 1);
        assert_eq!(stats.by_type.assembly, 1);
        assert_eq!(stats.by_status.draft, 2);
        assert_eq!(stats.require_signature, 1);
        assert_eq!(stats.avg_cycle_time, Some(15.0)); // (10 + 20) / 2
    }

    #[test]
    fn test_duplicate_equipment_rejected() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let equipment = Equipment {
            name: "Mill".into(),
            equipment_id: None,
            capability: None,
        };

        service
            .add_equipment(&created.id.to_string(), equipment.clone())
            .unwrap();

        let result = service.add_equipment(&created.id.to_string(), equipment);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_parameter_rejected() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ProcessService::new(&project, &cache);

        let created = service
            .create(CreateProcess {
                title: "Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let param = ProcessParameter {
            name: "Speed".into(),
            value: 100.0,
            units: None,
            min: None,
            max: None,
        };

        service
            .add_parameter(&created.id.to_string(), param.clone())
            .unwrap();

        let result = service.add_parameter(&created.id.to_string(), param);
        assert!(result.is_err());
    }
}
