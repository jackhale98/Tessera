//! Feature service for dimensional feature management
//!
//! Provides CRUD operations and business logic for features (GD&T, dimensions).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core::cache::{CachedFeature, EntityCache};
use crate::core::entity::Status;
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::feature::{
    Dimension, Feature, FeatureLinks, FeatureType, GdtControl, Geometry3D, GeometryClass,
    TorsorBounds,
};
use crate::entities::stackup::Distribution;
use crate::services::base::ServiceBase;
use crate::services::common::{
    CommonFilter, ServiceError, ServiceResult, SortDirection, SortKey, Sortable,
};

/// Filter options for listing features
#[derive(Debug, Clone, Default)]
pub struct FeatureFilter {
    /// Common filters (status, author, tags, search)
    pub common: CommonFilter,
    /// Filter by feature type
    pub feature_type: Option<FeatureType>,
    /// Filter by component ID (substring match)
    pub component: Option<String>,
    /// Filter by geometry class
    pub geometry_class: Option<GeometryClass>,
    /// Show only features with GD&T controls
    pub has_gdt: Option<bool>,
    /// Show only features that are datums
    pub is_datum: Option<bool>,
    /// Sort field
    pub sort: FeatureSortField,
    /// Sort direction
    pub sort_direction: SortDirection,
}

/// Fields available for sorting features
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureSortField {
    Id,
    Title,
    Type,
    Component,
    Status,
    Author,
    #[default]
    Created,
}

impl Sortable for Feature {
    type SortField = FeatureSortField;

    fn sort_key(&self, field: &Self::SortField) -> SortKey {
        match field {
            FeatureSortField::Id => SortKey::String(self.id.to_string()),
            FeatureSortField::Title => SortKey::String(self.title.clone()),
            FeatureSortField::Type => SortKey::String(self.feature_type.to_string()),
            FeatureSortField::Component => SortKey::String(self.component.clone()),
            FeatureSortField::Status => SortKey::String(self.status.to_string()),
            FeatureSortField::Author => SortKey::String(self.author.clone()),
            FeatureSortField::Created => SortKey::DateTime(self.created.timestamp()),
        }
    }
}

/// Input for creating a new feature
#[derive(Debug, Clone)]
pub struct CreateFeature {
    /// Parent component ID (required)
    pub component: String,
    /// Feature type
    pub feature_type: FeatureType,
    /// Feature title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Initial dimensions
    pub dimensions: Vec<Dimension>,
    /// Initial GD&T controls
    pub gdt: Vec<GdtControl>,
    /// Geometry class
    pub geometry_class: Option<GeometryClass>,
    /// Datum label if this feature is a datum
    pub datum_label: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Initial status
    pub status: Option<Status>,
    /// Author
    pub author: String,
}

/// Input for updating an existing feature
#[derive(Debug, Clone, Default)]
pub struct UpdateFeature {
    /// Update title
    pub title: Option<String>,
    /// Update description
    pub description: Option<Option<String>>,
    /// Update feature type
    pub feature_type: Option<FeatureType>,
    /// Update geometry class
    pub geometry_class: Option<Option<GeometryClass>>,
    /// Update datum label
    pub datum_label: Option<Option<String>>,
    /// Update 3D geometry
    pub geometry_3d: Option<Option<Geometry3D>>,
    /// Update torsor bounds
    pub torsor_bounds: Option<Option<TorsorBounds>>,
    /// Update tags
    pub tags: Option<Vec<String>>,
    /// Update status
    pub status: Option<Status>,
}

/// Statistics about features
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureStats {
    /// Total number of features
    pub total: usize,
    /// Counts by feature type
    pub by_type: FeatureTypeCounts,
    /// Counts by status
    pub by_status: FeatureStatusCounts,
    /// Number with GD&T controls
    pub with_gdt: usize,
    /// Number that are datums
    pub datums: usize,
    /// Number with 3D geometry defined
    pub with_geometry_3d: usize,
}

/// Feature type counts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureTypeCounts {
    pub internal: usize,
    pub external: usize,
}

/// Feature status counts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureStatusCounts {
    pub draft: usize,
    pub review: usize,
    pub approved: usize,
    pub released: usize,
    pub obsolete: usize,
}

/// Service for managing features
pub struct FeatureService<'a> {
    project: &'a Project,
    base: ServiceBase<'a>,
    cache: &'a EntityCache,
}

