//! Assembly service - business logic for BOM hierarchy management
//!
//! Provides CRUD operations, BOM tree traversal, and cost/mass calculations.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::core::cache::EntityCache;
use crate::core::entity::Status;
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::assembly::{Assembly, AssemblyLinks, BomItem, Document, ManufacturingConfig};
use crate::entities::component::Component;
use crate::entities::quote::Quote;

use super::common::{
    apply_pagination, CommonFilter, ListResult, ServiceError, ServiceResult, SortDirection,
};

/// Filter options specific to assemblies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssemblyFilter {
    /// Common filter options (status, author, search, etc.)
    #[serde(flatten)]
    pub common: CommonFilter,

    /// Filter by part number (substring match)
    pub part_number: Option<String>,

    /// Show only assemblies with no components
    pub empty_bom: bool,

    /// Show only assemblies with subassemblies
    pub has_subassemblies: bool,

    /// Show only top-level assemblies (no parent)
    pub top_level_only: bool,

    /// Show only sub-assemblies (have parent)
    pub sub_only: bool,
}

impl AssemblyFilter {
    /// Create a filter for top-level assemblies only
    pub fn top_level() -> Self {
        Self {
            top_level_only: true,
            ..Default::default()
        }
    }

    /// Create a filter for sub-assemblies only
    pub fn sub_assemblies() -> Self {
        Self {
            sub_only: true,
            ..Default::default()
        }
    }
}

/// Sort field for assemblies
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssemblySortField {
    Id,
    PartNumber,
    #[default]
    Title,
    BomCount,
    Status,
    Author,
    Created,
}

/// Input for creating a new assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssembly {
    /// Assembly part number
    pub part_number: String,

    /// Assembly title
    pub title: String,

    /// Author name
    pub author: String,

    /// Part revision
    #[serde(default)]
    pub revision: Option<String>,

    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,

    /// Initial BOM items
    #[serde(default)]
    pub bom: Vec<BomItem>,

    /// Sub-assembly references
    #[serde(default)]
    pub subassemblies: Vec<String>,

    /// Classification tags
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for CreateAssembly {
    fn default() -> Self {
        Self {
            part_number: String::new(),
            title: String::new(),
            author: String::new(),
            revision: None,
            description: None,
            bom: Vec::new(),
            subassemblies: Vec::new(),
            tags: Vec::new(),
        }
    }
}

/// Input for updating an existing assembly
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateAssembly {
    /// Update part number
    pub part_number: Option<String>,

    /// Update title
    pub title: Option<String>,

    /// Update revision
    pub revision: Option<String>,

    /// Update description
    pub description: Option<String>,

    /// Replace BOM
    pub bom: Option<Vec<BomItem>>,

    /// Replace subassemblies
    pub subassemblies: Option<Vec<String>>,

    /// Update status
    pub status: Option<Status>,

    /// Replace tags
    pub tags: Option<Vec<String>>,

    /// Replace documents
    pub documents: Option<Vec<Document>>,

    /// Update manufacturing config
    pub manufacturing: Option<ManufacturingConfig>,
}

/// A node in the BOM tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomNode {
    /// Entity ID (CMP or ASM)
    pub id: String,

    /// Title/name
    pub title: String,

    /// Part number
    pub part_number: String,

    /// Whether this is an assembly (true) or component (false)
    pub is_assembly: bool,

    /// Quantity at this level
    pub quantity: u32,

    /// Unit cost (if available)
    pub unit_cost: Option<f64>,

    /// Unit mass in kg (if available)
    pub mass_kg: Option<f64>,

    /// Extended cost (quantity * unit_cost)
    pub extended_cost: Option<f64>,

    /// Extended mass (quantity * mass_kg)
    pub extended_mass: Option<f64>,

    /// Children (for assemblies)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<BomNode>,
}

/// Result of BOM cost calculation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BomCostResult {
    /// Total cost including all components and sub-assemblies
    pub total_cost: f64,

    /// Total NRE costs (tooling, setup, etc.)
    pub total_nre: f64,

    /// Number of components with pricing
    pub components_with_cost: usize,

    /// Number of components without pricing
    pub components_without_cost: usize,

    /// List of component IDs missing cost data
    pub missing_cost: Vec<String>,
}

/// Result of BOM mass calculation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BomMassResult {
    /// Total mass in kg
    pub total_mass_kg: f64,

    /// Number of components with mass data
    pub components_with_mass: usize,

    /// Number of components without mass data
    pub components_without_mass: usize,

    /// List of component IDs missing mass data
    pub missing_mass: Vec<String>,
}

/// Statistics about assemblies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssemblyStats {
    pub total: usize,
    pub top_level: usize,
    pub sub_assemblies: usize,
    pub empty_bom: usize,
    pub total_bom_items: usize,
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

/// Service for assembly management (BOM hierarchy)
pub struct AssemblyService<'a> {
    project: &'a Project,
    cache: &'a EntityCache,
}

impl<'a> AssemblyService<'a> {
    /// Create a new assembly service
    pub fn new(project: &'a Project, cache: &'a EntityCache) -> Self {
        Self { project, cache }
    }

    /// Get the directory for storing assemblies
    fn get_directory(&self) -> PathBuf {
        self.project.root().join("bom/assemblies")
    }

    /// Get the file path for an assembly
    fn get_file_path(&self, id: &EntityId) -> PathBuf {
        self.get_directory().join(format!("{}.tdt.yaml", id))
    }

    /// Get the components directory
    fn components_dir(&self) -> PathBuf {
        self.project.root().join("bom/components")
    }

    /// Get the quotes directory
    fn quotes_dir(&self) -> PathBuf {
        self.project.root().join("bom/quotes")
    }

    /// List assemblies with filtering and pagination
    pub fn list(
        &self,
        filter: &AssemblyFilter,
        sort_by: AssemblySortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Assembly>> {
        let mut assemblies = self.load_all()?;

        // Apply filters
        assemblies.retain(|asm| self.matches_filter(asm, filter));

        // Sort
        self.sort_assemblies(&mut assemblies, sort_by, sort_dir);

        // Paginate
        Ok(apply_pagination(
            assemblies,
            filter.common.offset,
            filter.common.limit,
        ))
    }

    /// List assemblies from cache (fast path for list display)
    ///
    /// Returns cached assembly data without loading full YAML files.
    /// Use this for list commands where full entity data isn't needed.
    pub fn list_cached(&self) -> Vec<crate::core::CachedEntity> {
        use crate::core::cache::EntityFilter;
        use crate::core::identity::EntityPrefix;

        let filter = EntityFilter {
            prefix: Some(EntityPrefix::Asm),
            ..Default::default()
        };
        self.cache.list_entities(&filter)
    }

    /// Load all assemblies from the filesystem
    pub fn load_all(&self) -> ServiceResult<Vec<Assembly>> {
        let dir = self.get_directory();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        Ok(loader::load_all(&dir)?)
    }

    /// Get a single assembly by ID
    pub fn get(&self, id: &str) -> ServiceResult<Option<Assembly>> {
        let dir = self.get_directory();
        if let Some((_, asm)) = loader::load_entity::<Assembly>(&dir, id)? {
            return Ok(Some(asm));
        }
        Ok(None)
    }

    /// Get an assembly by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Assembly> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(id.to_string()))
    }

