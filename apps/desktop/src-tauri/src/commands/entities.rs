//! Generic entity commands for all entity types
//! Provides CRUD operations using the entity cache directly

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;
use tdt_core::core::{
    cache::EntityFilter,
    identity::{EntityId, EntityPrefix},
    loader,
};

/// Generic entity data returned to frontend
#[derive(Debug, Clone, Serialize)]
pub struct EntityData {
    pub id: String,
    pub prefix: String,
    pub title: String,
    pub status: String,
    pub author: String,
    pub created: String,
    pub tags: Vec<String>,
    /// Full entity data as JSON for detailed view
    pub data: Value,
}

/// List result for any entity type
#[derive(Debug, Clone, Serialize)]
pub struct EntityListResult {
    pub items: Vec<EntityData>,
    pub total_count: usize,
}

/// Parameters for listing entities
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListEntitiesParams {
    pub entity_type: String,
    pub status: Option<Vec<String>>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

fn prefix_from_string(s: &str) -> Option<EntityPrefix> {
    match s.to_uppercase().as_str() {
        "REQ" => Some(EntityPrefix::Req),
        "RISK" => Some(EntityPrefix::Risk),
        "TEST" => Some(EntityPrefix::Test),
        "RSLT" => Some(EntityPrefix::Rslt),
        "CMP" => Some(EntityPrefix::Cmp),
        "ASM" => Some(EntityPrefix::Asm),
        "FEAT" => Some(EntityPrefix::Feat),
        "MATE" => Some(EntityPrefix::Mate),
        "TOL" => Some(EntityPrefix::Tol),
        "PROC" => Some(EntityPrefix::Proc),
        "CTRL" => Some(EntityPrefix::Ctrl),
        "WORK" => Some(EntityPrefix::Work),
        "LOT" => Some(EntityPrefix::Lot),
        "DEV" => Some(EntityPrefix::Dev),
        "NCR" => Some(EntityPrefix::Ncr),
        "CAPA" => Some(EntityPrefix::Capa),
        "QUOT" => Some(EntityPrefix::Quot),
        "SUP" => Some(EntityPrefix::Sup),
        "HAZ" => Some(EntityPrefix::Haz),
        "ACT" => Some(EntityPrefix::Act),
        _ => None,
    }
}

fn entity_dir_name(prefix: EntityPrefix) -> &'static str {
    match prefix {
        EntityPrefix::Req => "requirements",
        EntityPrefix::Risk => "risks",
        EntityPrefix::Test => "verification/protocols",
        EntityPrefix::Rslt => "verification/results",
        EntityPrefix::Cmp => "bom/components",
        EntityPrefix::Asm => "bom/assemblies",
        EntityPrefix::Feat => "tolerances/features",
        EntityPrefix::Mate => "tolerances/mates",
        EntityPrefix::Tol => "tolerances/stackups",
        EntityPrefix::Proc => "manufacturing/processes",
        EntityPrefix::Ctrl => "manufacturing/controls",
        EntityPrefix::Work => "manufacturing/work_instructions",
        EntityPrefix::Lot => "manufacturing/lots",
        EntityPrefix::Dev => "manufacturing/deviations",
        EntityPrefix::Ncr => "manufacturing/ncrs",
        EntityPrefix::Capa => "manufacturing/capas",
        EntityPrefix::Quot => "bom/quotes",
        EntityPrefix::Sup => "bom/suppliers",
        EntityPrefix::Haz => "risks/hazards",
        EntityPrefix::Act => "actions",
    }
}

/// List entities of any type from the cache
#[tauri::command]
pub async fn list_entities(
    params: ListEntitiesParams,
    state: State<'_, AppState>,
) -> CommandResult<EntityListResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let prefix = prefix_from_string(&params.entity_type)
        .ok_or_else(|| CommandError::InvalidInput(format!("Unknown entity type: {}", params.entity_type)))?;

    let filter = EntityFilter {
        prefix: Some(prefix),
        status: params.status.as_ref().and_then(|v| {
            v.first().and_then(|s| s.parse().ok())
        }),
        search: params.search.clone(),
        ..Default::default()
    };

    let entities = cache.list_entities(&filter);

    // Apply tag filtering if specified
    let entities: Vec<_> = if let Some(ref tags) = params.tags {
        entities
            .into_iter()
            .filter(|e| {
                let entity_tags: Vec<String> = e.tags.iter().map(|t| t.to_lowercase()).collect();
                tags.iter().any(|t| entity_tags.contains(&t.to_lowercase()))
            })
            .collect()
    } else {
        entities
    };

    let total_count = entities.len();

    // Apply pagination
    let offset = params.offset.unwrap_or(0);
    let dir = project.root().join(entity_dir_name(prefix));

    let items: Vec<EntityData> = entities
        .into_iter()
        .skip(offset)
        .take(params.limit.unwrap_or(100))
        .map(|e| {
            // Load full entity data for entity-specific fields
            let data = load_entity_json(&dir, &e.id).unwrap_or(Value::Null);

            EntityData {
                id: e.id.clone(),
                prefix: format!("{:?}", prefix),
                title: e.title.clone(),
                status: format!("{:?}", e.status).to_lowercase(),
                author: e.author.clone(),
                created: e.created.to_rfc3339(),
                tags: e.tags.clone(),
                data,
            }
        })
        .collect();

    Ok(EntityListResult { items, total_count })
}

