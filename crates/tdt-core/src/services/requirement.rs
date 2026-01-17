//! Requirement service - business logic for requirement management
//!
//! This service uses the EntityCache for fast queries and only loads
//! full entities from disk when necessary.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::core::cache::{CachedRequirement, EntityCache};
use crate::core::entity::{Priority, Status};
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::requirement::{Level, Links, Requirement, RequirementType, Source};

use super::common::{
    apply_pagination, CommonFilter, ListResult, ServiceError, ServiceResult, SortDirection,
};

/// Filter options specific to requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequirementFilter {
    /// Common filter options (status, priority, author, search, etc.)
    #[serde(flatten)]
    pub common: CommonFilter,

    /// Filter by requirement type (input/output)
    pub req_type: Option<RequirementType>,

    /// Filter by V-model level
    pub level: Option<Level>,

    /// Filter by category
    pub category: Option<String>,

    /// Show only orphaned requirements (no satisfied_by or verified_by links)
    pub orphans_only: bool,

    /// Show only requirements needing review (draft or review status)
    pub needs_review: bool,

    /// Show only unverified requirements (no verified_by links)
    pub unverified_only: bool,
}

impl RequirementFilter {
    /// Create a filter for input requirements
    pub fn inputs() -> Self {
        Self {
            req_type: Some(RequirementType::Input),
            ..Default::default()
        }
    }

    /// Create a filter for output requirements
    pub fn outputs() -> Self {
        Self {
            req_type: Some(RequirementType::Output),
            ..Default::default()
        }
    }