    /// Get an assembly by part number
    pub fn get_by_part_number(&self, part_number: &str) -> ServiceResult<Option<Assembly>> {
        let assemblies = self.load_all()?;
        Ok(assemblies
            .into_iter()
            .find(|a| a.part_number.eq_ignore_ascii_case(part_number)))
    }

    /// Create a new assembly
    pub fn create(&self, input: CreateAssembly) -> ServiceResult<Assembly> {
        // Check for duplicate part number
        if let Some(existing) = self.get_by_part_number(&input.part_number)? {
            return Err(ServiceError::AlreadyExists(format!(
                "Assembly with part number '{}' already exists ({})",
                input.part_number, existing.id
            )));
        }

        let id = EntityId::new(EntityPrefix::Asm);

        let assembly = Assembly {
            id: id.clone(),
            part_number: input.part_number,
            revision: input.revision,
            title: input.title,
            description: input.description,
            bom: input.bom,
            subassemblies: input.subassemblies,
            sw_class: None,
            asil: None,
            dal: None,
            documents: Vec::new(),
            manufacturing: None,
            tags: input.tags,
            status: Status::Draft,
            links: AssemblyLinks::default(),
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
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Update an existing assembly
    pub fn update(&self, id: &str, input: UpdateAssembly) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(id)?;

        // Check for duplicate part number if changing it
        if let Some(new_pn) = &input.part_number {
            if new_pn != &assembly.part_number {
                if let Some(existing) = self.get_by_part_number(new_pn)? {
                    if existing.id != assembly.id {
                        return Err(ServiceError::AlreadyExists(format!(
                            "Assembly with part number '{}' already exists ({})",
                            new_pn, existing.id
                        )));
                    }
                }
            }
        }

        // Apply updates
        if let Some(part_number) = input.part_number {
            assembly.part_number = part_number;
        }
        if let Some(title) = input.title {
            assembly.title = title;
        }
        if let Some(revision) = input.revision {
            assembly.revision = Some(revision);
        }
        if let Some(description) = input.description {
            assembly.description = Some(description);
        }
        if let Some(bom) = input.bom {
            assembly.bom = bom;
        }
        if let Some(subassemblies) = input.subassemblies {
            assembly.subassemblies = subassemblies;
        }
        if let Some(status) = input.status {
            assembly.status = status;
        }
        if let Some(tags) = input.tags {
            assembly.tags = tags;
        }
        if let Some(documents) = input.documents {
            assembly.documents = documents;
        }
        if let Some(manufacturing) = input.manufacturing {
            assembly.manufacturing = Some(manufacturing);
        }

        // Increment revision
        assembly.entity_revision += 1;

        // Write back
        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Delete an assembly
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, assembly) = self.find_assembly(id)?;

        // Check for references unless force is true
        if !force {
            // Check if this is a subassembly of another assembly
            if assembly.links.parent.is_some() {
                return Err(ServiceError::HasReferences);
            }

            // Check if referenced by other assemblies
            let references = self.find_parent_assemblies(&assembly.id.to_string())?;
            if !references.is_empty() {
                return Err(ServiceError::HasReferences);
            }
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Add a component to an assembly's BOM
    pub fn add_component(
        &self,
        assembly_id: &str,
        component_id: &str,
        quantity: u32,
    ) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(assembly_id)?;

        // Check if component exists
        let cmp_dir = self.components_dir();
        if loader::load_entity::<Component>(&cmp_dir, component_id)?.is_none() {
            return Err(ServiceError::NotFound(format!(
                "Component not found: {}",
                component_id
            )));
        }

        // Check if already in BOM
        if assembly
            .bom
            .iter()
            .any(|item| item.component_id == component_id)
        {
            return Err(ServiceError::AlreadyExists(format!(
                "Component {} already in BOM",
                component_id
            )));
        }

        assembly.bom.push(BomItem {
            component_id: component_id.to_string(),
            quantity,
            reference_designators: Vec::new(),
            notes: None,
        });
        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Remove a component from an assembly's BOM
    pub fn remove_component(
        &self,
        assembly_id: &str,
        component_id: &str,
    ) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(assembly_id)?;

        let original_len = assembly.bom.len();
        assembly
            .bom
            .retain(|item| item.component_id != component_id);

        if assembly.bom.len() == original_len {
            return Err(ServiceError::NotFound(format!(
                "Component {} not in BOM",
                component_id
            )));
        }

        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Update the quantity of a component in an assembly's BOM
    pub fn update_component_quantity(
        &self,
        assembly_id: &str,
        component_id: &str,
        quantity: u32,
    ) -> ServiceResult<Assembly> {
        if quantity == 0 {
            return Err(ServiceError::InvalidInput(
                "Quantity must be greater than 0".to_string(),
            ));
        }

        let (path, mut assembly) = self.find_assembly(assembly_id)?;

        let item = assembly
            .bom
            .iter_mut()
            .find(|item| item.component_id == component_id)
            .ok_or_else(|| {
                ServiceError::NotFound(format!("Component {} not in BOM", component_id))
            })?;

        item.quantity = quantity;
        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Add a subassembly reference
    pub fn add_subassembly(
        &self,
        assembly_id: &str,
        subassembly_id: &str,
    ) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(assembly_id)?;

        // Check if subassembly exists
        if self.get(subassembly_id)?.is_none() {
            return Err(ServiceError::NotFound(format!(
                "Subassembly not found: {}",
                subassembly_id
            )));
        }

        // Prevent self-reference
        if assembly.id.to_string() == subassembly_id {
            return Err(ServiceError::InvalidInput(
                "Cannot add assembly as its own subassembly".to_string(),
            ));
        }

        // Check for circular reference
        if self.would_create_cycle(assembly_id, subassembly_id)? {
            return Err(ServiceError::InvalidInput(
                "Adding this subassembly would create a circular reference".to_string(),
            ));
        }

        // Check if already added
        if assembly.subassemblies.contains(&subassembly_id.to_string()) {
            return Err(ServiceError::AlreadyExists(format!(
                "Subassembly {} already added",
                subassembly_id
            )));
        }

        assembly.subassemblies.push(subassembly_id.to_string());
        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Remove a subassembly reference
    pub fn remove_subassembly(
        &self,
        assembly_id: &str,
        subassembly_id: &str,
    ) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(assembly_id)?;

        let original_len = assembly.subassemblies.len();
        assembly.subassemblies.retain(|id| id != subassembly_id);

        if assembly.subassemblies.len() == original_len {
            return Err(ServiceError::NotFound(format!(
                "Subassembly {} not found in assembly",
                subassembly_id
            )));
        }

        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Get the full BOM tree for an assembly
    ///
    /// The `assembly_qty` parameter determines how many assemblies are being built,
    /// which affects price break calculations. For example, if building 100 assemblies,
    /// each component's effective quantity is multiplied by 100 when looking up price breaks.
    pub fn get_bom_tree(&self, id: &str, assembly_qty: u32) -> ServiceResult<BomNode> {
        let assembly = self.get_required(id)?;

        // Load quotes for price lookup (same as calculate_cost_recursive)
        let quotes_dir = self.quotes_dir();
        let quotes: Vec<Quote> = if quotes_dir.exists() {
            loader::load_all(&quotes_dir)?
        } else {
            Vec::new()
        };
        let quote_map: HashMap<String, &Quote> = quotes
            .iter()
            .filter_map(|q| q.component.as_ref().map(|c| (c.clone(), q)))
            .collect();

        self.build_bom_tree(
            &assembly,
            1,
            assembly_qty,
            &quotes,
            &quote_map,
            &mut HashSet::new(),
        )
    }

    /// Calculate total cost for an assembly (recursive)
    pub fn calculate_cost(&self, id: &str, quantity: u32) -> ServiceResult<BomCostResult> {
        let assembly = self.get_required(id)?;
        let mut result = BomCostResult::default();
        let mut visited = HashSet::new();

        self.calculate_cost_recursive(&assembly, quantity, &mut result, &mut visited)?;

        Ok(result)
    }

    /// Calculate total mass for an assembly (recursive)
    pub fn calculate_mass(&self, id: &str, quantity: u32) -> ServiceResult<BomMassResult> {
        let assembly = self.get_required(id)?;
        let mut result = BomMassResult::default();
        let mut visited = HashSet::new();

        self.calculate_mass_recursive(&assembly, quantity, &mut result, &mut visited)?;

        Ok(result)
    }

    /// Get manufacturing routing for an assembly
    pub fn get_routing(&self, id: &str) -> ServiceResult<Vec<String>> {
        let assembly = self.get_required(id)?;
        Ok(assembly
            .manufacturing
            .map(|m| m.routing)
            .unwrap_or_default())
    }

    /// Set manufacturing routing for an assembly
    pub fn set_routing(&self, id: &str, routing: Vec<String>) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(id)?;

        let manufacturing = assembly
            .manufacturing
            .get_or_insert(ManufacturingConfig::default());
        manufacturing.routing = routing;
        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Add a process to manufacturing routing
    pub fn add_routing_process(&self, id: &str, process_id: &str) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(id)?;

        let manufacturing = assembly
            .manufacturing
            .get_or_insert(ManufacturingConfig::default());
        if manufacturing.routing.contains(&process_id.to_string()) {
            return Err(ServiceError::AlreadyExists(format!(
                "Process {} already in routing",
                process_id
            )));
        }
        manufacturing.routing.push(process_id.to_string());
        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Remove a process from manufacturing routing
    pub fn remove_routing_process(&self, id: &str, process_id: &str) -> ServiceResult<Assembly> {
        let (path, mut assembly) = self.find_assembly(id)?;

        if let Some(manufacturing) = &mut assembly.manufacturing {
            let original_len = manufacturing.routing.len();
            manufacturing.routing.retain(|p| p != process_id);

            if manufacturing.routing.len() == original_len {
                return Err(ServiceError::NotFound(format!(
                    "Process {} not in routing",
                    process_id
                )));
            }
        } else {
            return Err(ServiceError::NotFound(format!(
                "Process {} not in routing",
                process_id
            )));
        }

        assembly.entity_revision += 1;

        let yaml =
            serde_yml::to_string(&assembly).map_err(|e| ServiceError::Yaml(e.to_string()))?;
        fs::write(&path, yaml)?;

        Ok(assembly)
    }

    /// Get statistics about assemblies
    pub fn stats(&self) -> ServiceResult<AssemblyStats> {
        let assemblies = self.load_all()?;

        let mut stats = AssemblyStats::default();
        stats.total = assemblies.len();

        // Build set of all subassembly IDs
        let subasm_ids: HashSet<String> = assemblies
            .iter()
            .flat_map(|a| a.subassemblies.clone())
            .collect();

        for asm in &assemblies {
            // Count by status
            match asm.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            // Empty BOM
            if asm.bom.is_empty() && asm.subassemblies.is_empty() {
                stats.empty_bom += 1;
            }

            // Total BOM items
            stats.total_bom_items += asm.bom.len();

            // Top-level vs sub-assembly
            if subasm_ids.contains(&asm.id.to_string()) {
                stats.sub_assemblies += 1;
            } else {
                stats.top_level += 1;
            }
        }

        Ok(stats)
    }

    // --- Private helper methods ---

    /// Find an assembly and its file path (cache-first lookup)
    fn find_assembly(&self, id: &str) -> ServiceResult<(PathBuf, Assembly)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(assembly) = crate::yaml::parse_yaml_file::<Assembly>(&path) {
                    return Ok((path, assembly));
                }
            }
        }

        // Fall back to directory scan
        let dir = self.get_directory();
        if let Some((path, asm)) = loader::load_entity::<Assembly>(&dir, id)? {
            return Ok((path, asm));
        }
        Err(ServiceError::NotFound(id.to_string()))
    }

