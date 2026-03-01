//! General entity workflow commands (approve, submit, reject, release, status)
//!
//! These Tauri commands provide workflow operations for any entity type,
//! with unified authorization via [`WorkflowGuard`].

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tdt_core::services::{
    ApprovalResult, ApproveEntityInput, RejectEntityInput, ReleaseEntityInput, SubmitEntityInput,
    WorkflowGuard, WorkflowService,
};

/// Input for approving an entity via Tauri
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TauriApproveInput {
    /// Optional approval comment
    pub message: Option<String>,
    /// Sign the approval (GPG/SSH/gitsign)
    pub sign: Option<bool>,
    /// Force approval (skip auth check)
    pub force: Option<bool>,
}

/// Input for submitting an entity via Tauri
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TauriSubmitInput {
    /// Optional submission comment
    pub message: Option<String>,
}

/// Input for rejecting an entity via Tauri
#[derive(Debug, Clone, Deserialize)]
pub struct TauriRejectInput {
    /// Reason for rejection (required)
    pub reason: String,
}

/// Input for releasing an entity via Tauri
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TauriReleaseInput {
    /// Optional release comment
    pub message: Option<String>,
    /// Sign the release
    pub sign: Option<bool>,
    /// Force release (skip auth check)
    pub force: Option<bool>,
}

/// Approval status result for Tauri
#[derive(Debug, Clone, Serialize)]
pub struct TauriApprovalStatus {
    pub current_approvals: u32,
    pub required_approvals: u32,
    pub requirements_met: bool,
    pub missing_roles: Vec<String>,
    pub approvers: Vec<String>,
}

/// Approve an entity (any type) — requires entity to be in Review status
#[tauri::command]
pub async fn approve_entity(
    id: String,
    input: Option<TauriApproveInput>,
    state: State<'_, AppState>,
) -> CommandResult<ApprovalResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let result = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;

        let guard = WorkflowGuard::load(project);
        let service = WorkflowService::new(project, cache, guard);

        let input = input.unwrap_or_default();
        service.approve(
            &id,
            ApproveEntityInput {
                message: input.message,
                sign: input.sign.unwrap_or(false),
                force: input.force.unwrap_or(false),
            },
        )?
    };

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(result)
}

/// Submit an entity for review (Draft → Review)
#[tauri::command]
pub async fn submit_entity(
    id: String,
    input: Option<TauriSubmitInput>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;

        let guard = WorkflowGuard::load(project);
        let service = WorkflowService::new(project, cache, guard);

        let input = input.unwrap_or_default();
        service.submit(
            &id,
            SubmitEntityInput {
                message: input.message,
            },
        )?;
    }

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Reject an entity back to draft (Review → Draft)
#[tauri::command]
pub async fn reject_entity(
    id: String,
    input: TauriRejectInput,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;

        let guard = WorkflowGuard::load(project);
        let service = WorkflowService::new(project, cache, guard);

        service.reject(
            &id,
            RejectEntityInput {
                reason: input.reason,
            },
        )?;
    }

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Release an entity (Approved → Released)
#[tauri::command]
pub async fn release_entity(
    id: String,
    input: Option<TauriReleaseInput>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;

        let guard = WorkflowGuard::load(project);
        let service = WorkflowService::new(project, cache, guard);

        let input = input.unwrap_or_default();
        service.release(
            &id,
            ReleaseEntityInput {
                message: input.message,
                sign: input.sign.unwrap_or(false),
                force: input.force.unwrap_or(false),
            },
        )?;
    }

    // Sync cache
    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() {
        let _ = cache.sync();
    }

    Ok(())
}

/// Get the approval status for an entity
#[tauri::command]
pub async fn get_entity_approval_status(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<TauriApprovalStatus> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();

    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let guard = WorkflowGuard::load(project);
    let service = WorkflowService::new(project, cache, guard);

    let status = service.get_status(&id)?;

    Ok(TauriApprovalStatus {
        current_approvals: status.current_approvals,
        required_approvals: status.required_approvals,
        requirements_met: status.requirements_met,
        missing_roles: status.missing_roles.iter().map(|r| r.to_string()).collect(),
        approvers: status.approvers,
    })
}
