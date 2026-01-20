//! Project management commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tdt_core::core::{cache::EntityCache, config::Config, identity::EntityPrefix, project::Project};

/// Information about the currently open project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Path to the project root
    pub path: String,
    /// Project name (from config or directory name)
    pub name: String,
    /// Entity counts by type
    pub entity_counts: EntityCounts,
    /// Default author from config
    pub author: String,
}

/// Count of entities by type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityCounts {
    pub requirements: usize,
    pub risks: usize,
    pub tests: usize,
    pub results: usize,
    pub components: usize,
    pub assemblies: usize,
    pub features: usize,
    pub mates: usize,
    pub stackups: usize,
    pub processes: usize,
    pub controls: usize,
    pub work_instructions: usize,
    pub lots: usize,
    pub deviations: usize,
    pub ncrs: usize,
    pub capas: usize,
    pub quotes: usize,
    pub suppliers: usize,
}

impl EntityCounts {
    /// Get entity counts from the cache
    pub fn from_cache(cache: &EntityCache) -> Self {
        use tdt_core::core::cache::EntityFilter;

        let count_by_prefix = |prefix: EntityPrefix| -> usize {
            let filter = EntityFilter {
                prefix: Some(prefix),
                ..Default::default()
            };
            cache.list_entities(&filter).len()
        };

        Self {
            requirements: count_by_prefix(EntityPrefix::Req),
            risks: count_by_prefix(EntityPrefix::Risk),
            tests: count_by_prefix(EntityPrefix::Test),
            results: count_by_prefix(EntityPrefix::Rslt),
            components: count_by_prefix(EntityPrefix::Cmp),
            assemblies: count_by_prefix(EntityPrefix::Asm),
            features: count_by_prefix(EntityPrefix::Feat),
            mates: count_by_prefix(EntityPrefix::Mate),
            stackups: count_by_prefix(EntityPrefix::Tol),
            processes: count_by_prefix(EntityPrefix::Proc),
            controls: count_by_prefix(EntityPrefix::Ctrl),
            work_instructions: count_by_prefix(EntityPrefix::Work),
            lots: count_by_prefix(EntityPrefix::Lot),
            deviations: count_by_prefix(EntityPrefix::Dev),
            ncrs: count_by_prefix(EntityPrefix::Ncr),
            capas: count_by_prefix(EntityPrefix::Capa),
            quotes: count_by_prefix(EntityPrefix::Quot),
            suppliers: count_by_prefix(EntityPrefix::Sup),
        }
    }

    /// Get total count of all entities
    pub fn total(&self) -> usize {
        self.requirements
            + self.risks
            + self.tests
            + self.results
            + self.components
            + self.assemblies
            + self.features
            + self.mates
            + self.stackups
            + self.processes
            + self.controls
            + self.work_instructions
            + self.lots
            + self.deviations
            + self.ncrs
            + self.capas
            + self.quotes
            + self.suppliers
    }
}

/// Open an existing Tessera project
#[tauri::command]
pub async fn open_project(path: String, state: State<'_, AppState>) -> CommandResult<ProjectInfo> {
    let path = PathBuf::from(&path);

    // Discover the project
    let project = Project::discover_from(&path).map_err(|e| CommandError::ProjectOpen(e.to_string()))?;

    // Open the cache
    let cache = EntityCache::open(&project).map_err(|e| CommandError::ProjectOpen(e.to_string()))?;

    // Load config to get author
    let config = Config::load();

    // Build project info
    let info = ProjectInfo {
        path: project.root().display().to_string(),
        name: project
            .root()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Tessera Project".to_string()),
        entity_counts: EntityCounts::from_cache(&cache),
        author: config.author(),
    };

    // Store in state
    *state.project.lock().unwrap() = Some(project);
    *state.cache.lock().unwrap() = Some(cache);

    Ok(info)
}

/// Initialize a new Tessera project
#[tauri::command]
pub async fn init_project(path: String, state: State<'_, AppState>) -> CommandResult<ProjectInfo> {
    let path = PathBuf::from(&path);

    // Initialize the project
    let project = Project::init(&path).map_err(|e| CommandError::ProjectInit(e.to_string()))?;

    // Open the cache
    let cache = EntityCache::open(&project).map_err(|e| CommandError::ProjectInit(e.to_string()))?;

    // Load config to get author
    let config = Config::load();

    // Build project info
    let info = ProjectInfo {
        path: project.root().display().to_string(),
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Tessera Project".to_string()),
        entity_counts: EntityCounts::default(),
        author: config.author(),
    };

    // Store in state
    *state.project.lock().unwrap() = Some(project);
    *state.cache.lock().unwrap() = Some(cache);

    Ok(info)
}

/// Close the current project
#[tauri::command]
pub async fn close_project(state: State<'_, AppState>) -> CommandResult<()> {
    state.close_project();
    Ok(())
}

/// Get information about the current project
#[tauri::command]
pub async fn get_project_info(state: State<'_, AppState>) -> CommandResult<Option<ProjectInfo>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    match (&*project, &*cache) {
        (Some(project), Some(cache)) => {
            let config = Config::load();
            Ok(Some(ProjectInfo {
                path: project.root().display().to_string(),
                name: project
                    .root()
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Tessera Project".to_string()),
                entity_counts: EntityCounts::from_cache(cache),
                author: config.author(),
            }))
        }
        _ => Ok(None),
    }
}

/// Refresh the project cache
#[tauri::command]
pub async fn refresh_project(state: State<'_, AppState>) -> CommandResult<ProjectInfo> {
    let project_guard = state.project.lock().unwrap();
    let mut cache_guard = state.cache.lock().unwrap();

    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    // Rebuild the cache
    let new_cache =
        EntityCache::open(project).map_err(|e| CommandError::Service(e.to_string()))?;

    // Load config to get author
    let config = Config::load();

    let info = ProjectInfo {
        path: project.root().display().to_string(),
        name: project
            .root()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Tessera Project".to_string()),
        entity_counts: EntityCounts::from_cache(&new_cache),
        author: config.author(),
    };

    *cache_guard = Some(new_cache);

    Ok(info)
}
