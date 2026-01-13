//! Traceability and link management commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::core::{
    cache::{CachedLink, EntityFilter},
    identity::EntityPrefix,
};
use tdt_core::services::{
    CoverageReport, DesignStructureMatrix, TraceDirection, TraceOptions, TraceResult,
    TraceabilityService,
};

/// Get directory name for an entity prefix
fn entity_dir_name(prefix: EntityPrefix) -> &'static str {
    match prefix {
        EntityPrefix::Req => "requirements",
        EntityPrefix::Risk => "risks",
        EntityPrefix::Test => "tests",
        EntityPrefix::Rslt => "results",
        EntityPrefix::Cmp => "components",
        EntityPrefix::Asm => "assemblies",
        EntityPrefix::Feat => "features",
        EntityPrefix::Mate => "mates",
        EntityPrefix::Tol => "stackups",
        EntityPrefix::Proc => "processes",
        EntityPrefix::Ctrl => "controls",
        EntityPrefix::Work => "work_instructions",
        EntityPrefix::Lot => "lots",
        EntityPrefix::Dev => "deviations",
        EntityPrefix::Ncr => "ncrs",
        EntityPrefix::Capa => "capas",
        EntityPrefix::Quot => "quotes",
        EntityPrefix::Sup => "suppliers",
        EntityPrefix::Haz => "hazards",
        EntityPrefix::Act => "actions",
    }
}

/// Link information for frontend
#[derive(Debug, Clone, Serialize)]
pub struct LinkInfo {
    pub source_id: String,
    pub target_id: String,
    pub link_type: String,
    pub target_title: Option<String>,
    pub target_status: Option<String>,
}

impl From<&CachedLink> for LinkInfo {
    fn from(link: &CachedLink) -> Self {
        Self {
            source_id: link.source_id.clone(),
            target_id: link.target_id.clone(),
            link_type: link.link_type.clone(),
            target_title: None,
            target_status: None,
        }
    }
}

/// Parameters for trace query
#[derive(Debug, Clone, Deserialize)]
pub struct TraceParams {
    pub id: String,
    pub direction: Option<String>,
    pub depth: Option<usize>,
    pub link_types: Option<Vec<String>>,
}

/// Get all links from an entity
#[tauri::command]
pub async fn get_links_from(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<LinkInfo>> {
    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let links = cache.get_links_from(&id);

    // Enrich with target info
    let enriched: Vec<LinkInfo> = links
        .iter()
        .map(|link| {
            let target = cache.get_entity(&link.target_id);
            LinkInfo {
                source_id: link.source_id.clone(),
                target_id: link.target_id.clone(),
                link_type: link.link_type.clone(),
                target_title: target.as_ref().map(|e| e.title.clone()),
                target_status: target.as_ref().map(|e| format!("{:?}", e.status).to_lowercase()),
            }
        })
        .collect();

    Ok(enriched)
}

/// Get all links to an entity
#[tauri::command]
pub async fn get_links_to(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Vec<LinkInfo>> {
    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let links = cache.get_links_to(&id);

    // Enrich with source info
    let enriched: Vec<LinkInfo> = links
        .iter()
        .map(|link| {
            let source = cache.get_entity(&link.source_id);
            LinkInfo {
                source_id: link.source_id.clone(),
                target_id: link.target_id.clone(),
                link_type: link.link_type.clone(),
                target_title: source.as_ref().map(|e| e.title.clone()),
                target_status: source.as_ref().map(|e| format!("{:?}", e.status).to_lowercase()),
            }
        })
        .collect();

    Ok(enriched)
}

/// Trace from an entity (find downstream)
#[tauri::command]
pub async fn trace_from(
    params: TraceParams,
    state: State<'_, AppState>,
) -> CommandResult<TraceResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TraceabilityService::new(project, cache);

    let options = TraceOptions {
        direction: Some(TraceDirection::Forward),
        max_depth: params.depth,
        link_types: params.link_types,
        entity_types: None,
        include_source: true,
    };

    let result = service.trace_from(&params.id, &options)
        .map_err(|e| CommandError::Other(e.to_string()))?;
    Ok(result)
}

/// Trace to an entity (find upstream)
#[tauri::command]
pub async fn trace_to(
    params: TraceParams,
    state: State<'_, AppState>,
) -> CommandResult<TraceResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TraceabilityService::new(project, cache);

    let options = TraceOptions {
        direction: Some(TraceDirection::Backward),
        max_depth: params.depth,
        link_types: params.link_types,
        entity_types: None,
        include_source: true,
    };

    let result = service.trace_to(&params.id, &options)
        .map_err(|e| CommandError::Other(e.to_string()))?;
    Ok(result)
}

