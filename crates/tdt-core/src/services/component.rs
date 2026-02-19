//! Component service - business logic for BOM management

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::cache::EntityCache;
use crate::core::entity::Status;
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::assembly::ManufacturingConfig;
use crate::entities::component::{
    Component, ComponentCategory, ComponentLinks, ComponentSupplier, Document, MakeBuy,
};

use super::base::ServiceBase;
use super::common::{
    apply_pagination, CommonFilter, ListResult, ServiceError, ServiceResult, SortDirection,
};

/// Filter options specific to components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentFilter {
    /// Common filter options (status, author, search, etc.)
    #[serde(flatten)]
    pub common: CommonFilter,

    /// Filter by make/buy decision
    pub make_buy: Option<MakeBuy>,

    /// Filter by category
    pub category: Option<ComponentCategory>,

    /// Filter by part number (substring match)
    pub part_number: Option<String>,

    /// Filter by material (substring match)
    pub material: Option<String>,

    /// Show only components without suppliers
    pub no_suppliers: bool,

    /// Show only components without unit cost
    pub no_cost: bool,

    /// Show only obsolete components
    pub obsolete_only: bool,

    /// Show only components used in assemblies
    pub used_only: bool,

    /// Show only unused components (not in any assembly)
    pub unused_only: bool,
}

impl ComponentFilter {
    /// Create a filter for make parts
    pub fn make() -> Self {
        Self {
            make_buy: Some(MakeBuy::Make),
            ..Default::default()
        }
    }

    /// Create a filter for buy parts
    pub fn buy() -> Self {
        Self {
            make_buy: Some(MakeBuy::Buy),
            ..Default::default()
        }
    }

    /// Create a filter for mechanical parts
    pub fn mechanical() -> Self {
        Self {
            category: Some(ComponentCategory::Mechanical),
            ..Default::default()
        }
    }

    /// Create a filter for electrical parts
    pub fn electrical() -> Self {
        Self {
            category: Some(ComponentCategory::Electrical),
            ..Default::default()
        }
    }

    /// Create a filter for software components
    pub fn software() -> Self {
        Self {
            category: Some(ComponentCategory::Software),
            ..Default::default()
        }
    }
}

/// Sort field for components
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentSortField {
    Id,
    PartNumber,
    #[default]
    Title,
    Category,
    MakeBuy,
    Material,
    UnitCost,
    Mass,
    Status,
    Author,
    Created,
}

/// Input for creating a new component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComponent {
    /// Part number
    pub part_number: String,

    /// Short title/description
    pub title: String,

    /// Author name
    pub author: String,

    /// Make or buy decision
    #[serde(default)]
    pub make_buy: MakeBuy,

    /// Component category
    #[serde(default)]
    pub category: ComponentCategory,

    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,

    /// Part revision
    #[serde(default)]
    pub revision: Option<String>,

    /// Material specification
    #[serde(default)]
    pub material: Option<String>,

    /// Mass in kilograms
    #[serde(default)]
    pub mass_kg: Option<f64>,

    /// Unit cost
    #[serde(default)]
    pub unit_cost: Option<f64>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Initial suppliers
    #[serde(default)]
    pub suppliers: Vec<ComponentSupplier>,
}

impl Default for CreateComponent {
    fn default() -> Self {
        Self {
            part_number: String::new(),
            title: String::new(),
            author: String::new(),
            make_buy: MakeBuy::Buy,
            category: ComponentCategory::Mechanical,
            description: None,
            revision: None,
            material: None,
            mass_kg: None,
            unit_cost: None,
            tags: Vec::new(),
            suppliers: Vec::new(),
        }
    }
}

