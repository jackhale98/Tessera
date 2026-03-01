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
    CoverageReport, DesignStructureMatrix, TraceabilityService,
};

use super::entity_dir_name;

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
                target_status: target
                    .as_ref()
                    .map(|e| format!("{:?}", e.status).to_lowercase()),
            }
        })
        .collect();

    Ok(enriched)
}

/// Get all links to an entity
#[tauri::command]
pub async fn get_links_to(id: String, state: State<'_, AppState>) -> CommandResult<Vec<LinkInfo>> {
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
                target_status: source
                    .as_ref()
                    .map(|e| format!("{:?}", e.status).to_lowercase()),
            }
        })
        .collect();

    Ok(enriched)
}

/// A node in the trace graph
#[derive(Debug, Clone, Serialize)]
pub struct TraceNode {
    pub id: String,
    pub entity_type: String,
    pub title: String,
    pub status: String,
    pub depth: i32,
}

/// A directed edge in the trace graph (always from upstream to downstream)
#[derive(Debug, Clone, Serialize)]
pub struct TraceEdge {
    pub from_id: String,
    pub to_id: String,
    pub link_type: String,
}

/// Frontend-compatible trace result — a graph of nodes and directed edges
#[derive(Debug, Clone, Serialize)]
pub struct FrontendTraceResult {
    pub root_id: String,
    pub nodes: Vec<TraceNode>,
    pub edges: Vec<TraceEdge>,
}

