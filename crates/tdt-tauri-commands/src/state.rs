//! Application state management for Tessera Desktop

use std::sync::Mutex;
use tdt_core::core::{cache::EntityCache, project::Project};

/// Global application state managed by Tauri
pub struct AppState {
    /// Currently open project (if any)
    pub project: Mutex<Option<Project>>,
    /// Entity cache for the current project
    pub cache: Mutex<Option<EntityCache>>,
}

impl AppState {
    /// Create a new empty application state
    pub fn new() -> Self {
        Self {
            project: Mutex::new(None),
            cache: Mutex::new(None),
        }
    }

    /// Check if a project is currently open
    pub fn has_project(&self) -> bool {
        self.project.lock().unwrap().is_some()
    }

    /// Close the current project
    pub fn close_project(&self) {
        *self.project.lock().unwrap() = None;
        *self.cache.lock().unwrap() = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