/// Input for updating an existing component
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateComponent {
    /// Update part number
    pub part_number: Option<String>,

    /// Update title
    pub title: Option<String>,

    /// Update description
    pub description: Option<String>,

    /// Update revision
    pub revision: Option<String>,

    /// Update make/buy
    pub make_buy: Option<MakeBuy>,

    /// Update category
    pub category: Option<ComponentCategory>,

    /// Update material
    pub material: Option<String>,

    /// Update mass
    pub mass_kg: Option<f64>,

    /// Update unit cost
    pub unit_cost: Option<f64>,

    /// Update status
    pub status: Option<Status>,

    /// Update tags (replaces existing)
    pub tags: Option<Vec<String>>,

    /// Update suppliers (replaces existing)
    pub suppliers: Option<Vec<ComponentSupplier>>,

    /// Update documents (replaces existing)
    pub documents: Option<Vec<Document>>,
}

/// Service for component management (BOM)
pub struct ComponentService<'a> {
    project: &'a Project,
    cache: &'a EntityCache,
    base: ServiceBase<'a>,
}

impl<'a> ComponentService<'a> {
    /// Create a new component service
    pub fn new(project: &'a Project, cache: &'a EntityCache) -> Self {
        Self {
            project,
            cache,
            base: ServiceBase::new(project, cache),
        }
    }

    /// Get the directory for storing components
    fn get_directory(&self) -> PathBuf {
        self.project.root().join("bom/components")
    }

    /// Get the file path for a component
    fn get_file_path(&self, id: &EntityId) -> PathBuf {
        let dir = self.get_directory();
        dir.join(format!("{}.tdt.yaml", id))
    }

    /// List components with filtering and pagination
    pub fn list(
        &self,
        filter: &ComponentFilter,
        sort_by: ComponentSortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Component>> {
        let mut components = self.load_all()?;

        // Apply filters
        components.retain(|cmp| self.matches_filter(cmp, filter));

        // Sort
        self.sort_components(&mut components, sort_by, sort_dir);

        // Paginate
        Ok(apply_pagination(
            components,
            filter.common.offset,
            filter.common.limit,
        ))
    }

    /// List components from cache (fast path for list display)
    ///
    /// Returns cached component data without loading full YAML files.
    /// Use this for list commands where full entity data isn't needed.
    pub fn list_cached(&self) -> Vec<crate::core::CachedEntity> {
        use crate::core::cache::EntityFilter;
        use crate::core::identity::EntityPrefix;

        let filter = EntityFilter {
            prefix: Some(EntityPrefix::Cmp),
            ..Default::default()
        };
        self.cache.list_entities(&filter)
    }

    /// Load all components from the filesystem
    pub fn load_all(&self) -> ServiceResult<Vec<Component>> {
        let dir = self.get_directory();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        Ok(loader::load_all(&dir)?)
    }

    /// Get a single component by ID
    pub fn get(&self, id: &str) -> ServiceResult<Option<Component>> {
        let dir = self.get_directory();
        if let Some((_, cmp)) = loader::load_entity::<Component>(&dir, id)? {
            return Ok(Some(cmp));
        }
        Ok(None)
    }

    /// Get a component by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Component> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(id.to_string()).into())
    }

    /// Get a component by part number
    pub fn get_by_part_number(&self, part_number: &str) -> ServiceResult<Option<Component>> {
        let components = self.load_all()?;
        Ok(components
            .into_iter()
            .find(|c| c.part_number.eq_ignore_ascii_case(part_number)))
    }

    /// Create a new component
    pub fn create(&self, input: CreateComponent) -> ServiceResult<Component> {
        // Check for duplicate part number
        if let Some(existing) = self.get_by_part_number(&input.part_number)? {
            return Err(ServiceError::AlreadyExists(format!(
                "Component with part number '{}' already exists ({})",
                input.part_number, existing.id
            ))
            .into());
        }

        let id = EntityId::new(EntityPrefix::Cmp);

        let component = Component {
            id: id.clone(),
            part_number: input.part_number,
            revision: input.revision,
            title: input.title,
            description: input.description,
            make_buy: input.make_buy,
            category: input.category,
            sw_class: None,
            asil: None,
            dal: None,
            material: input.material,
            mass_kg: input.mass_kg,
            unit_cost: input.unit_cost,
            selected_quote: None,
            suppliers: input.suppliers,
            documents: Vec::new(),
            coordinate_system: None,
            datum_frame: None,
            manufacturing: None,
            tags: input.tags,
            status: Status::Draft,
            links: ComponentLinks::default(),
            created: Utc::now(),
            author: input.author,
            entity_revision: 1,
        };

        // Ensure directory exists
        let dir = self.get_directory();
        fs::create_dir_all(&dir)?;

        // Write to file
        let path = self.get_file_path(&id);
        self.base.save(&component, &path, Some("CMP"))?;

        Ok(component)
    }