impl<'a> FeatureService<'a> {
    /// Create a new FeatureService
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

    /// Get the features directory
    fn feature_dir(&self) -> PathBuf {
        self.project().root().join("tolerances/features")
    }

    /// Get the file path for a feature
    fn get_file_path(&self, id: &EntityId) -> PathBuf {
        self.feature_dir().join(format!("{}.tdt.yaml", id))
    }

    /// Load all features
    fn load_all(&self) -> ServiceResult<Vec<Feature>> {
        let dir = self.feature_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        loader::load_all(&dir).map_err(ServiceError::from)
    }

    /// List features using the cache (fast path)
    ///
    /// Returns cached feature data without loading full entities from disk.
    /// Use this for list views and simple queries.
    pub fn list_cached(&self, filter: &FeatureFilter) -> ServiceResult<Vec<CachedFeature>> {
        // Convert filter values to strings for cache query
        let status_str = filter
            .common
            .status
            .as_ref()
            .and_then(|s| s.first())
            .map(|s| format!("{:?}", s).to_lowercase());

        let feature_type_str = filter.feature_type.as_ref().map(|t| match t {
            FeatureType::Internal => "internal",
            FeatureType::External => "external",
        });

        // Query cache
        let mut cached = self.cache.list_features(
            status_str.as_deref(),
            feature_type_str,
            filter.component.as_deref(),
            filter.common.author.as_deref(),
            filter.common.search.as_deref(),
            None, // Apply limit after all filters
        );

        // Apply additional filters not supported by cache query
        if let Some(days) = filter.common.recent_days {
            let cutoff = Utc::now() - chrono::Duration::days(days as i64);
            cached.retain(|f| f.created >= cutoff);
        }

        // Note: geometry_class, has_gdt, is_datum filters require full entity load
        // These are handled in the regular list() method

        // Apply limit
        if let Some(limit) = filter.common.limit {
            cached.truncate(limit);
        }

        Ok(cached)
    }

