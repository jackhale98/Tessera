//! Error types for Tauri commands

use serde::Serialize;
use thiserror::Error;

/// Errors that can occur in Tauri commands
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("No project is currently open")]
    NoProject,

    #[error("Project already open at: {0}")]
    ProjectAlreadyOpen(String),

    #[error("Failed to open project: {0}")]
    ProjectOpen(String),

    #[error("Failed to initialize project: {0}")]
    ProjectInit(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError::Io(err.to_string())
    }
}

impl From<tdt_core::services::ServiceError> for CommandError {
    fn from(err: tdt_core::services::ServiceError) -> Self {
        CommandError::Service(err.to_string())
    }
}

/// Result type for Tauri commands
pub type CommandResult<T> = Result<T, CommandError>;