/// Get a single entity by ID with full data
#[tauri::command]
pub async fn get_entity(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<EntityData>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    // Get from cache first for metadata
    let cached = cache.get_entity(&id);

    if let Some(cached) = cached {
        // Parse the ID to get prefix
        let entity_id: EntityId = id.parse()
            .map_err(|_| CommandError::InvalidInput(format!("Invalid entity ID: {}", id)))?;

        let prefix = entity_id.prefix();
        let dir = project.root().join(entity_dir_name(prefix));

        // Load the full entity from file
        let data = load_entity_json(&dir, &id)?;

        Ok(Some(EntityData {
            id: cached.id.clone(),
            prefix: format!("{:?}", prefix),
            title: cached.title.clone(),
            status: format!("{:?}", cached.status).to_lowercase(),
            author: cached.author.clone(),
            created: cached.created.to_rfc3339(),
            tags: cached.tags.clone(),
            data,
        }))
    } else {
        Ok(None)
    }
}

/// Load entity from file and return as JSON Value
fn load_entity_json(dir: &PathBuf, id: &str) -> CommandResult<Value> {
    let file_path = loader::find_entity_file(dir, id)
        .ok_or_else(|| CommandError::NotFound(format!("Entity file not found: {}", id)))?;

    let content = std::fs::read_to_string(&file_path)?;
    let value: Value = serde_yml::from_str(&content)
        .map_err(|e| CommandError::Other(format!("Failed to parse entity: {}", e)))?;

    Ok(value)
}

/// Delete an entity by ID
#[tauri::command]
pub async fn delete_entity(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    // Parse the ID to get prefix
    let entity_id: EntityId = id.parse()
        .map_err(|_| CommandError::InvalidInput(format!("Invalid entity ID: {}", id)))?;

    let prefix = entity_id.prefix();
    let dir = project.root().join(entity_dir_name(prefix));

    // Find and delete the file
    let file_path = loader::find_entity_file(&dir, &id)
        .ok_or_else(|| CommandError::NotFound(format!("Entity not found: {}", id)))?;

    std::fs::remove_file(&file_path)?;

    // Sync cache to remove the deleted entity
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Create or update an entity from JSON data
#[tauri::command]
pub async fn save_entity(
    #[allow(non_snake_case)]
    entityType: String,
    data: Value,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let prefix = prefix_from_string(&entityType)
        .ok_or_else(|| CommandError::InvalidInput(format!("Unknown entity type: {}", entityType)))?;

    let dir = project.root().join(entity_dir_name(prefix));

    // Ensure directory exists
    std::fs::create_dir_all(&dir)?;

    // Get or generate ID
    let id = if let Some(id_val) = data.get("id") {
        id_val.as_str()
            .ok_or_else(|| CommandError::InvalidInput("Invalid ID field".to_string()))?
            .to_string()
    } else {
        EntityId::new(prefix).to_string()
    };

    // Add ID to data if not present
    let mut data = data;
    if let Value::Object(ref mut map) = data {
        map.insert("id".to_string(), Value::String(id.clone()));
    }

    // Convert to YAML and save
    let yaml = serde_yml::to_string(&data)
        .map_err(|e| CommandError::Other(format!("Failed to serialize entity: {}", e)))?;

    let file_path = dir.join(format!("{}.tdt.yaml", id));
    std::fs::write(&file_path, yaml)?;

    // Sync cache to pick up the new/updated entity
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(id)
}

/// Get entity count by type
#[tauri::command]
pub async fn get_entity_count(
    #[allow(non_snake_case)]
    entityType: String,
    state: State<'_, AppState>,
) -> CommandResult<usize> {
    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let prefix = prefix_from_string(&entityType)
        .ok_or_else(|| CommandError::InvalidInput(format!("Unknown entity type: {}", entityType)))?;

    let filter = EntityFilter {
        prefix: Some(prefix),
        ..Default::default()
    };

    let count = cache.list_entities(&filter).len();
    Ok(count)
}

/// Get all entity types with their counts
#[tauri::command]
pub async fn get_all_entity_counts(
    state: State<'_, AppState>,
) -> CommandResult<std::collections::HashMap<String, usize>> {
    let cache = state.cache.lock().unwrap();
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let mut counts = std::collections::HashMap::new();

    let prefixes = [
        ("requirements", EntityPrefix::Req),
        ("risks", EntityPrefix::Risk),
        ("tests", EntityPrefix::Test),
        ("results", EntityPrefix::Rslt),
        ("components", EntityPrefix::Cmp),
        ("assemblies", EntityPrefix::Asm),
        ("features", EntityPrefix::Feat),
        ("mates", EntityPrefix::Mate),
        ("stackups", EntityPrefix::Tol),
        ("processes", EntityPrefix::Proc),
        ("controls", EntityPrefix::Ctrl),
        ("work_instructions", EntityPrefix::Work),
        ("lots", EntityPrefix::Lot),
        ("deviations", EntityPrefix::Dev),
        ("ncrs", EntityPrefix::Ncr),
        ("capas", EntityPrefix::Capa),
        ("quotes", EntityPrefix::Quot),
        ("suppliers", EntityPrefix::Sup),
        ("hazards", EntityPrefix::Haz),
        ("actions", EntityPrefix::Act),
    ];

    for (name, prefix) in prefixes {
        let filter = EntityFilter {
            prefix: Some(prefix),
            ..Default::default()
        };
        counts.insert(name.to_string(), cache.list_entities(&filter).len());
    }

    Ok(counts)
}