    /// Find assemblies that contain this one as a subassembly
    fn find_parent_assemblies(&self, subassembly_id: &str) -> ServiceResult<Vec<String>> {
        let assemblies = self.load_all()?;
        Ok(assemblies
            .into_iter()
            .filter(|a| a.subassemblies.contains(&subassembly_id.to_string()))
            .map(|a| a.id.to_string())
            .collect())
    }

    /// Check if adding a subassembly would create a cycle
    fn would_create_cycle(&self, _parent_id: &str, subassembly_id: &str) -> ServiceResult<bool> {
        // Get the subassembly and check if parent is in its descendants
        if let Some(sub) = self.get(subassembly_id)? {
            let mut visited = HashSet::new();
            return self.has_descendant(&sub, _parent_id, &mut visited);
        }
        Ok(false)
    }

    /// Check if an assembly has a specific ID as a descendant
    fn has_descendant(
        &self,
        assembly: &Assembly,
        target_id: &str,
        visited: &mut HashSet<String>,
    ) -> ServiceResult<bool> {
        let asm_id = assembly.id.to_string();
        if visited.contains(&asm_id) {
            return Ok(false);
        }
        visited.insert(asm_id);

        for sub_id in &assembly.subassemblies {
            if sub_id == target_id {
                return Ok(true);
            }
            if let Some(sub) = self.get(sub_id)? {
                if self.has_descendant(&sub, target_id, visited)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Check if an assembly matches the given filter
    fn matches_filter(&self, asm: &Assembly, filter: &AssemblyFilter) -> bool {
        // Part number filter
        if let Some(pn) = &filter.part_number {
            if !asm.part_number.to_lowercase().contains(&pn.to_lowercase()) {
                return false;
            }
        }

        // Empty BOM filter
        if filter.empty_bom && (!asm.bom.is_empty() || !asm.subassemblies.is_empty()) {
            return false;
        }

        // Has subassemblies filter
        if filter.has_subassemblies && asm.subassemblies.is_empty() {
            return false;
        }

        // Top level filter (no parent link)
        if filter.top_level_only && asm.links.parent.is_some() {
            return false;
        }

        // Sub-assembly filter (has parent link)
        if filter.sub_only && asm.links.parent.is_none() {
            return false;
        }

        // Common filters
        if !filter.common.matches_status(&asm.status) {
            return false;
        }
        if !filter.common.matches_author(&asm.author) {
            return false;
        }
        if !filter.common.matches_tags(&asm.tags) {
            return false;
        }
        if !filter
            .common
            .matches_search(&[&asm.title, &asm.part_number])
        {
            return false;
        }
        if !filter.common.matches_recent(&asm.created) {
            return false;
        }

        true
    }

    /// Sort assemblies by the given field
    fn sort_assemblies(
        &self,
        assemblies: &mut [Assembly],
        sort_by: AssemblySortField,
        sort_dir: SortDirection,
    ) {
        assemblies.sort_by(|a, b| {
            let cmp = match sort_by {
                AssemblySortField::Id => a.id.to_string().cmp(&b.id.to_string()),
                AssemblySortField::PartNumber => a.part_number.cmp(&b.part_number),
                AssemblySortField::Title => a.title.cmp(&b.title),
                AssemblySortField::BomCount => a.bom.len().cmp(&b.bom.len()),
                AssemblySortField::Status => {
                    format!("{:?}", a.status).cmp(&format!("{:?}", b.status))
                }
                AssemblySortField::Author => a.author.cmp(&b.author),
                AssemblySortField::Created => a.created.cmp(&b.created),
            };

            match sort_dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }

    /// Build BOM tree recursively
    ///
    /// Uses quote price breaks when available, falling back to component unit_cost.
    /// The `assembly_qty` is used for price break calculations - the effective quantity
    /// for each component is `item.quantity * assembly_qty`.
    fn build_bom_tree(
        &self,
        assembly: &Assembly,
        quantity: u32,
        assembly_qty: u32,
        quotes: &[Quote],
        quote_map: &HashMap<String, &Quote>,
        visited: &mut HashSet<String>,
    ) -> ServiceResult<BomNode> {
        let asm_id = assembly.id.to_string();

        // Prevent infinite recursion
        if visited.contains(&asm_id) {
            return Ok(BomNode {
                id: asm_id,
                title: format!("{} (circular ref)", assembly.title),
                part_number: assembly.part_number.clone(),
                is_assembly: true,
                quantity,
                unit_cost: None,
                mass_kg: None,
                extended_cost: None,
                extended_mass: None,
                children: Vec::new(),
            });
        }
        visited.insert(asm_id.clone());

        let mut children = Vec::new();

        // Add component children
        let cmp_dir = self.components_dir();
        for item in &assembly.bom {
            if let Some((_, cmp)) = loader::load_entity::<Component>(&cmp_dir, &item.component_id)?
            {
                // Calculate effective quantity for price break lookup
                let effective_qty = item.quantity * assembly_qty;

                // Try to get price from quotes (same logic as calculate_cost_recursive)
                let unit_price = if let Some(quote_id) = &cmp.selected_quote {
                    // Use selected quote
                    quotes
                        .iter()
                        .find(|q| q.id.to_string() == *quote_id)
                        .and_then(|q| q.price_for_qty(effective_qty))
                } else if let Some(quote) = quote_map.get(&item.component_id) {
                    // Use quote linked to this component
                    quote.price_for_qty(effective_qty)
                } else {
                    // Fall back to component's unit_cost
                    cmp.unit_cost
                };

                let ext_cost = unit_price.map(|c| c * item.quantity as f64);
                let ext_mass = cmp.mass_kg.map(|m| m * item.quantity as f64);
                children.push(BomNode {
                    id: item.component_id.clone(),
                    title: cmp.title,
                    part_number: cmp.part_number,
                    is_assembly: false,
                    quantity: item.quantity,
                    unit_cost: unit_price,
                    mass_kg: cmp.mass_kg,
                    extended_cost: ext_cost,
                    extended_mass: ext_mass,
                    children: Vec::new(),
                });
            }
        }

        // Add subassembly children
        for sub_id in &assembly.subassemblies {
            if let Some(sub) = self.get(sub_id)? {
                // Subassemblies are quantity 1 unless specified otherwise
                let sub_node =
                    self.build_bom_tree(&sub, 1, assembly_qty, quotes, quote_map, visited)?;
                children.push(sub_node);
            }
        }

        // Calculate totals for this level
        let total_cost: f64 = children.iter().filter_map(|c| c.extended_cost).sum();
        let total_mass: f64 = children.iter().filter_map(|c| c.extended_mass).sum();

        Ok(BomNode {
            id: asm_id,
            title: assembly.title.clone(),
            part_number: assembly.part_number.clone(),
            is_assembly: true,
            quantity,
            unit_cost: if total_cost > 0.0 {
                Some(total_cost)
            } else {
                None
            },
            mass_kg: if total_mass > 0.0 {
                Some(total_mass)
            } else {
                None
            },
            extended_cost: if total_cost > 0.0 {
                Some(total_cost * quantity as f64)
            } else {
                None
            },
            extended_mass: if total_mass > 0.0 {
                Some(total_mass * quantity as f64)
            } else {
                None
            },
            children,
        })
    }

    /// Calculate cost recursively
    fn calculate_cost_recursive(
        &self,
        assembly: &Assembly,
        quantity: u32,
        result: &mut BomCostResult,
        visited: &mut HashSet<String>,
    ) -> ServiceResult<()> {
        let asm_id = assembly.id.to_string();
        if visited.contains(&asm_id) {
            return Ok(());
        }
        visited.insert(asm_id);

        let cmp_dir = self.components_dir();
        let quotes_dir = self.quotes_dir();

        // Load quotes for price lookup
        let quotes: Vec<Quote> = if quotes_dir.exists() {
            loader::load_all(&quotes_dir)?
        } else {
            Vec::new()
        };
        let quote_map: HashMap<String, &Quote> = quotes
            .iter()
            .filter_map(|q| q.component.as_ref().map(|c| (c.clone(), q)))
            .collect();

        // Process components
        for item in &assembly.bom {
            if let Some((_, cmp)) = loader::load_entity::<Component>(&cmp_dir, &item.component_id)?
            {
                // Try quote price first, then unit_cost
                let price = if let Some(quote_id) = &cmp.selected_quote {
                    quotes
                        .iter()
                        .find(|q| q.id.to_string() == *quote_id)
                        .and_then(|q| q.price_for_qty(item.quantity * quantity))
                } else if let Some(quote) = quote_map.get(&item.component_id) {
                    quote.price_for_qty(item.quantity * quantity)
                } else {
                    cmp.unit_cost
                };

                if let Some(unit_price) = price {
                    result.total_cost += unit_price * (item.quantity * quantity) as f64;
                    result.components_with_cost += 1;
                } else {
                    result.components_without_cost += 1;
                    result.missing_cost.push(item.component_id.clone());
                }
            }
        }

        // Process subassemblies
        for sub_id in &assembly.subassemblies {
            if let Some(sub) = self.get(sub_id)? {
                self.calculate_cost_recursive(&sub, quantity, result, visited)?;
            }
        }

        Ok(())
    }

    /// Calculate mass recursively
    fn calculate_mass_recursive(
        &self,
        assembly: &Assembly,
        quantity: u32,
        result: &mut BomMassResult,
        visited: &mut HashSet<String>,
    ) -> ServiceResult<()> {
        let asm_id = assembly.id.to_string();
        if visited.contains(&asm_id) {
            return Ok(());
        }
        visited.insert(asm_id);

        let cmp_dir = self.components_dir();

        // Process components
        for item in &assembly.bom {
            if let Some((_, cmp)) = loader::load_entity::<Component>(&cmp_dir, &item.component_id)?
            {
                if let Some(mass) = cmp.mass_kg {
                    result.total_mass_kg += mass * (item.quantity * quantity) as f64;
                    result.components_with_mass += 1;
                } else {
                    result.components_without_mass += 1;
                    result.missing_mass.push(item.component_id.clone());
                }
            }
        }

        // Process subassemblies
        for sub_id in &assembly.subassemblies {
            if let Some(sub) = self.get(sub_id)? {
                self.calculate_mass_recursive(&sub, quantity, result, visited)?;
            }
        }

        Ok(())
    }

    // =========================================================================
    // Cache-based BOM methods (Phase 2 - fast path using cache)
    // =========================================================================

    /// Get flattened BOM using cache (no assembly file reads)
    ///
    /// Returns all components in the BOM hierarchy with their effective quantities
    /// accumulated from all paths. This is much faster than loading YAML files
    /// for large assemblies as it uses the SQLite cache.
    ///
    /// Note: The cache must be synced (`cache.sync()`) after any BOM changes.
    pub fn get_flattened_bom_cached(
        &self,
        assembly_id: &str,
    ) -> ServiceResult<Vec<crate::core::cache::FlattenedBomItem>> {
        Ok(self.cache.get_flattened_bom(assembly_id))
    }

    /// Batch-load components by IDs
    ///
    /// Loads only the specified components from the filesystem, avoiding
    /// loading all components when only a subset is needed.
    pub fn batch_load_components(
        &self,
        ids: &HashSet<&String>,
    ) -> ServiceResult<HashMap<String, Component>> {
        let cmp_dir = self.components_dir();
        let mut result = HashMap::new();

        for id in ids {
            if let Some((_, cmp)) = loader::load_entity::<Component>(&cmp_dir, id)? {
                result.insert(cmp.id.to_string(), cmp);
            }
        }

        Ok(result)
    }

    /// Batch-load quotes by IDs
    ///
    /// Loads only the specified quotes from the filesystem.
    pub fn batch_load_quotes(
        &self,
        ids: &HashSet<&String>,
    ) -> ServiceResult<HashMap<String, Quote>> {
        let quotes_dir = self.quotes_dir();
        let mut result = HashMap::new();

        if !quotes_dir.exists() {
            return Ok(result);
        }

        for id in ids {
            if let Some((_, quote)) = loader::load_entity::<Quote>(&quotes_dir, id)? {
                result.insert(quote.id.to_string(), quote);
            }
        }

        Ok(result)
    }

    /// Calculate BOM cost using cache for hierarchy traversal (fast path)
    ///
    /// This is an optimized version of `calculate_cost()` that:
    /// 1. Uses the cache for BOM hierarchy traversal (no assembly file reads)
    /// 2. Batch-loads only the unique components and quotes needed
    /// 3. Uses existing `Quote.price_for_qty()` logic for price breaks
    ///
    /// The `production_qty` parameter is the number of assemblies being built,
    /// which affects price break calculations. For example, if building 100
    /// assemblies and each needs 5 of component X, the purchase quantity is 500.
    ///
    /// Returns a detailed cost breakdown with per-line pricing info.
    pub fn calculate_cost_cached(
        &self,
        assembly_id: &str,
        production_qty: u32,
    ) -> ServiceResult<BomCostResultDetailed> {
        // 1. Get flattened BOM from cache (fast - no file reads)
        let flattened = self.get_flattened_bom_cached(assembly_id)?;

        // 2. Collect unique component IDs
        let component_ids: HashSet<&String> = flattened.iter().map(|b| &b.component_id).collect();

        // 3. Batch-load components (only the ones we need)
        let components = self.batch_load_components(&component_ids)?;

        // 4. Load quotes for pricing
        // Note: We load all quotes rather than batch-loading by ID because we also need
        // to look up quotes by component ID for fallback pricing. For projects with many
        // quotes, this could be optimized further by caching quote data.
        let quotes_dir = self.quotes_dir();
        let all_quotes: Vec<Quote> = if quotes_dir.exists() {
            loader::load_all(&quotes_dir)?
        } else {
            Vec::new()
        };

        // Build maps for efficient lookup
        let quotes_by_id: HashMap<String, &Quote> =
            all_quotes.iter().map(|q| (q.id.to_string(), q)).collect();
        let quotes_by_component: HashMap<String, &Quote> = all_quotes
            .iter()
            .filter_map(|q| q.component.as_ref().map(|c| (c.clone(), q)))
            .collect();

        // 6. Calculate costs
        let mut result = BomCostResultDetailed {
            total_unit_cost: 0.0,
            total_nre_cost: 0.0,
            component_costs: Vec::new(),
            warnings: Vec::new(),
        };

        let mut processed_quotes: HashSet<String> = HashSet::new();

        for item in &flattened {
            let cmp = match components.get(&item.component_id) {
                Some(c) => c,
                None => {
                    result.warnings.push(format!(
                        "Component {} not found in filesystem",
                        item.component_id
                    ));
                    continue;
                }
            };

            // Calculate effective quantity for this production run
            let effective_qty = item.effective_qty;
            let purchase_qty = effective_qty * production_qty;

            // Resolve unit price using priority: selected_quote → component quotes → unit_cost
            let (unit_price, quote_id, price_break_tier) = self.resolve_component_price(
                cmp,
                purchase_qty,
                &quotes_by_id,
                &quotes_by_component,
            );

            // Calculate extended price
            let extended_price = unit_price.map(|p| p * effective_qty as f64);

            // Track NRE costs from quote (avoid double-counting)
            let nre_contribution = if let Some(ref qid) = quote_id {
                if !processed_quotes.contains(qid) {
                    processed_quotes.insert(qid.clone());
                    if let Some(quote) = quotes_by_id.get(qid) {
                        quote.total_nre()
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Add to totals
            if let Some(ext) = extended_price {
                result.total_unit_cost += ext;
            }
            result.total_nre_cost += nre_contribution;

            // Record line item
            result.component_costs.push(ComponentCostLine {
                component_id: item.component_id.clone(),
                title: cmp.title.clone(),
                part_number: cmp.part_number.clone(),
                effective_qty,
                unit_price,
                extended_price,
                quote_id,
                price_break_tier,
                nre_contribution,
            });
        }

        Ok(result)
    }

    /// Resolve price for a component using the standard priority chain
    ///
    /// Priority: selected_quote → quotes for component → unit_cost fallback
    fn resolve_component_price(
        &self,
        cmp: &Component,
        purchase_qty: u32,
        quotes_by_id: &HashMap<String, &Quote>,
        quotes_by_component: &HashMap<String, &Quote>,
    ) -> (Option<f64>, Option<String>, Option<u32>) {
        // 1. Try selected_quote first
        if let Some(quote_id) = &cmp.selected_quote {
            if let Some(quote) = quotes_by_id.get(quote_id) {
                if let Some(price) = quote.price_for_qty(purchase_qty) {
                    let tier = quote
                        .price_breaks
                        .iter()
                        .filter(|pb| pb.min_qty <= purchase_qty)
                        .map(|pb| pb.min_qty)
                        .last();
                    return (Some(price), Some(quote_id.clone()), tier);
                }
            }
        }

        // 2. Try quotes linked to this component
        if let Some(quote) = quotes_by_component.get(&cmp.id.to_string()) {
            if let Some(price) = quote.price_for_qty(purchase_qty) {
                let tier = quote
                    .price_breaks
                    .iter()
                    .filter(|pb| pb.min_qty <= purchase_qty)
                    .map(|pb| pb.min_qty)
                    .last();
                return (Some(price), Some(quote.id.to_string()), tier);
            }
        }

        // 3. Fall back to component's unit_cost
        (cmp.unit_cost, None, None)
    }
}

/// Detailed BOM cost result with per-line breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BomCostResultDetailed {
    /// Total unit cost for one assembly (sum of extended prices)
    pub total_unit_cost: f64,

    /// Total NRE costs (tooling, setup, etc.) - one-time costs
    pub total_nre_cost: f64,

    /// Per-component cost breakdown
    pub component_costs: Vec<ComponentCostLine>,

    /// Warnings (missing components, etc.)
    pub warnings: Vec<String>,
}

impl BomCostResultDetailed {
    /// Get total cost for a production run
    ///
    /// `production_qty` - number of assemblies being built
    /// `amortize_nre` - if true, amortize NRE over production_qty
    pub fn total_production_cost(&self, production_qty: u32, amortize_nre: bool) -> f64 {
        let unit_total = self.total_unit_cost * production_qty as f64;
        if amortize_nre {
            unit_total + self.total_nre_cost
        } else {
            unit_total
        }
    }

    /// Get effective unit cost including amortized NRE
    pub fn effective_unit_cost(&self, amortize_qty: u32) -> f64 {
        if amortize_qty == 0 {
            self.total_unit_cost
        } else {
            self.total_unit_cost + (self.total_nre_cost / amortize_qty as f64)
        }
    }

    /// Count of components with pricing
    pub fn components_with_cost(&self) -> usize {
        self.component_costs
            .iter()
            .filter(|c| c.unit_price.is_some())
            .count()
    }

    /// Count of components without pricing
    pub fn components_without_cost(&self) -> usize {
        self.component_costs
            .iter()
            .filter(|c| c.unit_price.is_none())
            .count()
    }

    /// List of component IDs missing cost data
    pub fn missing_cost_ids(&self) -> Vec<String> {
        self.component_costs
            .iter()
            .filter(|c| c.unit_price.is_none())
            .map(|c| c.component_id.clone())
            .collect()
    }
}

/// Cost line item for a single component in the BOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCostLine {
    /// Component entity ID
    pub component_id: String,

    /// Component title
    pub title: String,

    /// Part number
    pub part_number: String,

    /// Effective quantity per assembly (accumulated from BOM hierarchy)
    pub effective_qty: u32,

    /// Unit price (from quote or component fallback)
    pub unit_price: Option<f64>,

    /// Extended price (unit_price × effective_qty)
    pub extended_price: Option<f64>,

    /// Quote ID used for pricing (if any)
    pub quote_id: Option<String>,

    /// Price break tier used (min_qty of the tier)
    pub price_break_tier: Option<u32>,

    /// NRE contribution from this quote (only counted once per quote)
    pub nre_contribution: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();

        // Initialize project structure
        fs::create_dir_all(tmp.path().join(".tdt")).unwrap();
        fs::create_dir_all(tmp.path().join("bom/assemblies")).unwrap();
        fs::create_dir_all(tmp.path().join("bom/components")).unwrap();

        // Create config file
        fs::write(tmp.path().join(".tdt/config.yaml"), "author: Test Author\n").unwrap();

        let project = Project::discover_from(tmp.path()).unwrap();
        let cache = EntityCache::open(&project).unwrap();

        (tmp, project, cache)
    }

    fn create_test_component(
        tmp: &TempDir,
        part_number: &str,
        cost: Option<f64>,
        mass: Option<f64>,
    ) -> Component {
        let id = EntityId::new(EntityPrefix::Cmp);
        let cmp = Component {
            id: id.clone(),
            part_number: part_number.to_string(),
            revision: None,
            title: format!("Component {}", part_number),
            description: None,
            make_buy: crate::entities::component::MakeBuy::Buy,
            category: crate::entities::component::ComponentCategory::Mechanical,
            sw_class: None,
            asil: None,
            dal: None,
            material: None,
            mass_kg: mass,
            unit_cost: cost,
            selected_quote: None,
            suppliers: Vec::new(),
            documents: Vec::new(),
            coordinate_system: None,
            datum_frame: None,
            manufacturing: None,
            tags: Vec::new(),
            status: Status::Draft,
            links: crate::entities::component::ComponentLinks::default(),
            created: Utc::now(),
            author: "Test".to_string(),
            entity_revision: 1,
        };

        let yaml = serde_yml::to_string(&cmp).unwrap();
        fs::write(
            tmp.path().join(format!("bom/components/{}.tdt.yaml", id)),
            yaml,
        )
        .unwrap();

        cmp
    }

    #[test]
    fn test_create_assembly() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let input = CreateAssembly {
            part_number: "ASM-001".into(),
            title: "Main Assembly".into(),
            author: "Test Author".into(),
            ..Default::default()
        };

        let asm = service.create(input).unwrap();

        assert_eq!(asm.part_number, "ASM-001");
        assert_eq!(asm.title, "Main Assembly");
        assert_eq!(asm.status, Status::Draft);
        assert!(asm.bom.is_empty());
    }

    #[test]
    fn test_duplicate_part_number_rejected() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        // Create first assembly
        service
            .create(CreateAssembly {
                part_number: "ASM-001".into(),
                title: "First".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // Try to create duplicate
        let result = service.create(CreateAssembly {
            part_number: "ASM-001".into(),
            title: "Second".into(),
            author: "Test".into(),
            ..Default::default()
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_get_assembly() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let created = service
            .create(CreateAssembly {
                part_number: "ASM-002".into(),
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
    fn test_update_assembly() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let created = service
            .create(CreateAssembly {
                part_number: "ASM-003".into(),
                title: "Original".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .update(
                &created.id.to_string(),
                UpdateAssembly {
                    title: Some("Updated Title".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.entity_revision, 2);
    }

    #[test]
    fn test_delete_assembly() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let created = service
            .create(CreateAssembly {
                part_number: "ASM-004".into(),
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
    fn test_add_component_to_bom() {
        let (tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        // Create component first
        let cmp = create_test_component(&tmp, "CMP-001", Some(10.0), Some(0.5));

        // Create assembly
        let asm = service
            .create(CreateAssembly {
                part_number: "ASM-005".into(),
                title: "With Components".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // Add component
        let updated = service
            .add_component(&asm.id.to_string(), &cmp.id.to_string(), 4)
            .unwrap();

        assert_eq!(updated.bom.len(), 1);
        assert_eq!(updated.bom[0].component_id, cmp.id.to_string());
        assert_eq!(updated.bom[0].quantity, 4);
    }

    #[test]
    fn test_remove_component_from_bom() {
        let (tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let cmp = create_test_component(&tmp, "CMP-002", Some(20.0), None);

        let asm = service
            .create(CreateAssembly {
                part_number: "ASM-006".into(),
                title: "Remove Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .add_component(&asm.id.to_string(), &cmp.id.to_string(), 2)
            .unwrap();

        let updated = service
            .remove_component(&asm.id.to_string(), &cmp.id.to_string())
            .unwrap();

        assert!(updated.bom.is_empty());
    }

    #[test]
    fn test_add_subassembly() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let sub = service
            .create(CreateAssembly {
                part_number: "SUB-001".into(),
                title: "Sub Assembly".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let parent = service
            .create(CreateAssembly {
                part_number: "PARENT-001".into(),
                title: "Parent Assembly".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .add_subassembly(&parent.id.to_string(), &sub.id.to_string())
            .unwrap();

        assert_eq!(updated.subassemblies.len(), 1);
        assert_eq!(updated.subassemblies[0], sub.id.to_string());
    }

    #[test]
    fn test_self_reference_rejected() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let asm = service
            .create(CreateAssembly {
                part_number: "SELF-001".into(),
                title: "Self Ref Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let result = service.add_subassembly(&asm.id.to_string(), &asm.id.to_string());

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_cost() {
        let (tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        // Create components with costs
        let cmp1 = create_test_component(&tmp, "CMP-C1", Some(10.0), None);
        let cmp2 = create_test_component(&tmp, "CMP-C2", Some(25.0), None);

        // Create assembly
        let asm = service
            .create(CreateAssembly {
                part_number: "ASM-COST".into(),
                title: "Cost Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .add_component(&asm.id.to_string(), &cmp1.id.to_string(), 4)
            .unwrap();
        service
            .add_component(&asm.id.to_string(), &cmp2.id.to_string(), 2)
            .unwrap();

        let result = service.calculate_cost(&asm.id.to_string(), 1).unwrap();

        // 4 * 10 + 2 * 25 = 90
        assert_eq!(result.total_cost, 90.0);
        assert_eq!(result.components_with_cost, 2);
        assert_eq!(result.components_without_cost, 0);
    }

    #[test]
    fn test_calculate_mass() {
        let (tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        // Create components with mass
        let cmp1 = create_test_component(&tmp, "CMP-M1", None, Some(0.5));
        let cmp2 = create_test_component(&tmp, "CMP-M2", None, Some(1.0));
        let _cmp3 = create_test_component(&tmp, "CMP-M3", None, None); // No mass

        // Create assembly
        let asm = service
            .create(CreateAssembly {
                part_number: "ASM-MASS".into(),
                title: "Mass Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .add_component(&asm.id.to_string(), &cmp1.id.to_string(), 4)
            .unwrap();
        service
            .add_component(&asm.id.to_string(), &cmp2.id.to_string(), 2)
            .unwrap();

        let result = service.calculate_mass(&asm.id.to_string(), 1).unwrap();

        // 4 * 0.5 + 2 * 1.0 = 4.0
        assert_eq!(result.total_mass_kg, 4.0);
        assert_eq!(result.components_with_mass, 2);
    }

    #[test]
    fn test_bom_tree() {
        let (tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let cmp = create_test_component(&tmp, "CMP-TREE", Some(5.0), Some(0.1));

        let asm = service
            .create(CreateAssembly {
                part_number: "ASM-TREE".into(),
                title: "Tree Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .add_component(&asm.id.to_string(), &cmp.id.to_string(), 10)
            .unwrap();

        let tree = service.get_bom_tree(&asm.id.to_string(), 1).unwrap();

        assert_eq!(tree.title, "Tree Test");
        assert!(tree.is_assembly);
        assert_eq!(tree.children.len(), 1);
        assert!(!tree.children[0].is_assembly);
        assert_eq!(tree.children[0].quantity, 10);
        // Check that unit_cost is populated from component
        assert_eq!(tree.children[0].unit_cost, Some(5.0));
        assert_eq!(tree.children[0].extended_cost, Some(50.0)); // 10 * 5.0
    }

    #[test]
    fn test_set_routing() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let asm = service
            .create(CreateAssembly {
                part_number: "ASM-ROUTE".into(),
                title: "Routing Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let routing = vec!["PROC-001".to_string(), "PROC-002".to_string()];
        let updated = service
            .set_routing(&asm.id.to_string(), routing.clone())
            .unwrap();

        assert_eq!(updated.manufacturing.as_ref().unwrap().routing, routing);

        let fetched_routing = service.get_routing(&asm.id.to_string()).unwrap();
        assert_eq!(fetched_routing, routing);
    }

    #[test]
    fn test_add_routing_process() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        let asm = service
            .create(CreateAssembly {
                part_number: "ASM-ADDROUTE".into(),
                title: "Add Route Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .add_routing_process(&asm.id.to_string(), "PROC-001")
            .unwrap();
        let updated = service
            .add_routing_process(&asm.id.to_string(), "PROC-002")
            .unwrap();

        let routing = updated.manufacturing.as_ref().unwrap().routing.clone();
        assert_eq!(routing.len(), 2);
        assert_eq!(routing[0], "PROC-001");
        assert_eq!(routing[1], "PROC-002");
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        service
            .create(CreateAssembly {
                part_number: "ASM-S1".into(),
                title: "Stat 1".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateAssembly {
                part_number: "ASM-S2".into(),
                title: "Stat 2".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let stats = service.stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.by_status.draft, 2);
        assert_eq!(stats.empty_bom, 2);
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup_test_project();
        let service = AssemblyService::new(&project, &cache);

        service
            .create(CreateAssembly {
                part_number: "PROD-001".into(),
                title: "Product".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateAssembly {
                part_number: "SUB-002".into(),
                title: "Subassy".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        // List all
        let all = service
            .list(
                &AssemblyFilter::default(),
                AssemblySortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(all.items.len(), 2);

        // Filter by part number
        let filtered = service
            .list(
                &AssemblyFilter {
                    part_number: Some("PROD".into()),
                    ..Default::default()
                },
                AssemblySortField::Created,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(filtered.items.len(), 1);
        assert_eq!(filtered.items[0].part_number, "PROD-001");
    }

    // =========================================================================
    // Cache-based cost calculation tests
    // =========================================================================

    #[test]
    fn test_calculate_cost_cached_simple() {
        let (tmp, project, mut cache) = setup_test_project();

        // Create components with costs
        let cmp1 = create_test_component(&tmp, "CMP-CC1", Some(10.0), None);
        let cmp2 = create_test_component(&tmp, "CMP-CC2", Some(25.0), None);

        let asm_id = {
            let service = AssemblyService::new(&project, &cache);

            // Create assembly
            let asm = service
                .create(CreateAssembly {
                    part_number: "ASM-CACHED-COST".into(),
                    title: "Cached Cost Test".into(),
                    author: "Test".into(),
                    ..Default::default()
                })
                .unwrap();

            service
                .add_component(&asm.id.to_string(), &cmp1.id.to_string(), 4)
                .unwrap();
            service
                .add_component(&asm.id.to_string(), &cmp2.id.to_string(), 2)
                .unwrap();

            asm.id.to_string()
        };

        // Sync cache so BOM items are populated
        cache.sync().unwrap();

        // Calculate cost using cache (new service instance)
        let service = AssemblyService::new(&project, &cache);
        let result = service.calculate_cost_cached(&asm_id, 1).unwrap();

        // 4 * 10 + 2 * 25 = 90
        assert_eq!(result.total_unit_cost, 90.0);
        assert_eq!(result.components_with_cost(), 2);
        assert_eq!(result.components_without_cost(), 0);
        assert_eq!(result.component_costs.len(), 2);
    }

    #[test]
    fn test_calculate_cost_cached_with_production_qty() {
        let (tmp, project, mut cache) = setup_test_project();

        // Create component with cost
        let cmp = create_test_component(&tmp, "CMP-PQ", Some(5.0), None);

        let asm_id = {
            let service = AssemblyService::new(&project, &cache);

            // Create assembly
            let asm = service
                .create(CreateAssembly {
                    part_number: "ASM-PQ".into(),
                    title: "Production Qty Test".into(),
                    author: "Test".into(),
                    ..Default::default()
                })
                .unwrap();

            service
                .add_component(&asm.id.to_string(), &cmp.id.to_string(), 2)
                .unwrap();

            asm.id.to_string()
        };

        // Sync cache
        cache.sync().unwrap();

        // Calculate for production qty of 100 (new service instance)
        let service = AssemblyService::new(&project, &cache);
        let result = service.calculate_cost_cached(&asm_id, 100).unwrap();

        // Unit cost is still 2 * 5 = 10 (per assembly)
        assert_eq!(result.total_unit_cost, 10.0);

        // Total production cost would be 10 * 100 = 1000
        assert_eq!(result.total_production_cost(100, false), 1000.0);
    }

    #[test]
    fn test_calculate_cost_cached_missing_cost() {
        let (tmp, project, mut cache) = setup_test_project();

        // Create component without cost
        let cmp = create_test_component(&tmp, "CMP-NC", None, None);
        let cmp_id = cmp.id.to_string();

        let asm_id = {
            let service = AssemblyService::new(&project, &cache);

            // Create assembly
            let asm = service
                .create(CreateAssembly {
                    part_number: "ASM-NC".into(),
                    title: "No Cost Test".into(),
                    author: "Test".into(),
                    ..Default::default()
                })
                .unwrap();

            service
                .add_component(&asm.id.to_string(), &cmp_id, 3)
                .unwrap();

            asm.id.to_string()
        };

        // Sync cache
        cache.sync().unwrap();

        let service = AssemblyService::new(&project, &cache);
        let result = service.calculate_cost_cached(&asm_id, 1).unwrap();

        assert_eq!(result.total_unit_cost, 0.0);
        assert_eq!(result.components_with_cost(), 0);
        assert_eq!(result.components_without_cost(), 1);
        assert_eq!(result.missing_cost_ids(), vec![cmp_id]);
    }

    #[test]
    fn test_calculate_cost_cached_with_subassembly() {
        let (tmp, project, mut cache) = setup_test_project();

        // Create components
        let cmp1 = create_test_component(&tmp, "CMP-SUB1", Some(10.0), None);
        let cmp2 = create_test_component(&tmp, "CMP-SUB2", Some(20.0), None);

        let parent_id = {
            let service = AssemblyService::new(&project, &cache);

            // Create sub-assembly with cmp1 (qty 2)
            let sub_asm = service
                .create(CreateAssembly {
                    part_number: "SUB-ASM".into(),
                    title: "Sub Assembly".into(),
                    author: "Test".into(),
                    ..Default::default()
                })
                .unwrap();
            service
                .add_component(&sub_asm.id.to_string(), &cmp1.id.to_string(), 2)
                .unwrap();

            // Create parent assembly with sub-assembly and cmp2 (qty 3)
            let parent = service
                .create(CreateAssembly {
                    part_number: "PARENT-ASM".into(),
                    title: "Parent Assembly".into(),
                    author: "Test".into(),
                    ..Default::default()
                })
                .unwrap();
            service
                .add_subassembly(&parent.id.to_string(), &sub_asm.id.to_string())
                .unwrap();
            service
                .add_component(&parent.id.to_string(), &cmp2.id.to_string(), 3)
                .unwrap();

            parent.id.to_string()
        };

        // Sync cache
        cache.sync().unwrap();

        let service = AssemblyService::new(&project, &cache);
        let result = service.calculate_cost_cached(&parent_id, 1).unwrap();

        // Sub-assembly: 2 * 10 = 20
        // Parent direct: 3 * 20 = 60
        // Total: 20 + 60 = 80
        assert_eq!(result.total_unit_cost, 80.0);
        assert_eq!(result.components_with_cost(), 2);
    }

    #[test]
    fn test_get_flattened_bom_cached() {
        let (tmp, project, mut cache) = setup_test_project();

        // Create components
        let cmp1 = create_test_component(&tmp, "CMP-FLAT1", Some(5.0), None);
        let cmp2 = create_test_component(&tmp, "CMP-FLAT2", Some(10.0), None);
        let cmp1_id = cmp1.id.to_string();
        let cmp2_id = cmp2.id.to_string();

        let asm_id = {
            let service = AssemblyService::new(&project, &cache);

            // Create assembly
            let asm = service
                .create(CreateAssembly {
                    part_number: "ASM-FLAT".into(),
                    title: "Flat BOM Test".into(),
                    author: "Test".into(),
                    ..Default::default()
                })
                .unwrap();

            service
                .add_component(&asm.id.to_string(), &cmp1_id, 4)
                .unwrap();
            service
                .add_component(&asm.id.to_string(), &cmp2_id, 2)
                .unwrap();

            asm.id.to_string()
        };

        // Sync cache
        cache.sync().unwrap();

        let service = AssemblyService::new(&project, &cache);
        let flattened = service.get_flattened_bom_cached(&asm_id).unwrap();

        assert_eq!(flattened.len(), 2);

        // Verify component quantities
        let cmp1_entry = flattened.iter().find(|f| f.component_id == cmp1_id);
        let cmp2_entry = flattened.iter().find(|f| f.component_id == cmp2_id);

        assert!(cmp1_entry.is_some());
        assert!(cmp2_entry.is_some());
        assert_eq!(cmp1_entry.unwrap().effective_qty, 4);
        assert_eq!(cmp2_entry.unwrap().effective_qty, 2);
    }

    #[test]
    fn test_bom_cost_result_detailed_helpers() {
        let result = BomCostResultDetailed {
            total_unit_cost: 100.0,
            total_nre_cost: 500.0,
            component_costs: vec![ComponentCostLine {
                component_id: "CMP-1".into(),
                title: "Test".into(),
                part_number: "PN-1".into(),
                effective_qty: 2,
                unit_price: Some(50.0),
                extended_price: Some(100.0),
                quote_id: None,
                price_break_tier: None,
                nre_contribution: 0.0,
            }],
            warnings: Vec::new(),
        };

        // Test total_production_cost
        assert_eq!(result.total_production_cost(10, false), 1000.0); // 100 * 10
        assert_eq!(result.total_production_cost(10, true), 1500.0); // 100 * 10 + 500

        // Test effective_unit_cost
        assert_eq!(result.effective_unit_cost(0), 100.0); // No amortization
        assert_eq!(result.effective_unit_cost(100), 105.0); // 100 + 500/100
        assert_eq!(result.effective_unit_cost(500), 101.0); // 100 + 500/500

        // Test helpers
        assert_eq!(result.components_with_cost(), 1);
        assert_eq!(result.components_without_cost(), 0);
        assert!(result.missing_cost_ids().is_empty());
    }
}