/// Trace from an entity — returns a graph of upstream and downstream entities
/// with edges always pointing from upstream to downstream.
///
/// Uses `is_downstream_link` to classify link types:
/// - `verified_by`, `satisfied_by`, etc. → target is downstream
/// - `verifies`, `component`, `requirement`, etc. → target is upstream
#[tauri::command]
pub async fn trace_from(
    params: TraceParams,
    state: State<'_, AppState>,
) -> CommandResult<FrontendTraceResult> {
    use std::collections::{HashMap, HashSet, VecDeque};
    use tdt_core::core::links::is_downstream_link;

    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let root = cache
        .get_entity(&params.id)
        .ok_or_else(|| CommandError::NotFound(format!("Entity not found: {}", params.id)))?;

    let max_depth = params.depth.unwrap_or(5) as i32;

    let mut nodes: HashMap<String, TraceNode> = HashMap::new();
    let mut edges: Vec<TraceEdge> = Vec::new();
    let mut seen_edges: HashSet<(String, String)> = HashSet::new();

    // Add root node
    nodes.insert(
        root.id.clone(),
        TraceNode {
            id: root.id.clone(),
            entity_type: root.id.split('-').next().unwrap_or("").to_string(),
            title: root.title.clone(),
            status: format!("{:?}", root.status).to_lowercase(),
            depth: 0,
        },
    );

    // Helper: try to add an edge and node, returns true if the neighbor is new
    fn try_add_edge(
        upstream_id: &str,
        downstream_id: &str,
        link_type: &str,
        neighbor_id: &str,
        neighbor_depth: i32,
        cache: &tdt_core::core::cache::EntityCache,
        nodes: &mut HashMap<String, TraceNode>,
        edges: &mut Vec<TraceEdge>,
        seen_edges: &mut HashSet<(String, String)>,
    ) -> bool {
        let edge_key = (upstream_id.to_string(), downstream_id.to_string());
        let reverse_key = (downstream_id.to_string(), upstream_id.to_string());
        if seen_edges.contains(&edge_key) || seen_edges.contains(&reverse_key) {
            return false;
        }
        seen_edges.insert(edge_key);
        edges.push(TraceEdge {
            from_id: upstream_id.to_string(),
            to_id: downstream_id.to_string(),
            link_type: link_type.to_string(),
        });
        if !nodes.contains_key(neighbor_id) {
            if let Some(entity) = cache.get_entity(neighbor_id) {
                nodes.insert(
                    neighbor_id.to_string(),
                    TraceNode {
                        id: entity.id.clone(),
                        entity_type: entity.id.split('-').next().unwrap_or("").to_string(),
                        title: entity.title.clone(),
                        status: format!("{:?}", entity.status).to_lowercase(),
                        depth: neighbor_depth,
                    },
                );
            }
        }
        true
    }

    // BFS downstream
    let mut queue: VecDeque<(String, i32)> = VecDeque::new();
    queue.push_back((params.id.clone(), 0));
    let mut visited_down: HashSet<String> = HashSet::new();
    visited_down.insert(params.id.clone());

    while let Some((current_id, current_depth)) = queue.pop_front() {
        if current_depth >= max_depth {
            continue;
        }

        // Downstream via links_from: downstream link types → target is downstream
        for link in cache.get_links_from(&current_id) {
            if is_downstream_link(&link.link_type) {
                let added = try_add_edge(
                    &current_id,
                    &link.target_id,
                    &link.link_type,
                    &link.target_id,
                    current_depth + 1,
                    cache,
                    &mut nodes,
                    &mut edges,
                    &mut seen_edges,
                );
                if added && !visited_down.contains(&link.target_id) {
                    visited_down.insert(link.target_id.clone());
                    queue.push_back((link.target_id.clone(), current_depth + 1));
                }
            }
        }

        // Downstream via links_to: upstream link types pointing to us → source is downstream
        for link in cache.get_links_to(&current_id) {
            if !is_downstream_link(&link.link_type) {
                let added = try_add_edge(
                    &current_id,
                    &link.source_id,
                    &link.link_type,
                    &link.source_id,
                    current_depth + 1,
                    cache,
                    &mut nodes,
                    &mut edges,
                    &mut seen_edges,
                );
                if added && !visited_down.contains(&link.source_id) {
                    visited_down.insert(link.source_id.clone());
                    queue.push_back((link.source_id.clone(), current_depth + 1));
                }
            }
        }
    }

    // BFS upstream
    let mut queue: VecDeque<(String, i32)> = VecDeque::new();
    queue.push_back((params.id.clone(), 0));
    let mut visited_up: HashSet<String> = HashSet::new();
    visited_up.insert(params.id.clone());

    while let Some((current_id, current_depth)) = queue.pop_front() {
        if current_depth >= max_depth {
            continue;
        }

        // Upstream via links_from: upstream link types → target is upstream
        for link in cache.get_links_from(&current_id) {
            if !is_downstream_link(&link.link_type) {
                let added = try_add_edge(
                    &link.target_id,
                    &current_id,
                    &link.link_type,
                    &link.target_id,
                    -(current_depth + 1),
                    cache,
                    &mut nodes,
                    &mut edges,
                    &mut seen_edges,
                );
                if added && !visited_up.contains(&link.target_id) {
                    visited_up.insert(link.target_id.clone());
                    queue.push_back((link.target_id.clone(), current_depth + 1));
                }
            }
        }

        // Upstream via links_to: downstream link types pointing to us → source is upstream
        for link in cache.get_links_to(&current_id) {
            if is_downstream_link(&link.link_type) {
                let added = try_add_edge(
                    &link.source_id,
                    &current_id,
                    &link.link_type,
                    &link.source_id,
                    -(current_depth + 1),
                    cache,
                    &mut nodes,
                    &mut edges,
                    &mut seen_edges,
                );
                if added && !visited_up.contains(&link.source_id) {
                    visited_up.insert(link.source_id.clone());
                    queue.push_back((link.source_id.clone(), current_depth + 1));
                }
            }
        }
    }

    Ok(FrontendTraceResult {
        root_id: params.id.clone(),
        nodes: nodes.into_values().collect(),
        edges,
    })
}

/// Trace to an entity — returns same graph as trace_from
#[tauri::command]
pub async fn trace_to(
    params: TraceParams,
    state: State<'_, AppState>,
) -> CommandResult<FrontendTraceResult> {
    trace_from(params, state).await
}