    /// Create a filter for urgent requirements (high or critical priority)
    pub fn urgent() -> Self {
        Self {
            common: CommonFilter {
                priority: Some(vec![Priority::High, Priority::Critical]),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Sort field for requirements
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementSortField {
    Id,
    Title,
    Type,
    Level,
    Status,
    Priority,
    Category,
    Author,
    #[default]
    Created,
}

/// Input for creating a new requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequirement {
    /// Requirement type (input or output)
    pub req_type: RequirementType,

    /// Short title
    pub title: String,

    /// Full requirement text
    pub text: String,

    /// Author name
    pub author: String,

    /// V-model level
    #[serde(default)]
    pub level: Level,

    /// Priority
    #[serde(default)]
    pub priority: Priority,

    /// Category
    pub category: Option<String>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Rationale
    pub rationale: Option<String>,

    /// Acceptance criteria
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,

    /// Source reference
    pub source: Option<Source>,
}

impl Default for CreateRequirement {
    fn default() -> Self {
        Self {
            req_type: RequirementType::Input,
            title: String::new(),
            text: String::new(),
            author: String::new(),
            level: Level::System,
            priority: Priority::Medium,
            category: None,
            tags: Vec::new(),
            rationale: None,
            acceptance_criteria: Vec::new(),
            source: None,
        }
    }
}

/// Input for updating an existing requirement
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateRequirement {
    /// Update title
    pub title: Option<String>,

    /// Update text
    pub text: Option<String>,

    /// Update V-model level
    pub level: Option<Level>,

    /// Update priority
    pub priority: Option<Priority>,

    /// Update status
    pub status: Option<Status>,

    /// Update category
    pub category: Option<String>,

    /// Update tags (replaces existing)
    pub tags: Option<Vec<String>>,

    /// Update rationale
    pub rationale: Option<String>,

    /// Update acceptance criteria (replaces existing)
    pub acceptance_criteria: Option<Vec<String>>,

    /// Update source
    pub source: Option<Source>,
}

/// Service for requirement management
pub struct RequirementService<'a> {
    project: &'a Project,
    cache: &'a EntityCache,
}

impl<'a> RequirementService<'a> {
    /// Create a new requirement service
    pub fn new(project: &'a Project, cache: &'a EntityCache) -> Self {
        Self { project, cache }
    }

    /// Get the directory for storing requirements of a given type
    fn get_directory(&self, req_type: RequirementType) -> PathBuf {
        match req_type {
            RequirementType::Input => self.project.root().join("requirements/inputs"),
            RequirementType::Output => self.project.root().join("requirements/outputs"),
        }
    }

    /// Get the file path for a requirement
    fn get_file_path(&self, id: &EntityId, req_type: RequirementType) -> PathBuf {
        let dir = self.get_directory(req_type);
        dir.join(format!("{}.tdt.yaml", id))
    }

    /// List requirements using the cache (fast path)
    ///
    /// Returns cached requirement data without loading full entities from disk.
    /// Use this for list views and simple queries.
    pub fn list_cached(
        &self,
        filter: &RequirementFilter,
    ) -> ServiceResult<ListResult<CachedRequirement>> {
        // Build cache query parameters
        let status_str = filter
            .common
            .status
            .as_ref()
            .and_then(|s| s.first())
            .map(|s| s.to_string());

        let priority_str = filter
            .common
            .priority
            .as_ref()
            .and_then(|p| p.first())
            .map(|p| match p {
                Priority::Low => "low",
                Priority::Medium => "medium",
                Priority::High => "high",
                Priority::Critical => "critical",
            });

        let req_type_str = filter.req_type.as_ref().map(|t| match t {
            RequirementType::Input => "input",
            RequirementType::Output => "output",
        });

        // Query cache
        let mut cached = self.cache.list_requirements(
            status_str.as_deref(),
            priority_str,
            req_type_str,
            filter.category.as_deref(),
            filter.common.author.as_deref(),
            filter.common.search.as_deref(),
            None, // Apply limit after all filters
        );

        // Apply additional filters not supported by cache query
        if filter.needs_review {
            cached.retain(|r| r.status == Status::Draft || r.status == Status::Review);
        }

        if let Some(level) = &filter.level {
            let level_str = level.to_string();
            cached.retain(|r| r.level.as_ref().map(|l| l == &level_str).unwrap_or(false));
        }

        if let Some(days) = filter.common.recent_days {
            let cutoff = Utc::now() - chrono::Duration::days(days as i64);
            cached.retain(|r| r.created >= cutoff);
        }

        if let Some(tags) = &filter.common.tags {
            cached.retain(|r| {
                tags.iter().any(|t| {
                    r.tags
                        .iter()
                        .any(|rt| rt.to_lowercase() == t.to_lowercase())
                })
            });
        }

        // Apply link-based filters using cache
        if filter.orphans_only {
            let orphan_ids = self.get_orphan_ids();
            cached.retain(|r| orphan_ids.contains(&r.id));
        }

        if filter.unverified_only {
            let unverified_ids = self.get_unverified_ids();
            cached.retain(|r| unverified_ids.contains(&r.id));
        }

        // Paginate
        Ok(apply_pagination(
            cached,
            filter.common.offset,
            filter.common.limit,
        ))
    }

    /// List requirements with full entity data
    ///
    /// Loads full requirement entities from disk. Use this when you need
    /// access to text, rationale, acceptance_criteria, or links.
    pub fn list(
        &self,
        filter: &RequirementFilter,
        sort_by: RequirementSortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Requirement>> {
        // Use cache to get file paths, then load full entities
        let cached = self.list_cached(filter)?;

        let mut requirements: Vec<Requirement> = Vec::new();
        for c in &cached.items {
            // Cache stores relative paths - resolve to absolute
            let full_path = if c.file_path.is_absolute() {
                c.file_path.clone()
            } else {
                self.project.root().join(&c.file_path)
            };

            match crate::yaml::parse_yaml_file::<Requirement>(&full_path) {
                Ok(req) => requirements.push(req),
                Err(e) => {
                    // Log error but continue - don't fail entire list for one bad file
                    eprintln!("Warning: Failed to load requirement from {:?}: {}", full_path, e);
                }
            }
        }

        // Sort
        self.sort_requirements(&mut requirements, sort_by, sort_dir);

        Ok(ListResult::new(requirements, cached.total_count, cached.has_more))
    }

    /// Get IDs of orphaned requirements (no incoming links)
    fn get_orphan_ids(&self) -> HashSet<String> {
        let all_reqs = self.cache.list_requirements(None, None, None, None, None, None, None);
        let mut orphans = HashSet::new();

        for req in all_reqs {
            let links_to = self.cache.get_links_to(&req.id);
            if links_to.is_empty() {
                orphans.insert(req.id);
            }
        }

        orphans
    }

    /// Get IDs of unverified requirements (no verified_by links)
    fn get_unverified_ids(&self) -> HashSet<String> {
        let all_reqs = self.cache.list_requirements(None, None, None, None, None, None, None);
        let mut unverified = HashSet::new();

        for req in all_reqs {
            let verifies_links = self.cache.get_links_to_of_type(&req.id, "verifies");
            if verifies_links.is_empty() {
                unverified.insert(req.id);
            }
        }

        unverified
    }

    /// Load all requirements from the filesystem (slow, use list_cached when possible)
    pub fn load_all(&self) -> ServiceResult<Vec<Requirement>> {
        let mut requirements = Vec::new();

        // Load from inputs directory
        let inputs_dir = self.get_directory(RequirementType::Input);
        if inputs_dir.exists() {
            let loaded: Vec<Requirement> = loader::load_all(&inputs_dir)?;
            requirements.extend(loaded);
        }

        // Load from outputs directory
        let outputs_dir = self.get_directory(RequirementType::Output);
        if outputs_dir.exists() {
            let loaded: Vec<Requirement> = loader::load_all(&outputs_dir)?;
            requirements.extend(loaded);
        }

        Ok(requirements)
    }

    /// Get a single requirement by ID
    ///
    /// Uses the cache to find the file path, then loads the full entity.
    pub fn get(&self, id: &str) -> ServiceResult<Option<Requirement>> {
        // Try cache first for fast lookup
        if let Some(entity) = self.cache.get_entity(id) {
            if entity.prefix == "REQ" {
                // Cache stores relative paths - resolve to absolute
                let full_path = if entity.file_path.is_absolute() {
                    entity.file_path.clone()
                } else {
                    self.project.root().join(&entity.file_path)
                };
                if let Ok(req) = crate::yaml::parse_yaml_file::<Requirement>(&full_path) {
                    return Ok(Some(req));
                }
            }
        }

        // Fallback to filesystem search if not in cache
        let inputs_dir = self.get_directory(RequirementType::Input);
        if let Some((_, req)) = loader::load_entity::<Requirement>(&inputs_dir, id)? {
            return Ok(Some(req));
        }

        let outputs_dir = self.get_directory(RequirementType::Output);
        if let Some((_, req)) = loader::load_entity::<Requirement>(&outputs_dir, id)? {
            return Ok(Some(req));
        }

        Ok(None)
    }

    /// Get a requirement by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Requirement> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(id.to_string()).into())
    }

    /// Create a new requirement
    pub fn create(&self, input: CreateRequirement) -> ServiceResult<Requirement> {
        let id = EntityId::new(EntityPrefix::Req);

        let requirement = Requirement {
            id: id.clone(),
            req_type: input.req_type,
            level: input.level,
            title: input.title,
            text: input.text,
            author: input.author,
            priority: input.priority,
            status: Status::Draft,
            category: input.category,
            tags: input.tags,
            rationale: input.rationale,
            acceptance_criteria: input.acceptance_criteria,
            source: input.source,
            links: Links::default(),
            created: Utc::now(),
            revision: 1,
        };

        // Ensure directory exists
        let dir = self.get_directory(input.req_type);
        fs::create_dir_all(&dir)?;

        // Write to file
        let path = self.get_file_path(&id, input.req_type);
        let yaml = serde_yml::to_string(&requirement)
            .map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(requirement)
    }

    /// Update an existing requirement
    pub fn update(&self, id: &str, input: UpdateRequirement) -> ServiceResult<Requirement> {
        // Find the requirement
        let (path, mut requirement) = self.find_requirement(id)?;

        // Apply updates
        if let Some(title) = input.title {
            requirement.title = title;
        }
        if let Some(text) = input.text {
            requirement.text = text;
        }
        if let Some(level) = input.level {
            requirement.level = level;
        }
        if let Some(priority) = input.priority {
            requirement.priority = priority;
        }
        if let Some(status) = input.status {
            requirement.status = status;
        }
        if let Some(category) = input.category {
            requirement.category = Some(category);
        }
        if let Some(tags) = input.tags {
            requirement.tags = tags;
        }
        if let Some(rationale) = input.rationale {
            requirement.rationale = Some(rationale);
        }
        if let Some(acceptance_criteria) = input.acceptance_criteria {
            requirement.acceptance_criteria = acceptance_criteria;
        }
        if let Some(source) = input.source {
            requirement.source = Some(source);
        }

        // Increment revision
        requirement.revision += 1;

        // Write back
        let yaml = serde_yml::to_string(&requirement)
            .map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(requirement)
    }

    /// Delete a requirement
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, requirement) = self.find_requirement(id)?;

        // Check for references unless force is true
        if !force {
            let references = self.find_references(&requirement.id)?;
            if !references.is_empty() {
                return Err(ServiceError::HasReferences.into());
            }
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Find a requirement and its file path
    fn find_requirement(&self, id: &str) -> ServiceResult<(PathBuf, Requirement)> {
        // Try inputs first
        let inputs_dir = self.get_directory(RequirementType::Input);
        if let Some((path, req)) = loader::load_entity::<Requirement>(&inputs_dir, id)? {
            return Ok((path, req));
        }

        // Try outputs
        let outputs_dir = self.get_directory(RequirementType::Output);
        if let Some((path, req)) = loader::load_entity::<Requirement>(&outputs_dir, id)? {
            return Ok((path, req));
        }

        Err(ServiceError::NotFound(id.to_string()).into())
    }

    /// Find entities that reference this requirement
    fn find_references(&self, _id: &EntityId) -> ServiceResult<Vec<EntityId>> {
        // TODO: Implement reference checking via cache or file scan
        // For now, return empty (always allow delete)
        Ok(Vec::new())
    }

    /// Check if a requirement matches the given filter
    fn matches_filter(&self, req: &Requirement, filter: &RequirementFilter) -> bool {
        // Type filter
        if let Some(req_type) = &filter.req_type {
            if req.req_type != *req_type {
                return false;
            }
        }

        // Level filter
        if let Some(level) = &filter.level {
            if req.level != *level {
                return false;
            }
        }

        // Category filter
        if let Some(category) = &filter.category {
            if req.category.as_ref().map(|c| c.to_lowercase())
                != Some(category.to_lowercase())
            {
                return false;
            }
        }

        // Orphans filter
        if filter.orphans_only {
            if !req.links.satisfied_by.is_empty() || !req.links.verified_by.is_empty() {
                return false;
            }
        }

        // Needs review filter
        if filter.needs_review {
            if req.status != Status::Draft && req.status != Status::Review {
                return false;
            }
        }

        // Unverified filter
        if filter.unverified_only {
            if !req.links.verified_by.is_empty() {
                return false;
            }
        }

        // Common filters
        if !filter.common.matches_status(&req.status) {
            return false;
        }
        if !filter.common.matches_priority(&req.priority) {
            return false;
        }
        if !filter.common.matches_author(&req.author) {
            return false;
        }
        if !filter.common.matches_tags(&req.tags) {
            return false;
        }
        if !filter.common.matches_search(&[&req.title, &req.text]) {
            return false;
        }
        if !filter.common.matches_recent(&req.created) {
            return false;
        }

        true
    }

    /// Sort requirements by the given field
    fn sort_requirements(
        &self,
        reqs: &mut [Requirement],
        sort_by: RequirementSortField,
        sort_dir: SortDirection,
    ) {
        reqs.sort_by(|a, b| {
            let cmp = match sort_by {
                RequirementSortField::Id => a.id.to_string().cmp(&b.id.to_string()),
                RequirementSortField::Title => a.title.cmp(&b.title),
                RequirementSortField::Type => {
                    format!("{}", a.req_type).cmp(&format!("{}", b.req_type))
                }
                RequirementSortField::Level => format!("{}", a.level).cmp(&format!("{}", b.level)),
                RequirementSortField::Status => a.status.cmp(&b.status),
                RequirementSortField::Priority => {
                    // Priority: Critical > High > Medium > Low
                    let priority_order = |p: &Priority| match p {
                        Priority::Critical => 0,
                        Priority::High => 1,
                        Priority::Medium => 2,
                        Priority::Low => 3,
                    };
                    priority_order(&a.priority).cmp(&priority_order(&b.priority))
                }
                RequirementSortField::Category => a.category.cmp(&b.category),
                RequirementSortField::Author => a.author.cmp(&b.author),
                RequirementSortField::Created => a.created.cmp(&b.created),
            };

            match sort_dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }

    /// Get count of requirements matching a filter
    pub fn count(&self, filter: &RequirementFilter) -> ServiceResult<usize> {
        let requirements = self.load_all()?;
        Ok(requirements
            .iter()
            .filter(|req| self.matches_filter(req, filter))
            .count())
    }

    /// Get statistics about requirements
    ///
    /// Uses the cache's SQL-based stats for fast aggregation without loading entities.
    pub fn stats(&self) -> ServiceResult<RequirementStats> {
        // Use cache for fast SQL-based stats
        let cached = self.cache.requirement_stats();

        Ok(RequirementStats {
            total: cached.total,
            inputs: cached.inputs,
            outputs: cached.outputs,
            unverified: cached.unverified,
            orphaned: cached.orphaned,
            by_status: StatusCounts {
                draft: cached.by_status.draft,
                review: cached.by_status.review,
                approved: cached.by_status.approved,
                released: cached.by_status.released,
                obsolete: cached.by_status.obsolete,
            },
        })
    }

    /// Get statistics by loading all entities (slower, but provides full data)
    ///
    /// Use this when you need accurate link-based stats that require full entity data.
    pub fn stats_full(&self) -> ServiceResult<RequirementStats> {
        let requirements = self.load_all()?;

        let mut stats = RequirementStats::default();
        stats.total = requirements.len();

        for req in &requirements {
            match req.req_type {
                RequirementType::Input => stats.inputs += 1,
                RequirementType::Output => stats.outputs += 1,
            }

            match req.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            if req.links.verified_by.is_empty() {
                stats.unverified += 1;
            }

            if req.links.satisfied_by.is_empty() && req.links.verified_by.is_empty() {
                stats.orphaned += 1;
            }
        }

        Ok(stats)
    }
}

// Implement ListableService trait for generic CLI list operations
impl<'a> super::common::ListableService<
    Requirement,
    CachedRequirement,
    RequirementFilter,
    RequirementSortField,
> for RequirementService<'a> {
    fn list(
        &self,
        filter: &RequirementFilter,
        sort_by: RequirementSortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Requirement>> {
        RequirementService::list(self, filter, sort_by, sort_dir)
    }

    fn list_cached(&self, filter: &RequirementFilter) -> ServiceResult<ListResult<CachedRequirement>> {
        RequirementService::list_cached(self, filter)
    }
}

/// Statistics about requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequirementStats {
    pub total: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub unverified: usize,
    pub orphaned: usize,
    pub by_status: StatusCounts,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();

        // Initialize project structure
        fs::create_dir_all(tmp.path().join(".tdt")).unwrap();
        fs::create_dir_all(tmp.path().join("requirements/inputs")).unwrap();
        fs::create_dir_all(tmp.path().join("requirements/outputs")).unwrap();

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
    fn test_create_requirement() {
        let (_tmp, project, cache) = setup_test_project();
        let service = RequirementService::new(&project, &cache);

        let input = CreateRequirement {
            req_type: RequirementType::Input,
            title: "Test Requirement".into(),
            text: "This is a test requirement.".into(),
            author: "Test Author".into(),
            level: Level::System,
            priority: Priority::High,
            ..Default::default()
        };

        let req = service.create(input).unwrap();

        assert_eq!(req.title, "Test Requirement");
        assert_eq!(req.req_type, RequirementType::Input);
        assert_eq!(req.priority, Priority::High);
        assert_eq!(req.status, Status::Draft);
    }

    #[test]
    fn test_get_requirement() {
        let (_tmp, project, cache) = setup_test_project();
        let service = RequirementService::new(&project, &cache);

        // Create a requirement
        let created = service
            .create(CreateRequirement {
                req_type: RequirementType::Input,
                title: "Find Me".into(),
                text: "Can you find this?".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // Get it back
        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Find Me");
    }

    #[test]
    fn test_update_requirement() {
        let (_tmp, project, cache) = setup_test_project();
        let service = RequirementService::new(&project, &cache);

        // Create
        let created = service
            .create(CreateRequirement {
                req_type: RequirementType::Input,
                title: "Original Title".into(),
                text: "Original text.".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // Update
        let updated = service
            .update(
                &created.id.to_string(),
                UpdateRequirement {
                    title: Some("Updated Title".into()),
                    priority: Some(Priority::Critical),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.priority, Priority::Critical);
        assert_eq!(updated.revision, 2); // Incremented
    }

    #[test]
    fn test_delete_requirement() {
        let (_tmp, project, cache) = setup_test_project();
        let service = RequirementService::new(&project, &cache);

        // Create
        let created = service
            .create(CreateRequirement {
                req_type: RequirementType::Input,
                title: "Delete Me".into(),
                text: "I will be deleted.".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // Delete
        service.delete(&created.id.to_string(), false).unwrap();

        // Verify gone
        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup_test_project();
        let service = RequirementService::new(&project, &cache);

        // Create multiple requirements
        let req1 = service
            .create(CreateRequirement {
                req_type: RequirementType::Input,
                title: "Input 1".into(),
                text: "First input".into(),
                author: "Test".into(),
                priority: Priority::High,
                ..Default::default()
            })
            .unwrap();

        let req2 = service
            .create(CreateRequirement {
                req_type: RequirementType::Output,
                title: "Output 1".into(),
                text: "First output".into(),
                author: "Test".into(),
                priority: Priority::Low,
                ..Default::default()
            })
            .unwrap();

        // Verify files exist
        let input_path = service.get_file_path(&req1.id, RequirementType::Input);
        let output_path = service.get_file_path(&req2.id, RequirementType::Output);
        assert!(input_path.exists(), "Input file should exist at {:?}", input_path);
        assert!(output_path.exists(), "Output file should exist at {:?}", output_path);

        // Sync cache to pick up newly created files
        let cache = EntityCache::open(&project).unwrap();

        // Debug: check cache directly
        let cached_reqs = cache.list_requirements(None, None, None, None, None, None, None);
        assert_eq!(cached_reqs.len(), 2, "Cache should have 2 requirements, found {}", cached_reqs.len());

        let service = RequirementService::new(&project, &cache);

        // List all
        let all = service
            .list(
                &RequirementFilter::default(),
                RequirementSortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(all.items.len(), 2);

        // List only inputs
        let inputs = service
            .list(
                &RequirementFilter::inputs(),
                RequirementSortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(inputs.items.len(), 1);
        assert_eq!(inputs.items[0].title, "Input 1");
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup_test_project();
        let service = RequirementService::new(&project, &cache);

        // Create some requirements
        service
            .create(CreateRequirement {
                req_type: RequirementType::Input,
                title: "Input".into(),
                text: "Text".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateRequirement {
                req_type: RequirementType::Output,
                title: "Output".into(),
                text: "Text".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // Sync cache to pick up newly created files
        let cache = EntityCache::open(&project).unwrap();
        let service = RequirementService::new(&project, &cache);

        let stats = service.stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.inputs, 1);
        assert_eq!(stats.outputs, 1);
        assert_eq!(stats.by_status.draft, 2);
    }
}