/// Get coverage report
#[tauri::command]
pub async fn get_coverage_report(
    state: State<'_, AppState>,
) -> CommandResult<CoverageReport> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TraceabilityService::new(project, cache);
    let report = service.get_coverage();

    Ok(report)
}

/// Get design structure matrix
#[tauri::command]
pub async fn get_dsm(
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<DesignStructureMatrix> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TraceabilityService::new(project, cache);

    let prefixes: Vec<EntityPrefix> = entity_type
        .and_then(|s| match s.to_uppercase().as_str() {
            "REQ" => Some(EntityPrefix::Req),
            "RISK" => Some(EntityPrefix::Risk),
            "TEST" => Some(EntityPrefix::Test),
            "CMP" => Some(EntityPrefix::Cmp),
            _ => None,
        })
        .map(|p| vec![p])
        .unwrap_or_default();

    let dsm = service.generate_dsm(&prefixes);
    Ok(dsm)
}

/// Find orphan entities (no links)
#[tauri::command]
pub async fn find_orphans(
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<String>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TraceabilityService::new(project, cache);

    let prefix = entity_type.and_then(|s| match s.to_uppercase().as_str() {
        "REQ" => Some(EntityPrefix::Req),
        "RISK" => Some(EntityPrefix::Risk),
        "TEST" => Some(EntityPrefix::Test),
        "CMP" => Some(EntityPrefix::Cmp),
        _ => None,
    });

    let orphans = service.find_orphans(prefix);
    // Extract just the IDs from TracedEntity
    Ok(orphans.into_iter().map(|e| e.id).collect())
}

/// Find circular dependencies
#[tauri::command]
pub async fn find_cycles(
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<Vec<String>>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = TraceabilityService::new(project, cache);

    let prefix = entity_type.and_then(|s| match s.to_uppercase().as_str() {
        "REQ" => Some(EntityPrefix::Req),
        "RISK" => Some(EntityPrefix::Risk),
        "TEST" => Some(EntityPrefix::Test),
        "CMP" => Some(EntityPrefix::Cmp),
        _ => None,
    });

    let cycles = service.find_cycles(prefix);

    Ok(cycles)
}

/// Add a link between entities using inferred link type
#[tauri::command]
pub async fn add_link(
    source_id: String,
    target_id: String,
    _link_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project = state.project.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;

    use tdt_core::core::{identity::EntityId, links, loader};

    // Parse IDs to get prefixes
    let source_entity_id: EntityId = source_id
        .parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid source ID: {}", source_id)))?;
    let target_entity_id: EntityId = target_id
        .parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid target ID: {}", target_id)))?;

    // Find the source entity file
    let source_dir = project.root().join(entity_dir_name(source_entity_id.prefix()));
    let source_path = loader::find_entity_file(&source_dir, &source_id)
        .ok_or_else(|| CommandError::NotFound(format!("Source entity not found: {}", source_id)))?;

    // Use add_inferred_link to add the link
    links::add_inferred_link(
        &source_path,
        source_entity_id.prefix(),
        &target_id,
        target_entity_id.prefix(),
    )
    .map_err(|e| CommandError::Other(e))?;

    Ok(())
}

/// Remove a link between entities
/// Note: This currently requires editing entity YAML files directly.
/// For now, use the generic save_entity command to update links.
#[tauri::command]
pub async fn remove_link(
    _source_id: String,
    _target_id: String,
    _link_type: Option<String>,
    _state: State<'_, AppState>,
) -> CommandResult<()> {
    // TODO: Implement remove_link when tdt-core exports the necessary functions
    // For now, links can be removed by editing the entity via save_entity
    Err(CommandError::Other(
        "Link removal not yet implemented. Use save_entity to update entity links.".to_string(),
    ))
}

/// Get all link types used in the project
#[tauri::command]
pub async fn get_link_types(
    state: State<'_, AppState>,
) -> CommandResult<Vec<String>> {
    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    // Get all entities and collect unique link types
    let filter = EntityFilter::default();
    let entities = cache.list_entities(&filter);

    let mut link_types = std::collections::HashSet::new();
    for entity in entities {
        let links = cache.get_links_from(&entity.id);
        for link in links {
            link_types.insert(link.link_type.clone());
        }
    }

    let mut types: Vec<String> = link_types.into_iter().collect();
    types.sort();

    Ok(types)
}