/// Get coverage report
#[tauri::command]
pub async fn get_coverage_report(state: State<'_, AppState>) -> CommandResult<CoverageReport> {
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

    // If a specific type is requested, use that; otherwise include all main types
    let prefixes: Vec<EntityPrefix> = match entity_type {
        Some(ref s) => match s.to_uppercase().as_str() {
            "REQ" => vec![EntityPrefix::Req],
            "RISK" => vec![EntityPrefix::Risk],
            "TEST" => vec![EntityPrefix::Test],
            "CMP" => vec![EntityPrefix::Cmp],
            "ASM" => vec![EntityPrefix::Asm],
            "HAZ" => vec![EntityPrefix::Haz],
            _ => vec![],
        },
        None => vec![
            EntityPrefix::Req,
            EntityPrefix::Risk,
            EntityPrefix::Test,
            EntityPrefix::Cmp,
            EntityPrefix::Asm,
            EntityPrefix::Haz,
        ],
    };

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

/// Entity info within a cycle
#[derive(Debug, Clone, Serialize)]
pub struct CycleEntity {
    pub id: String,
    pub entity_type: String,
    pub title: String,
}

/// Find circular dependencies — returns enriched cycle data, filtering out
/// 2-cycles caused by reciprocal links (e.g. REQ→TEST→REQ via verified_by/verifies)
#[tauri::command]
pub async fn find_cycles(
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<Vec<CycleEntity>>> {
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

    let raw_cycles = service.find_cycles(prefix);

    // Filter out 2-cycles (almost always reciprocal link pairs, not real circular deps)
    // and enrich with entity info
    let enriched: Vec<Vec<CycleEntity>> = raw_cycles
        .into_iter()
        .filter(|cycle| cycle.len() > 2)
        .map(|cycle| {
            cycle
                .iter()
                .map(|id| {
                    let entity = cache.get_entity(id);
                    CycleEntity {
                        id: id.clone(),
                        entity_type: id.split('-').next().unwrap_or("").to_string(),
                        title: entity.map(|e| e.title.clone()).unwrap_or_default(),
                    }
                })
                .collect()
        })
        .collect();

    Ok(enriched)
}

/// Add a link between entities with automatic reciprocal link creation
///
/// If `link_type` is provided, uses explicit linking with that type.
/// Otherwise, infers the link type based on entity prefixes.
/// Also creates the reciprocal link on the target entity if one is defined.
#[tauri::command(rename_all = "camelCase")]
pub async fn add_link(
    source_id: String,
    target_id: String,
    link_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    use tdt_core::core::{identity::EntityId, links, loader};

    // Parse IDs to get prefixes
    let source_entity_id: EntityId = source_id
        .parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid source ID: {}", source_id)))?;
    let target_entity_id: EntityId = target_id
        .parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid target ID: {}", target_id)))?;

    // Find the source entity file
    let source_dir = project
        .root()
        .join(entity_dir_name(source_entity_id.prefix()));
    let source_path = loader::find_entity_file(&source_dir, &source_id)
        .ok_or_else(|| CommandError::NotFound(format!("Source entity not found: {}", source_id)))?;

    // Determine the actual link type used (either explicit or inferred)
    let actual_link_type: String;

    // Use explicit link type if provided, otherwise infer
    if let Some(explicit_type) = link_type {
        links::add_explicit_link(&source_path, &explicit_type, &target_id)
            .map_err(CommandError::Other)?;
        actual_link_type = explicit_type;
    } else {
        actual_link_type = links::add_inferred_link(
            &source_path,
            source_entity_id.prefix(),
            &target_id,
            target_entity_id.prefix(),
        )
        .map_err(CommandError::Other)?;
    }

    // Now add the reciprocal link on the target entity
    if let Some(reciprocal_type) = links::get_reciprocal_link_type(
        &actual_link_type,
        target_entity_id.prefix(),
        source_entity_id.prefix(),
    ) {
        // Find the target entity file
        let target_dir = project
            .root()
            .join(entity_dir_name(target_entity_id.prefix()));
        if let Some(target_path) = loader::find_entity_file(&target_dir, &target_id) {
            // Add reciprocal link (ignore errors - target may not support this link type)
            let _ = links::add_explicit_link(&target_path, &reciprocal_type, &source_id);
        }
    }

    // Sync cache to pick up the link changes
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Remove a link between entities with automatic reciprocal link removal
///
/// Removes the specified link from the source entity and also removes
/// the reciprocal link from the target entity if one exists.
#[tauri::command(rename_all = "camelCase")]
pub async fn remove_link(
    source_id: String,
    target_id: String,
    link_type: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    use tdt_core::core::{identity::EntityId, links, loader};

    // Parse IDs to get prefixes
    let source_entity_id: EntityId = source_id
        .parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid source ID: {}", source_id)))?;
    let target_entity_id: EntityId = target_id
        .parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid target ID: {}", target_id)))?;

    // Find the source entity file
    let source_dir = project
        .root()
        .join(entity_dir_name(source_entity_id.prefix()));
    let source_path = loader::find_entity_file(&source_dir, &source_id)
        .ok_or_else(|| CommandError::NotFound(format!("Source entity not found: {}", source_id)))?;

    // Remove the link from source entity
    links::remove_explicit_link(&source_path, &link_type, &target_id)
        .map_err(CommandError::Other)?;

    // Now remove the reciprocal link from the target entity
    if let Some(reciprocal_type) = links::get_reciprocal_link_type(
        &link_type,
        target_entity_id.prefix(),
        source_entity_id.prefix(),
    ) {
        // Find the target entity file
        let target_dir = project
            .root()
            .join(entity_dir_name(target_entity_id.prefix()));
        if let Some(target_path) = loader::find_entity_file(&target_dir, &target_id) {
            // Remove reciprocal link (ignore errors - it may not exist)
            let _ = links::remove_explicit_link(&target_path, &reciprocal_type, &source_id);
        }
    }

    // Sync cache to pick up the link changes
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Domain Mapping Matrix result
#[derive(Debug, Clone, Serialize)]
pub struct DmmResult {
    /// Row entity type (e.g., "REQ")
    pub row_type: String,
    /// Column entity type (e.g., "TEST")
    pub col_type: String,
    /// Row entities
    pub row_entities: Vec<DmmEntity>,
    /// Column entities
    pub col_entities: Vec<DmmEntity>,
    /// Links between row and column entities
    pub links: Vec<DmmLink>,
    /// Coverage statistics
    pub coverage: DmmCoverage,
}

/// Entity info for DMM
#[derive(Debug, Clone, Serialize)]
pub struct DmmEntity {
    pub id: String,
    pub title: String,
}

/// Link in the DMM
#[derive(Debug, Clone, Serialize)]
pub struct DmmLink {
    pub row_id: String,
    pub col_id: String,
}

/// Coverage statistics for DMM
#[derive(Debug, Clone, Serialize)]
pub struct DmmCoverage {
    pub row_coverage_pct: f64,
    pub col_coverage_pct: f64,
    pub rows_with_links: usize,
    pub total_rows: usize,
    pub cols_with_links: usize,
    pub total_cols: usize,
    pub total_links: usize,
}

/// Helper to convert string to EntityPrefix
fn parse_entity_type(s: &str) -> Option<EntityPrefix> {
    match s.to_uppercase().as_str() {
        "REQ" => Some(EntityPrefix::Req),
        "RISK" => Some(EntityPrefix::Risk),
        "TEST" => Some(EntityPrefix::Test),
        "CMP" => Some(EntityPrefix::Cmp),
        "ASM" => Some(EntityPrefix::Asm),
        "HAZ" => Some(EntityPrefix::Haz),
        "PROC" => Some(EntityPrefix::Proc),
        "CTRL" => Some(EntityPrefix::Ctrl),
        "FEAT" => Some(EntityPrefix::Feat),
        "MATE" => Some(EntityPrefix::Mate),
        "TOL" => Some(EntityPrefix::Tol),
        "DEV" => Some(EntityPrefix::Dev),
        "NCR" => Some(EntityPrefix::Ncr),
        "CAPA" => Some(EntityPrefix::Capa),
        "LOT" => Some(EntityPrefix::Lot),
        "WORK" => Some(EntityPrefix::Work),
        "QUOT" => Some(EntityPrefix::Quot),
        "SUP" => Some(EntityPrefix::Sup),
        _ => None,
    }
}

/// Get domain mapping matrix (relationships between two entity types)
#[tauri::command(rename_all = "camelCase")]
pub async fn get_dmm(
    row_type: String,
    col_type: String,
    state: State<'_, AppState>,
) -> CommandResult<DmmResult> {
    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let row_prefix = parse_entity_type(&row_type)
        .ok_or_else(|| CommandError::InvalidInput(format!("Invalid row type: {}", row_type)))?;
    let col_prefix = parse_entity_type(&col_type)
        .ok_or_else(|| CommandError::InvalidInput(format!("Invalid col type: {}", col_type)))?;

    if row_prefix == col_prefix {
        return Err(CommandError::InvalidInput(
            "Row and column types must be different. Use DSM for same-type analysis.".to_string(),
        ));
    }

    let row_prefix_str = row_type.to_uppercase();
    let col_prefix_str = col_type.to_uppercase();

    // Get entities for rows
    let filter = EntityFilter::default();
    let all_entities = cache.list_entities(&filter);

    let row_entities: Vec<DmmEntity> = all_entities
        .iter()
        .filter(|e| e.id.starts_with(&row_prefix_str))
        .map(|e| DmmEntity {
            id: e.id.clone(),
            title: e.title.clone(),
        })
        .collect();

    let col_entities: Vec<DmmEntity> = all_entities
        .iter()
        .filter(|e| e.id.starts_with(&col_prefix_str))
        .map(|e| DmmEntity {
            id: e.id.clone(),
            title: e.title.clone(),
        })
        .collect();

    // Build links
    let mut links: Vec<DmmLink> = Vec::new();
    let mut row_ids_with_links: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut col_ids_with_links: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // Check outgoing links from row entities to col entities
    for row_entity in &row_entities {
        let entity_links = cache.get_links_from(&row_entity.id);
        for link in entity_links {
            if link.target_id.starts_with(&col_prefix_str) {
                links.push(DmmLink {
                    row_id: row_entity.id.clone(),
                    col_id: link.target_id.clone(),
                });
                row_ids_with_links.insert(row_entity.id.clone());
                col_ids_with_links.insert(link.target_id.clone());
            }
        }
        // Check incoming links from col entities
        let reverse_links = cache.get_links_to(&row_entity.id);
        for link in reverse_links {
            if link.source_id.starts_with(&col_prefix_str) {
                links.push(DmmLink {
                    row_id: row_entity.id.clone(),
                    col_id: link.source_id.clone(),
                });
                row_ids_with_links.insert(row_entity.id.clone());
                col_ids_with_links.insert(link.source_id.clone());
            }
        }
    }

    // Also check from column entities for bidirectional coverage
    for col_entity in &col_entities {
        let entity_links = cache.get_links_from(&col_entity.id);
        for link in entity_links {
            if link.target_id.starts_with(&row_prefix_str) {
                // Add if not already present
                let link_exists = links
                    .iter()
                    .any(|l| l.row_id == link.target_id && l.col_id == col_entity.id);
                if !link_exists {
                    links.push(DmmLink {
                        row_id: link.target_id.clone(),
                        col_id: col_entity.id.clone(),
                    });
                    row_ids_with_links.insert(link.target_id.clone());
                    col_ids_with_links.insert(col_entity.id.clone());
                }
            }
        }
    }

    let total_rows = row_entities.len();
    let total_cols = col_entities.len();
    let rows_with_links = row_ids_with_links.len();
    let cols_with_links = col_ids_with_links.len();

    let row_coverage_pct = if total_rows > 0 {
        (rows_with_links as f64 / total_rows as f64) * 100.0
    } else {
        0.0
    };

    let col_coverage_pct = if total_cols > 0 {
        (cols_with_links as f64 / total_cols as f64) * 100.0
    } else {
        0.0
    };

    let total_links = links.len();

    Ok(DmmResult {
        row_type: row_prefix_str,
        col_type: col_prefix_str,
        row_entities,
        col_entities,
        links,
        coverage: DmmCoverage {
            row_coverage_pct,
            col_coverage_pct,
            rows_with_links,
            total_rows,
            cols_with_links,
            total_cols,
            total_links,
        },
    })
}

/// A maturity mismatch: a source entity is more mature than its target
#[derive(Debug, Clone, Serialize)]
pub struct MaturityMismatch {
    pub source_id: String,
    pub source_title: String,
    pub source_status: String,
    pub target_id: String,
    pub target_title: String,
    pub target_status: String,
    pub link_type: String,
}

/// Get maturity mismatches — entities linked to less-mature targets
///
/// Mirrors the CLI `tdt validate` maturity check: skip draft/obsolete sources,
/// flag when source.status > target.status (excluding obsolete targets).
#[tauri::command]
pub async fn get_maturity_mismatches(
    state: State<'_, AppState>,
) -> CommandResult<Vec<MaturityMismatch>> {
    use tdt_core::core::entity::Status;

    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let filter = EntityFilter::default();
    let entities = cache.list_entities(&filter);

    let mut mismatches = Vec::new();

    for entity in &entities {
        // Only check entities above Draft and not Obsolete
        if entity.status <= Status::Draft || entity.status == Status::Obsolete {
            continue;
        }

        let links = cache.get_links_from(&entity.id);
        for link in &links {
            if let Some(target) = cache.get_entity(&link.target_id) {
                // Flag if source is more mature than target (excluding obsolete targets)
                if target.status != Status::Obsolete && entity.status > target.status {
                    mismatches.push(MaturityMismatch {
                        source_id: entity.id.clone(),
                        source_title: entity.title.clone(),
                        source_status: format!("{:?}", entity.status).to_lowercase(),
                        target_id: target.id.clone(),
                        target_title: target.title.clone(),
                        target_status: format!("{:?}", target.status).to_lowercase(),
                        link_type: link.link_type.clone(),
                    });
                }
            }
        }
    }

    Ok(mismatches)
}

/// Get all link types used in the project
#[tauri::command]
pub async fn get_link_types(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
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