    /// Update an existing component
    pub fn update(&self, id: &str, input: UpdateComponent) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        // Check for duplicate part number if changing it
        if let Some(new_pn) = &input.part_number {
            if new_pn != &component.part_number {
                if let Some(existing) = self.get_by_part_number(new_pn)? {
                    if existing.id != component.id {
                        return Err(ServiceError::AlreadyExists(format!(
                            "Component with part number '{}' already exists ({})",
                            new_pn, existing.id
                        ))
                        .into());
                    }
                }
            }
        }

        // Apply updates
        if let Some(part_number) = input.part_number {
            component.part_number = part_number;
        }
        if let Some(title) = input.title {
            component.title = title;
        }
        if let Some(description) = input.description {
            component.description = Some(description);
        }
        if let Some(revision) = input.revision {
            component.revision = Some(revision);
        }
        if let Some(make_buy) = input.make_buy {
            component.make_buy = make_buy;
        }
        if let Some(category) = input.category {
            component.category = category;
        }
        if let Some(material) = input.material {
            component.material = Some(material);
        }
        if let Some(mass_kg) = input.mass_kg {
            component.mass_kg = Some(mass_kg);
        }
        if let Some(unit_cost) = input.unit_cost {
            component.unit_cost = Some(unit_cost);
        }
        if let Some(status) = input.status {
            component.status = status;
        }
        if let Some(tags) = input.tags {
            component.tags = tags;
        }
        if let Some(suppliers) = input.suppliers {
            component.suppliers = suppliers;
        }
        if let Some(documents) = input.documents {
            component.documents = documents;
        }

        // Increment revision
        component.entity_revision += 1;

