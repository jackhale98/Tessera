//! Feature entity commands
//!
//! Provides commands for managing GD&T features.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::feature::{Feature, FeatureType, GeometryClass};
use tdt_core::entities::stackup::Distribution;
use tdt_core::services::common::SortDirection;
use tdt_core::services::feature::{
    CreateFeature, FeatureFilter, FeatureService, FeatureSortField, FeatureStats, UpdateFeature,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Feature summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct FeatureSummary {
    pub id: String,
    pub title: String,
    pub component_id: Option<String>,
    pub feature_type: String,
    pub geometry_class: Option<String>,
    pub datum_label: Option<String>,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Feature> for FeatureSummary {
    fn from(f: &Feature) -> Self {
        Self {
            id: f.id.to_string(),
            title: f.title.clone(),
            component_id: Some(f.component.clone()),
            feature_type: f.feature_type.to_string(),
            geometry_class: f.geometry_class.as_ref().map(|g| g.to_string()),
            datum_label: f.datum_label.clone(),
            status: format!("{:?}", f.status).to_lowercase(),
            author: f.author.clone(),
            created: f.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListFeaturesResult {
    pub items: Vec<FeatureSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListFeaturesParams {
    pub status: Option<Vec<String>>,
    pub component_id: Option<String>,
    pub feature_type: Option<String>,
    pub geometry_class: Option<String>,
    pub is_datum: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFeatureInput {
    pub title: String,
    pub author: String,
    pub component: String,
    pub feature_type: Option<String>,
    pub description: Option<String>,
    pub geometry_class: Option<String>,
    pub datum_label: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateFeatureInput {
    pub title: Option<String>,
    pub feature_type: Option<String>,
    pub description: Option<String>,
    pub geometry_class: Option<String>,
    pub datum_label: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddDimensionInput {
    pub name: String,
    pub nominal: f64,
    pub plus_tol: f64,
    pub minus_tol: f64,
    pub internal: Option<bool>,
    pub units: Option<String>,
    pub distribution: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddGdtInput {
    pub control_type: String,
    pub tolerance: f64,
    pub datum_refs: Option<Vec<String>>,
    pub material_condition: Option<String>,
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

fn parse_feature_type(s: &str) -> Option<FeatureType> {
    match s.to_lowercase().as_str() {
        "internal" => Some(FeatureType::Internal),
        "external" => Some(FeatureType::External),
        _ => None,
    }
}

fn parse_geometry_class(s: &str) -> Option<GeometryClass> {
    match s.to_lowercase().as_str() {
        "plane" | "planar" => Some(GeometryClass::Plane),
        "cylinder" | "cylindrical" => Some(GeometryClass::Cylinder),
        "sphere" | "spherical" => Some(GeometryClass::Sphere),
        "cone" | "conical" => Some(GeometryClass::Cone),
        "point" => Some(GeometryClass::Point),
        "line" => Some(GeometryClass::Line),
        "complex" => Some(GeometryClass::Complex),
        _ => None,
    }
}

fn parse_distribution(s: &str) -> Option<Distribution> {
    match s.to_lowercase().as_str() {
        "normal" => Some(Distribution::Normal),
        "uniform" => Some(Distribution::Uniform),
        "triangular" => Some(Distribution::Triangular),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> FeatureSortField {
    match s.to_lowercase().as_str() {
        "id" => FeatureSortField::Id,
        "title" => FeatureSortField::Title,
        "component" => FeatureSortField::Component,
        "type" => FeatureSortField::Type,
        "status" => FeatureSortField::Status,
        "author" => FeatureSortField::Author,
        _ => FeatureSortField::Created,
    }
}

fn build_feature_filter(params: &ListFeaturesParams) -> FeatureFilter {
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
        search: params.search.clone(),
        limit: params.limit,
        ..Default::default()
    };

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

    FeatureFilter {
        common,
        component: params.component_id.clone(),
        feature_type: params
            .feature_type
            .as_ref()
            .and_then(|t| parse_feature_type(t)),
        geometry_class: params
            .geometry_class
            .as_ref()
            .and_then(|g| parse_geometry_class(g)),
        is_datum: params.is_datum,
        has_gdt: None,
        sort,
        sort_direction,
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_features(
    params: Option<ListFeaturesParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListFeaturesResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = FeatureService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_feature_filter(&params);
    let features = service.list(&filter)?;

    Ok(ListFeaturesResult {
        total_count: features.len(),
        items: features.iter().map(FeatureSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_feature(id: String, state: State<'_, AppState>) -> CommandResult<Option<Feature>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = FeatureService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn get_features_by_component(
    component_id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<Feature>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = FeatureService::new(project, cache);
    Ok(service.get_by_component(&component_id)?)
}

#[tauri::command]
pub async fn create_feature(
    input: CreateFeatureInput,
    state: State<'_, AppState>,
) -> CommandResult<Feature> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let feature = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = FeatureService::new(project, cache);

        let create = CreateFeature {
            title: input.title,
            author: input.author,
            component: input.component,
            feature_type: input
                .feature_type
                .and_then(|t| parse_feature_type(&t))
                .unwrap_or(FeatureType::External),
            description: input.description,
            dimensions: Vec::new(),
            gdt: Vec::new(),
            geometry_class: input.geometry_class.and_then(|g| parse_geometry_class(&g)),
            datum_label: input.datum_label,
            tags: input.tags.unwrap_or_default(),
            status: None,
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(feature)
}

#[tauri::command]
pub async fn update_feature(
    id: String,
    input: UpdateFeatureInput,
    state: State<'_, AppState>,
) -> CommandResult<Feature> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let feature = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = FeatureService::new(project, cache);

        let update = UpdateFeature {
            title: input.title,
            feature_type: input.feature_type.and_then(|t| parse_feature_type(&t)),
            description: input.description.map(Some),
            geometry_class: input.geometry_class.map(|g| parse_geometry_class(&g)),
            datum_label: input.datum_label.map(Some),
            geometry_3d: None,
            torsor_bounds: None,
            status: input.status.and_then(|s| parse_status(&s)),
            tags: input.tags,
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(feature)
}

#[tauri::command]
pub async fn delete_feature(
    id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = FeatureService::new(project, cache);
        service.delete(&id, force.unwrap_or(false))?;
    }

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

#[tauri::command]
pub async fn add_feature_dimension(
    id: String,
    input: AddDimensionInput,
    state: State<'_, AppState>,
) -> CommandResult<Feature> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let feature = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = FeatureService::new(project, cache);
        service.add_dimension(
            &id,
            input.name,
            input.nominal,
            input.plus_tol,
            input.minus_tol,
            input.internal.unwrap_or(false),
            input.units,
            input.distribution.and_then(|d| parse_distribution(&d)),
        )?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(feature)
}

#[tauri::command]
pub async fn remove_feature_dimension(
    id: String,
    name: String,
    state: State<'_, AppState>,
) -> CommandResult<Feature> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let feature = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = FeatureService::new(project, cache);
        service.remove_dimension(&id, &name)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(feature)
}

#[tauri::command]
pub async fn set_feature_datum_label(
    id: String,
    label: String,
    state: State<'_, AppState>,
) -> CommandResult<Feature> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let feature = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = FeatureService::new(project, cache);
        service.set_datum_label(&id, label)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(feature)
}

#[tauri::command]
pub async fn clear_feature_datum_label(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Feature> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let feature = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = FeatureService::new(project, cache);
        service.clear_datum_label(&id)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(feature)
}

#[tauri::command]
pub async fn get_feature_stats(state: State<'_, AppState>) -> CommandResult<FeatureStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = FeatureService::new(project, cache);
    Ok(service.stats()?)
}