    /// Find a feature by ID, returning the path and feature
    ///
    /// Uses the cache to find the file path for faster lookup.
    fn find_feature(&self, id: &str) -> ServiceResult<(PathBuf, Feature)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(feature) = crate::yaml::parse_yaml_file::<Feature>(&path) {
                    return Ok((path, feature));
                }
            }
        }

        // Fall back to directory scan
        let dir = self.feature_dir();
        if let Some((path, feature)) = loader::load_entity::<Feature>(&dir, id)? {
            return Ok((path, feature));
        }
        Err(ServiceError::NotFound(format!("Feature: {}", id)))
    }

    /// List features with filtering and sorting
    pub fn list(&self, filter: &FeatureFilter) -> ServiceResult<Vec<Feature>> {
        let mut features = self.load_all()?;

        // Apply filters
        features.retain(|f| self.matches_filter(f, filter));

        // Sort
        crate::services::common::sort_entities(&mut features, filter.sort, filter.sort_direction);

        // Apply limit from common filter
        if let Some(limit) = filter.common.limit {
            features.truncate(limit);
        }

        Ok(features)
    }

    /// Check if a feature matches the filter
    fn matches_filter(&self, feature: &Feature, filter: &FeatureFilter) -> bool {
        // Common filters
        if !filter.common.matches_entity(feature) {
            return false;
        }

        // Feature type filter
        if let Some(ref ft) = filter.feature_type {
            if &feature.feature_type != ft {
                return false;
            }
        }

        // Component filter
        if let Some(ref cmp) = filter.component {
            if !feature.component.contains(cmp) {
                return false;
            }
        }

        // Geometry class filter
        if let Some(ref gc) = filter.geometry_class {
            match &feature.geometry_class {
                Some(feature_gc) if feature_gc == gc => {}
                _ => return false,
            }
        }

        // Has GD&T filter
        if let Some(has_gdt) = filter.has_gdt {
            if has_gdt != feature.has_gdt() {
                return false;
            }
        }

        // Is datum filter
        if let Some(is_datum) = filter.is_datum {
            if is_datum != feature.datum_label.is_some() {
                return false;
            }
        }

        true
    }

    /// Get a feature by ID (cache-first lookup)
    pub fn get(&self, id: &str) -> ServiceResult<Option<Feature>> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                match crate::yaml::parse_yaml_file::<Feature>(&path) {
                    Ok(feature) => return Ok(Some(feature)),
                    Err(_) => {} // Fall through to directory scan
                }
            }
        }

        // Fall back to directory scan
        let dir = self.feature_dir();
        if let Some((_, feature)) = loader::load_entity::<Feature>(&dir, id)? {
            return Ok(Some(feature));
        }
        Ok(None)
    }

    /// Get a feature by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Feature> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(format!("Feature: {}", id)))
    }

    /// Get features for a specific component
    pub fn get_by_component(&self, component_id: &str) -> ServiceResult<Vec<Feature>> {
        self.list(&FeatureFilter {
            component: Some(component_id.to_string()),
            ..Default::default()
        })
    }

    /// Create a new feature
    pub fn create(&self, input: CreateFeature) -> ServiceResult<Feature> {
        let now = Utc::now();
        let id = EntityId::new(EntityPrefix::Feat);

        let feature = Feature {
            id: id.clone(),
            component: input.component,
            feature_type: input.feature_type,
            title: input.title,
            description: input.description,
            dimensions: input.dimensions,
            gdt: input.gdt,
            geometry_class: input.geometry_class,
            datum_label: input.datum_label,
            geometry_3d: None,
            torsor_bounds: None,
            drawing: Default::default(),
            tags: input.tags,
            status: input.status.unwrap_or(Status::Draft),
            links: FeatureLinks::default(),
            created: now,
            author: input.author,
            entity_revision: 1,
        };

        // Ensure directory exists
        let dir = self.feature_dir();
        fs::create_dir_all(&dir)?;

        // Save
        let file_path = self.get_file_path(&id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Update an existing feature
    pub fn update(&self, id: &str, input: UpdateFeature) -> ServiceResult<Feature> {
        let (_, mut feature) = self.find_feature(id)?;

        // Apply updates
        if let Some(title) = input.title {
            feature.title = title;
        }
        if let Some(description) = input.description {
            feature.description = description;
        }
        if let Some(feature_type) = input.feature_type {
            feature.feature_type = feature_type;
        }
        if let Some(geometry_class) = input.geometry_class {
            feature.geometry_class = geometry_class;
        }
        if let Some(datum_label) = input.datum_label {
            feature.datum_label = datum_label;
        }
        if let Some(geometry_3d) = input.geometry_3d {
            feature.geometry_3d = geometry_3d;
        }
        if let Some(torsor_bounds) = input.torsor_bounds {
            feature.torsor_bounds = torsor_bounds;
        }
        if let Some(tags) = input.tags {
            feature.tags = tags;
        }
        if let Some(status) = input.status {
            feature.status = status;
        }

        feature.entity_revision += 1;

        // Save
        let file_path = self.get_file_path(&feature.id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Delete a feature
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, feature) = self.find_feature(id)?;

        // Check for references unless force is true
        if !force {
            // Check if used in mates or stackups
            if !feature.links.used_in_mates.is_empty()
                || !feature.links.used_in_stackups.is_empty()
            {
                return Err(ServiceError::HasReferences);
            }

            // Check cache for incoming links
            let links_to = self.cache.get_links_to(id);
            if !links_to.is_empty() {
                return Err(ServiceError::HasReferences);
            }
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Add a dimension to a feature
    pub fn add_dimension(
        &self,
        id: &str,
        name: String,
        nominal: f64,
        plus_tol: f64,
        minus_tol: f64,
        internal: bool,
        units: Option<String>,
        distribution: Option<Distribution>,
    ) -> ServiceResult<Feature> {
        let (_, mut feature) = self.find_feature(id)?;

        // Check if dimension with same name already exists
        if feature.dimensions.iter().any(|d| d.name == name) {
            return Err(ServiceError::ValidationFailed(format!(
                "Dimension '{}' already exists",
                name
            )));
        }

        feature.dimensions.push(Dimension {
            name,
            nominal,
            plus_tol,
            minus_tol,
            units: units.unwrap_or_else(|| "mm".to_string()),
            internal,
            distribution: distribution.unwrap_or_default(),
        });

        feature.entity_revision += 1;

        let file_path = self.get_file_path(&feature.id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Update a dimension
    pub fn update_dimension(
        &self,
        id: &str,
        name: &str,
        nominal: Option<f64>,
        plus_tol: Option<f64>,
        minus_tol: Option<f64>,
        internal: Option<bool>,
        units: Option<String>,
        distribution: Option<Distribution>,
    ) -> ServiceResult<Feature> {
        let (_, mut feature) = self.find_feature(id)?;

        let dim = feature
            .dimensions
            .iter_mut()
            .find(|d| d.name == name)
            .ok_or_else(|| {
                ServiceError::ValidationFailed(format!("Dimension '{}' not found", name))
            })?;

        if let Some(n) = nominal {
            dim.nominal = n;
        }
        if let Some(p) = plus_tol {
            dim.plus_tol = p;
        }
        if let Some(m) = minus_tol {
            dim.minus_tol = m;
        }
        if let Some(i) = internal {
            dim.internal = i;
        }
        if let Some(u) = units {
            dim.units = u;
        }
        if let Some(d) = distribution {
            dim.distribution = d;
        }

        feature.entity_revision += 1;

        let file_path = self.get_file_path(&feature.id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Remove a dimension from a feature
    pub fn remove_dimension(&self, id: &str, name: &str) -> ServiceResult<Feature> {
        let (_, mut feature) = self.find_feature(id)?;

        let original_len = feature.dimensions.len();
        feature.dimensions.retain(|d| d.name != name);

        if feature.dimensions.len() == original_len {
            return Err(ServiceError::ValidationFailed(format!(
                "Dimension '{}' not found",
                name
            )));
        }

        feature.entity_revision += 1;

        let file_path = self.get_file_path(&feature.id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Add a GD&T control to a feature
    pub fn add_gdt(&self, id: &str, control: GdtControl) -> ServiceResult<Feature> {
        let (_, mut feature) = self.find_feature(id)?;

        feature.gdt.push(control);
        feature.entity_revision += 1;

        let file_path = self.get_file_path(&feature.id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Remove a GD&T control by index
    pub fn remove_gdt(&self, id: &str, index: usize) -> ServiceResult<Feature> {
        let (_, mut feature) = self.find_feature(id)?;

        if index >= feature.gdt.len() {
            return Err(ServiceError::ValidationFailed(format!(
                "GD&T index {} out of range (0-{})",
                index,
                feature.gdt.len().saturating_sub(1)
            )));
        }

        feature.gdt.remove(index);
        feature.entity_revision += 1;

        let file_path = self.get_file_path(&feature.id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Set 3D geometry for a feature
    pub fn set_geometry_3d(&self, id: &str, geometry: Geometry3D) -> ServiceResult<Feature> {
        self.update(
            id,
            UpdateFeature {
                geometry_3d: Some(Some(geometry)),
                ..Default::default()
            },
        )
    }

    /// Set geometry class for a feature
    pub fn set_geometry_class(
        &self,
        id: &str,
        geometry_class: GeometryClass,
    ) -> ServiceResult<Feature> {
        self.update(
            id,
            UpdateFeature {
                geometry_class: Some(Some(geometry_class)),
                ..Default::default()
            },
        )
    }

    /// Set datum label for a feature
    pub fn set_datum_label(&self, id: &str, label: String) -> ServiceResult<Feature> {
        self.update(
            id,
            UpdateFeature {
                datum_label: Some(Some(label)),
                ..Default::default()
            },
        )
    }

    /// Clear datum label
    pub fn clear_datum_label(&self, id: &str) -> ServiceResult<Feature> {
        self.update(
            id,
            UpdateFeature {
                datum_label: Some(None),
                ..Default::default()
            },
        )
    }

    /// Set length from another feature's dimension
    pub fn set_length_from(
        &self,
        id: &str,
        source_feature_id: &str,
        dimension_name: &str,
    ) -> ServiceResult<Feature> {
        // Get source feature and dimension value
        let source = self.get_required(source_feature_id)?;
        let dimension_value = source.get_dimension_value(dimension_name).ok_or_else(|| {
            ServiceError::ValidationFailed(format!(
                "Dimension '{}' not found in feature '{}'",
                dimension_name, source_feature_id
            ))
        })?;

        // Get target feature
        let (_, mut feature) = self.find_feature(id)?;

        // Update geometry_3d with length and reference
        let length_ref = format!("{}:{}", source.id, dimension_name);
        match feature.geometry_3d {
            Some(ref mut geom) => {
                geom.length = Some(dimension_value);
                geom.length_ref = Some(length_ref);
            }
            None => {
                feature.geometry_3d = Some(Geometry3D {
                    origin: [0.0, 0.0, 0.0],
                    axis: [0.0, 0.0, 1.0],
                    length: Some(dimension_value),
                    length_ref: Some(length_ref),
                });
            }
        }

        feature.entity_revision += 1;

        let file_path = self.get_file_path(&feature.id);
        self.base.save(&feature, &file_path)?;

        Ok(feature)
    }

    /// Compute and set torsor bounds from GD&T controls
    pub fn compute_bounds(
        &self,
        id: &str,
        actual_size: Option<f64>,
        update: bool,
    ) -> ServiceResult<(Feature, TorsorBounds)> {
        use crate::core::gdt_torsor::compute_torsor_bounds;

        let (_, mut feature) = self.find_feature(id)?;

        // Compute bounds using the gdt_torsor module
        let result =
            compute_torsor_bounds::<fn(&str) -> Option<Feature>>(&feature, actual_size, None);

        if update {
            feature.torsor_bounds = Some(result.bounds.clone());
            feature.entity_revision += 1;

            let file_path = self.get_file_path(&feature.id);
            self.base.save(&feature, &file_path)?;
        }

        Ok((feature, result.bounds))
    }

    /// Calculate statistics
    pub fn stats(&self) -> ServiceResult<FeatureStats> {
        let features = self.load_all()?;

        let mut stats = FeatureStats {
            total: features.len(),
            ..Default::default()
        };

        for feature in &features {
            // Count by type
            match feature.feature_type {
                FeatureType::Internal => stats.by_type.internal += 1,
                FeatureType::External => stats.by_type.external += 1,
            }

            // Count by status
            match feature.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            // Count with GD&T
            if feature.has_gdt() {
                stats.with_gdt += 1;
            }

            // Count datums
            if feature.datum_label.is_some() {
                stats.datums += 1;
            }

            // Count with 3D geometry
            if feature.geometry_3d.is_some() {
                stats.with_geometry_3d += 1;
            }
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::feature::{GdtSymbol, MaterialCondition};
    use tempfile::TempDir;

    fn setup() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();
        let project = Project::init(tmp.path()).unwrap();
        let cache = EntityCache::open(&project).unwrap();
        (tmp, project, cache)
    }

    fn create_test_feature(service: &FeatureService) -> Feature {
        service
            .create(CreateFeature {
                component: "CMP-001".to_string(),
                feature_type: FeatureType::Internal,
                title: "Test Hole".to_string(),
                description: Some("A test hole".to_string()),
                dimensions: Vec::new(),
                gdt: Vec::new(),
                geometry_class: None,
                datum_label: None,
                tags: Vec::new(),
                status: None,
                author: "author".to_string(),
            })
            .unwrap()
    }

    #[test]
    fn test_create_feature() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let feature = service
            .create(CreateFeature {
                component: "CMP-001".to_string(),
                feature_type: FeatureType::External,
                title: "Test Shaft".to_string(),
                description: None,
                dimensions: Vec::new(),
                gdt: Vec::new(),
                geometry_class: Some(GeometryClass::Cylinder),
                datum_label: Some("A".to_string()),
                tags: vec!["primary".to_string()],
                status: None,
                author: "author".to_string(),
            })
            .unwrap();

        assert!(feature.id.to_string().starts_with("FEAT-"));
        assert_eq!(feature.component, "CMP-001");
        assert_eq!(feature.feature_type, FeatureType::External);
        assert_eq!(feature.geometry_class, Some(GeometryClass::Cylinder));
        assert_eq!(feature.datum_label, Some("A".to_string()));
        assert_eq!(feature.status, Status::Draft);
    }

    #[test]
    fn test_get_feature() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        let retrieved = service.get(&created.id.to_string()).unwrap().unwrap();

        assert_eq!(created.id, retrieved.id);
        assert_eq!(created.title, retrieved.title);
    }

    #[test]
    fn test_update_feature() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        let updated = service
            .update(
                &created.id.to_string(),
                UpdateFeature {
                    title: Some("Updated Hole".to_string()),
                    geometry_class: Some(Some(GeometryClass::Cylinder)),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Hole");
        assert_eq!(updated.geometry_class, Some(GeometryClass::Cylinder));
        assert_eq!(updated.entity_revision, 2);
    }

    #[test]
    fn test_delete_feature() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        service.delete(&created.id.to_string(), false).unwrap();

        let result = service.get(&created.id.to_string()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        // Create features with different types
        service
            .create(CreateFeature {
                component: "CMP-001".to_string(),
                feature_type: FeatureType::Internal,
                title: "Hole A".to_string(),
                description: None,
                dimensions: Vec::new(),
                gdt: Vec::new(),
                geometry_class: None,
                datum_label: None,
                tags: Vec::new(),
                status: None,
                author: "author".to_string(),
            })
            .unwrap();
        service
            .create(CreateFeature {
                component: "CMP-001".to_string(),
                feature_type: FeatureType::External,
                title: "Shaft A".to_string(),
                description: None,
                dimensions: Vec::new(),
                gdt: Vec::new(),
                geometry_class: None,
                datum_label: None,
                tags: Vec::new(),
                status: None,
                author: "author".to_string(),
            })
            .unwrap();

        // Filter by type
        let internal_features = service
            .list(&FeatureFilter {
                feature_type: Some(FeatureType::Internal),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(internal_features.len(), 1);
        assert_eq!(internal_features[0].feature_type, FeatureType::Internal);
    }

    #[test]
    fn test_add_dimension() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        let updated = service
            .add_dimension(
                &created.id.to_string(),
                "diameter".to_string(),
                10.0,
                0.1,
                0.05,
                true,
                None,
                None,
            )
            .unwrap();

        assert_eq!(updated.dimensions.len(), 1);
        assert_eq!(updated.dimensions[0].name, "diameter");
        assert_eq!(updated.dimensions[0].nominal, 10.0);
        assert!(updated.dimensions[0].internal);
    }

    #[test]
    fn test_remove_dimension() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        let with_dim = service
            .add_dimension(
                &created.id.to_string(),
                "diameter".to_string(),
                10.0,
                0.1,
                0.05,
                true,
                None,
                None,
            )
            .unwrap();

        let removed = service
            .remove_dimension(&with_dim.id.to_string(), "diameter")
            .unwrap();

        assert!(removed.dimensions.is_empty());
    }

    #[test]
    fn test_add_gdt() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        let updated = service
            .add_gdt(
                &created.id.to_string(),
                GdtControl {
                    symbol: GdtSymbol::Position,
                    value: 0.25,
                    units: "mm".to_string(),
                    datum_refs: vec!["A".to_string(), "B".to_string()],
                    material_condition: MaterialCondition::Mmc,
                },
            )
            .unwrap();

        assert_eq!(updated.gdt.len(), 1);
        assert_eq!(updated.gdt[0].symbol, GdtSymbol::Position);
        assert_eq!(updated.gdt[0].value, 0.25);
    }

    #[test]
    fn test_set_datum_label() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        let updated = service
            .set_datum_label(&created.id.to_string(), "A".to_string())
            .unwrap();

        assert_eq!(updated.datum_label, Some("A".to_string()));

        let cleared = service.clear_datum_label(&updated.id.to_string()).unwrap();
        assert!(cleared.datum_label.is_none());
    }

    #[test]
    fn test_set_geometry_class() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        let created = create_test_feature(&service);
        let updated = service
            .set_geometry_class(&created.id.to_string(), GeometryClass::Cylinder)
            .unwrap();

        assert_eq!(updated.geometry_class, Some(GeometryClass::Cylinder));
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup();
        let service = FeatureService::new(&project, &cache);

        // Create features
        service
            .create(CreateFeature {
                component: "CMP-001".to_string(),
                feature_type: FeatureType::Internal,
                title: "Hole".to_string(),
                description: None,
                dimensions: Vec::new(),
                gdt: vec![GdtControl {
                    symbol: GdtSymbol::Flatness,
                    value: 0.1,
                    units: "mm".to_string(),
                    datum_refs: Vec::new(),
                    material_condition: MaterialCondition::Rfs,
                }],
                geometry_class: None,
                datum_label: Some("A".to_string()),
                tags: Vec::new(),
                status: None,
                author: "author".to_string(),
            })
            .unwrap();
        service
            .create(CreateFeature {
                component: "CMP-001".to_string(),
                feature_type: FeatureType::External,
                title: "Shaft".to_string(),
                description: None,
                dimensions: Vec::new(),
                gdt: Vec::new(),
                geometry_class: None,
                datum_label: None,
                tags: Vec::new(),
                status: None,
                author: "author".to_string(),
            })
            .unwrap();

        let stats = service.stats().unwrap();

        assert_eq!(stats.total, 2);
        assert_eq!(stats.by_type.internal, 1);
        assert_eq!(stats.by_type.external, 1);
        assert_eq!(stats.with_gdt, 1);
        assert_eq!(stats.datums, 1);
    }
}