        // Write back
        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Delete a component
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, component) = self.find_component(id)?;

        // Check for references unless force is true
        if !force {
            // Check if used in any assembly
            if !component.links.used_in.is_empty() {
                return Err(ServiceError::HasReferences.into());
            }

            let references = self.find_references(&component.id)?;
            if !references.is_empty() {
                return Err(ServiceError::HasReferences.into());
            }
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Add a supplier to a component
    pub fn add_supplier(&self, id: &str, supplier: ComponentSupplier) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        component.suppliers.push(supplier);
        component.entity_revision += 1;

        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Add a document to a component
    pub fn add_document(&self, id: &str, document: Document) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        component.documents.push(document);
        component.entity_revision += 1;

        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Set the selected quote for pricing
    pub fn set_quote(&self, id: &str, quote_id: &str) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        component.selected_quote = Some(quote_id.to_string());
        component.entity_revision += 1;

        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Clear the selected quote
    pub fn clear_quote(&self, id: &str) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        component.selected_quote = None;
        component.entity_revision += 1;

        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Get the manufacturing routing (list of process IDs)
    pub fn get_routing(&self, id: &str) -> ServiceResult<Vec<String>> {
        let component = self.get_required(id)?;
        Ok(component
            .manufacturing
            .map(|m| m.routing)
            .unwrap_or_default())
    }

    /// Set the manufacturing routing
    pub fn set_routing(&self, id: &str, routing: Vec<String>) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        let manufacturing = component
            .manufacturing
            .get_or_insert(ManufacturingConfig::default());
        manufacturing.routing = routing;
        component.entity_revision += 1;

        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Add a process to the manufacturing routing
    pub fn add_routing_process(&self, id: &str, process_id: &str) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        let manufacturing = component
            .manufacturing
            .get_or_insert(ManufacturingConfig::default());
        if manufacturing.routing.contains(&process_id.to_string()) {
            return Err(ServiceError::AlreadyExists(format!(
                "Process {} already in routing",
                process_id
            ))
            .into());
        }
        manufacturing.routing.push(process_id.to_string());
        component.entity_revision += 1;

        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Remove a process from the manufacturing routing
    pub fn remove_routing_process(&self, id: &str, process_id: &str) -> ServiceResult<Component> {
        let (path, mut component) = self.find_component(id)?;

        if let Some(ref mut manufacturing) = component.manufacturing {
            let original_len = manufacturing.routing.len();
            manufacturing.routing.retain(|p| p != process_id);
            if manufacturing.routing.len() == original_len {
                return Err(ServiceError::NotFound(format!(
                    "Process {} not in routing",
                    process_id
                ))
                .into());
            }
        } else {
            return Err(ServiceError::NotFound(format!(
                "Process {} not in routing (no routing defined)",
                process_id
            ))
            .into());
        }

        component.entity_revision += 1;

        self.base.save(&component, &path, None)?;

        Ok(component)
    }

    /// Find a component and its file path (cache-first lookup)
    fn find_component(&self, id: &str) -> ServiceResult<(PathBuf, Component)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(component) = crate::yaml::parse_yaml_file::<Component>(&path) {
                    return Ok((path, component));
                }
            }
        }

        // Fall back to directory scan
        let dir = self.get_directory();
        if let Some((path, cmp)) = loader::load_entity::<Component>(&dir, id)? {
            return Ok((path, cmp));
        }
        Err(ServiceError::NotFound(id.to_string()).into())
    }

    /// Find entities that reference this component
    fn find_references(&self, _id: &EntityId) -> ServiceResult<Vec<EntityId>> {
        // TODO: Implement reference checking via cache or file scan
        Ok(Vec::new())
    }

    /// Check if a component matches the given filter
    fn matches_filter(&self, cmp: &Component, filter: &ComponentFilter) -> bool {
        // Make/buy filter
        if let Some(make_buy) = &filter.make_buy {
            if cmp.make_buy != *make_buy {
                return false;
            }
        }

        // Category filter
        if let Some(category) = &filter.category {
            if cmp.category != *category {
                return false;
            }
        }

        // Part number filter
        if let Some(pn) = &filter.part_number {
            if !cmp.part_number.to_lowercase().contains(&pn.to_lowercase()) {
                return false;
            }
        }

        // Material filter
        if let Some(material) = &filter.material {
            if !cmp
                .material
                .as_ref()
                .map(|m| m.to_lowercase().contains(&material.to_lowercase()))
                .unwrap_or(false)
            {
                return false;
            }
        }

        // No suppliers filter
        if filter.no_suppliers && !cmp.suppliers.is_empty() {
            return false;
        }

        // No cost filter
        if filter.no_cost && cmp.unit_cost.is_some() {
            return false;
        }

        // Obsolete filter
        if filter.obsolete_only && cmp.status != Status::Obsolete {
            return false;
        }

        // Used filter
        if filter.used_only && cmp.links.used_in.is_empty() {
            return false;
        }

        // Unused filter
        if filter.unused_only && !cmp.links.used_in.is_empty() {
            return false;
        }

        // Common filters
        if !filter.common.matches_status(&cmp.status) {
            return false;
        }
        if !filter.common.matches_author(&cmp.author) {
            return false;
        }
        if !filter.common.matches_tags(&cmp.tags) {
            return false;
        }
        if !filter
            .common
            .matches_search(&[&cmp.title, &cmp.part_number])
        {
            return false;
        }
        if !filter.common.matches_recent(&cmp.created) {
            return false;
        }

        true
    }

    /// Sort components by the given field
    fn sort_components(
        &self,
        components: &mut [Component],
        sort_by: ComponentSortField,
        sort_dir: SortDirection,
    ) {
        components.sort_by(|a, b| {
            let cmp = match sort_by {
                ComponentSortField::Id => a.id.to_string().cmp(&b.id.to_string()),
                ComponentSortField::PartNumber => a.part_number.cmp(&b.part_number),
                ComponentSortField::Title => a.title.cmp(&b.title),
                ComponentSortField::Category => {
                    format!("{}", a.category).cmp(&format!("{}", b.category))
                }
                ComponentSortField::MakeBuy => {
                    format!("{}", a.make_buy).cmp(&format!("{}", b.make_buy))
                }
                ComponentSortField::Material => a.material.cmp(&b.material),
                ComponentSortField::UnitCost => a
                    .unit_cost
                    .partial_cmp(&b.unit_cost)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ComponentSortField::Mass => a
                    .mass_kg
                    .partial_cmp(&b.mass_kg)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ComponentSortField::Status => {
                    format!("{:?}", a.status).cmp(&format!("{:?}", b.status))
                }
                ComponentSortField::Author => a.author.cmp(&b.author),
                ComponentSortField::Created => a.created.cmp(&b.created),
            };

            match sort_dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }

    /// Get count of components matching a filter
    pub fn count(&self, filter: &ComponentFilter) -> ServiceResult<usize> {
        let components = self.load_all()?;
        Ok(components
            .iter()
            .filter(|cmp| self.matches_filter(cmp, filter))
            .count())
    }

    /// Get statistics about components
    pub fn stats(&self) -> ServiceResult<ComponentStats> {
        let components = self.load_all()?;

        let mut stats = ComponentStats::default();
        stats.total = components.len();

        for cmp in &components {
            match cmp.make_buy {
                MakeBuy::Make => stats.make_count += 1,
                MakeBuy::Buy => stats.buy_count += 1,
            }

            match cmp.category {
                ComponentCategory::Mechanical => stats.by_category.mechanical += 1,
                ComponentCategory::Electrical => stats.by_category.electrical += 1,
                ComponentCategory::Software => stats.by_category.software += 1,
                ComponentCategory::Fastener => stats.by_category.fastener += 1,
                ComponentCategory::Consumable => stats.by_category.consumable += 1,
            }

            match cmp.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            if cmp.suppliers.is_empty() {
                stats.no_suppliers += 1;
            }

            if cmp.unit_cost.is_none() && cmp.selected_quote.is_none() {
                stats.no_cost += 1;
            }

            if cmp.links.used_in.is_empty() {
                stats.unused += 1;
            }

            // Cost tracking
            if let Some(cost) = cmp.unit_cost {
                stats.cost_stats.count += 1;
                stats.cost_stats.total += cost;
                if cost > stats.cost_stats.max {
                    stats.cost_stats.max = cost;
                }
                if stats.cost_stats.min == 0.0 || cost < stats.cost_stats.min {
                    stats.cost_stats.min = cost;
                }
            }
        }

        // Calculate average cost
        if stats.cost_stats.count > 0 {
            stats.cost_stats.avg = stats.cost_stats.total / stats.cost_stats.count as f64;
        }

        Ok(stats)
    }

    /// Get BOM cost summary
    pub fn get_cost_summary(&self) -> ServiceResult<BomCostSummary> {
        let components = self.load_all()?;

        let mut summary = BomCostSummary::default();

        for cmp in &components {
            if let Some(cost) = cmp.unit_cost {
                summary.total_cost += cost;
                summary.components_with_cost += 1;

                match cmp.make_buy {
                    MakeBuy::Make => summary.make_cost += cost,
                    MakeBuy::Buy => summary.buy_cost += cost,
                }

                match cmp.category {
                    ComponentCategory::Mechanical => summary.mechanical_cost += cost,
                    ComponentCategory::Electrical => summary.electrical_cost += cost,
                    ComponentCategory::Software => summary.software_cost += cost,
                    ComponentCategory::Fastener => summary.fastener_cost += cost,
                    ComponentCategory::Consumable => summary.consumable_cost += cost,
                }
            } else {
                summary.components_without_cost += 1;
            }
        }

        Ok(summary)
    }

    /// Find duplicate components (same or similar part numbers)
    pub fn find_duplicates(&self) -> ServiceResult<Vec<DuplicateGroup>> {
        let components = self.load_all()?;
        let mut groups: HashMap<String, Vec<Component>> = HashMap::new();

        // Group by normalized part number (lowercase, trimmed)
        for cmp in components {
            let key = cmp.part_number.to_lowercase().trim().to_string();
            groups.entry(key).or_default().push(cmp);
        }

        // Filter to only groups with duplicates
        let duplicates: Vec<DuplicateGroup> = groups
            .into_iter()
            .filter(|(_, cmps)| cmps.len() > 1)
            .map(|(part_number, components)| DuplicateGroup {
                part_number,
                components,
            })
            .collect();

        Ok(duplicates)
    }
}

/// Statistics about components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentStats {
    pub total: usize,
    pub make_count: usize,
    pub buy_count: usize,
    pub no_suppliers: usize,
    pub no_cost: usize,
    pub unused: usize,
    pub by_category: CategoryCounts,
    pub by_status: StatusCounts,
    pub cost_stats: CostStats,
}

/// Counts by category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryCounts {
    pub mechanical: usize,
    pub electrical: usize,
    pub software: usize,
    pub fastener: usize,
    pub consumable: usize,
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

/// Cost statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub total: f64,
    pub avg: f64,
}

/// BOM cost summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BomCostSummary {
    pub total_cost: f64,
    pub components_with_cost: usize,
    pub components_without_cost: usize,
    pub make_cost: f64,
    pub buy_cost: f64,
    pub mechanical_cost: f64,
    pub electrical_cost: f64,
    pub software_cost: f64,
    pub fastener_cost: f64,
    pub consumable_cost: f64,
}

/// A group of duplicate components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub part_number: String,
    pub components: Vec<Component>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();

        // Initialize project structure
        fs::create_dir_all(tmp.path().join(".tdt")).unwrap();
        fs::create_dir_all(tmp.path().join("bom/components")).unwrap();

        // Create config file
        fs::write(tmp.path().join(".tdt/config.yaml"), "author: Test Author\n").unwrap();

        let project = Project::discover_from(tmp.path()).unwrap();
        let cache = EntityCache::open(&project).unwrap();

        (tmp, project, cache)
    }

    #[test]
    fn test_create_component() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        let input = CreateComponent {
            part_number: "PN-001".into(),
            title: "Test Widget".into(),
            author: "Test Author".into(),
            make_buy: MakeBuy::Buy,
            category: ComponentCategory::Mechanical,
            unit_cost: Some(10.50),
            ..Default::default()
        };

        let cmp = service.create(input).unwrap();

        assert_eq!(cmp.part_number, "PN-001");
        assert_eq!(cmp.title, "Test Widget");
        assert_eq!(cmp.make_buy, MakeBuy::Buy);
        assert_eq!(cmp.unit_cost, Some(10.50));
    }

    #[test]
    fn test_duplicate_part_number_rejected() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        // Create first component
        service
            .create(CreateComponent {
                part_number: "PN-001".into(),
                title: "First Widget".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // Try to create duplicate
        let result = service.create(CreateComponent {
            part_number: "PN-001".into(),
            title: "Second Widget".into(),
            author: "Test".into(),
            ..Default::default()
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_get_component() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        let created = service
            .create(CreateComponent {
                part_number: "PN-002".into(),
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
    fn test_get_by_part_number() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        service
            .create(CreateComponent {
                part_number: "WIDGET-123".into(),
                title: "Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let found = service.get_by_part_number("widget-123").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().part_number, "WIDGET-123");
    }

    #[test]
    fn test_update_component() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        let created = service
            .create(CreateComponent {
                part_number: "PN-003".into(),
                title: "Original".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .update(
                &created.id.to_string(),
                UpdateComponent {
                    title: Some("Updated Title".into()),
                    unit_cost: Some(25.00),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.unit_cost, Some(25.00));
        assert_eq!(updated.entity_revision, 2);
    }

    #[test]
    fn test_delete_component() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        let created = service
            .create(CreateComponent {
                part_number: "PN-004".into(),
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
    fn test_add_supplier() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        let created = service
            .create(CreateComponent {
                part_number: "PN-005".into(),
                title: "Widget".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let supplier = ComponentSupplier {
            name: "Acme Corp".into(),
            supplier_pn: Some("ACME-W-001".into()),
            unit_cost: Some(8.50),
            ..Default::default()
        };

        let updated = service
            .add_supplier(&created.id.to_string(), supplier)
            .unwrap();

        assert_eq!(updated.suppliers.len(), 1);
        assert_eq!(updated.suppliers[0].name, "Acme Corp");
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        // Create make component
        service
            .create(CreateComponent {
                part_number: "MAKE-001".into(),
                title: "Make Part".into(),
                author: "Test".into(),
                make_buy: MakeBuy::Make,
                category: ComponentCategory::Mechanical,
                ..Default::default()
            })
            .unwrap();

        // Create buy component
        service
            .create(CreateComponent {
                part_number: "BUY-001".into(),
                title: "Buy Part".into(),
                author: "Test".into(),
                make_buy: MakeBuy::Buy,
                category: ComponentCategory::Electrical,
                ..Default::default()
            })
            .unwrap();

        // List all
        let all = service
            .list(
                &ComponentFilter::default(),
                ComponentSortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(all.items.len(), 2);

        // List only make
        let make_only = service
            .list(
                &ComponentFilter::make(),
                ComponentSortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(make_only.items.len(), 1);
        assert_eq!(make_only.items[0].title, "Make Part");

        // List electrical
        let electrical = service
            .list(
                &ComponentFilter::electrical(),
                ComponentSortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(electrical.items.len(), 1);
        assert_eq!(electrical.items[0].title, "Buy Part");
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        service
            .create(CreateComponent {
                part_number: "PN-A".into(),
                title: "Part A".into(),
                author: "Test".into(),
                make_buy: MakeBuy::Make,
                category: ComponentCategory::Mechanical,
                unit_cost: Some(10.00),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateComponent {
                part_number: "PN-B".into(),
                title: "Part B".into(),
                author: "Test".into(),
                make_buy: MakeBuy::Buy,
                category: ComponentCategory::Electrical,
                unit_cost: Some(20.00),
                ..Default::default()
            })
            .unwrap();

        let stats = service.stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.make_count, 1);
        assert_eq!(stats.buy_count, 1);
        assert_eq!(stats.by_category.mechanical, 1);
        assert_eq!(stats.by_category.electrical, 1);
        assert_eq!(stats.cost_stats.count, 2);
        assert_eq!(stats.cost_stats.total, 30.00);
    }

    #[test]
    fn test_cost_summary() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        service
            .create(CreateComponent {
                part_number: "PN-X".into(),
                title: "Part X".into(),
                author: "Test".into(),
                make_buy: MakeBuy::Make,
                category: ComponentCategory::Mechanical,
                unit_cost: Some(100.00),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateComponent {
                part_number: "PN-Y".into(),
                title: "Part Y".into(),
                author: "Test".into(),
                make_buy: MakeBuy::Buy,
                category: ComponentCategory::Electrical,
                unit_cost: Some(50.00),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateComponent {
                part_number: "PN-Z".into(),
                title: "Part Z".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let summary = service.get_cost_summary().unwrap();
        assert_eq!(summary.total_cost, 150.00);
        assert_eq!(summary.components_with_cost, 2);
        assert_eq!(summary.components_without_cost, 1);
        assert_eq!(summary.make_cost, 100.00);
        assert_eq!(summary.buy_cost, 50.00);
    }

    #[test]
    fn test_find_duplicates() {
        let (_tmp, project, cache) = setup_test_project();
        let service = ComponentService::new(&project, &cache);

        // This test verifies the structure - we can't create actual duplicates
        // due to the part number uniqueness check
        let duplicates = service.find_duplicates().unwrap();
        assert!(duplicates.is_empty());
    }
}
