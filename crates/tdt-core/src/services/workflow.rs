//! General-purpose workflow service for entity status transitions
//!
//! Wraps `core::workflow` functions with authorization via [`WorkflowGuard`].
//! Provides approve, submit, reject, release, and status operations for any entity type.

use std::path::PathBuf;

use serde::Serialize;

use crate::core::cache::EntityCache;
use crate::core::entity::Status;
use crate::core::identity::EntityPrefix;
use crate::core::project::Project;
use crate::core::workflow::{self, ApprovalOptions, ApprovalStatus};
use crate::services::common::{ServiceError, ServiceResult};
use crate::services::workflow_guard::{validate_status_for_operation, WorkflowGuard};

/// Input for approving an entity
#[derive(Debug, Clone, Default)]
pub struct ApproveEntityInput {
    /// Optional approval comment
    pub message: Option<String>,
    /// Sign the approval (GPG/SSH/gitsign)
    pub sign: bool,
    /// Force approval (skip auth check — admin override)
    pub force: bool,
}

/// Input for submitting an entity for review
#[derive(Debug, Clone, Default)]
pub struct SubmitEntityInput {
    /// Optional submission comment
    pub message: Option<String>,
}

/// Input for rejecting an entity
#[derive(Debug, Clone)]
pub struct RejectEntityInput {
    /// Reason for rejection (required)
    pub reason: String,
}

/// Input for releasing an entity
#[derive(Debug, Clone, Default)]
pub struct ReleaseEntityInput {
    /// Optional release comment
    pub message: Option<String>,
    /// Sign the release
    pub sign: bool,
    /// Force release (skip auth check — admin override)
    pub force: bool,
}

/// Result of an approval operation
#[derive(Debug, Clone, Serialize)]
pub struct ApprovalResult {
    /// Entity ID that was approved
    pub id: String,
    /// Entity title
    pub title: String,
    /// New status after approval
    pub new_status: String,
    /// Current approval count
    pub current_approvals: u32,
    /// Required approval count
    pub required_approvals: u32,
    /// Whether all requirements are now met
    pub requirements_met: bool,
    /// Roles still needed
    pub missing_roles: Vec<String>,
    /// Name of the approver
    pub approver: String,
    /// Whether the approval was signed
    pub signed: bool,
}

/// General-purpose workflow service
///
/// Wraps existing `core::workflow` operations with authorization checks
/// via `WorkflowGuard`. Works for any entity type.
pub struct WorkflowService<'a> {
    project: &'a Project,
    cache: &'a EntityCache,
    guard: WorkflowGuard,
}

impl<'a> WorkflowService<'a> {
    /// Create a new WorkflowService
    pub fn new(project: &'a Project, cache: &'a EntityCache, guard: WorkflowGuard) -> Self {
        Self {
            project,
            cache,
            guard,
        }
    }

    /// Get a reference to the guard
    pub fn guard(&self) -> &WorkflowGuard {
        &self.guard
    }

    /// Approve an entity
    ///
    /// 1. Resolves entity ID to file path
    /// 2. Checks authorization (if workflow enabled)
    /// 3. Checks signature requirements
    /// 4. Checks for duplicate approval
    /// 5. Records the approval
    /// 6. Returns approval result with status
    pub fn approve(
        &self,
        id: &str,
        input: ApproveEntityInput,
    ) -> ServiceResult<ApprovalResult> {
        let (file_path, entity_id, title, status) = self.resolve_entity(id)?;

        // Validate status
        validate_status_for_operation(status, "approve", Status::Review)?;

        // Parse prefix from ID
        let prefix = self.parse_prefix(&entity_id)?;
        let requirements = self.guard.approval_requirements(prefix);

        // Authorization check (unless force)
        let authorized_user = if input.force {
            self.guard.current_user()
        } else {
            self.guard.check_approval_auth(prefix)?
        };

        // Signature check
        let sig_check = self
            .guard
            .check_signature_required(prefix, input.sign)?;

        // Determine approver identity
        let (approver_name, approver_email, approver_role) =
            if let Some(ref user) = authorized_user {
                let role = user.role_for_prefix(
                    self.guard.roster().unwrap_or(&Default::default()),
                    prefix,
                );
                (
                    user.name.clone(),
                    Some(user.email.clone()),
                    role,
                )
            } else {
                // Workflow disabled — use git identity
                let name = self.git_user_name().unwrap_or_else(|| "unknown".to_string());
                let email = self.git_user_email();
                (name, email, None)
            };

        // Check for duplicate approval
        if workflow::would_be_duplicate_approval(&file_path, &approver_name)
            .map_err(|e| ServiceError::Other(e.to_string()))?
        {
            return Err(ServiceError::Other(format!(
                "{} has already approved this entity",
                approver_name
            )));
        }

        // Build approval options
        let options = ApprovalOptions {
            approver: approver_name.clone(),
            email: approver_email,
            role: approver_role,
            comment: input.message,
            signature_verified: if sig_check.required && input.sign {
                Some(true)
            } else {
                None
            },
            signing_key: if input.sign {
                self.git_signing_key()
            } else {
                None
            },
        };

        // Record the approval
        workflow::record_approval_ext(&file_path, &options, Some(requirements))
            .map_err(|e| ServiceError::Other(e.to_string()))?;

        // Get updated status
        let approval_status = workflow::get_approval_status(&file_path, requirements)
            .map_err(|e| ServiceError::Other(e.to_string()))?;

        // Re-read status to see if it auto-transitioned
        let (_, _, new_status) = workflow::get_entity_info(&file_path)
            .map_err(|e| ServiceError::Other(e.to_string()))?;

        Ok(ApprovalResult {
            id: entity_id,
            title,
            new_status: new_status.to_string(),
            current_approvals: approval_status.current_approvals,
            required_approvals: approval_status.required_approvals,
            requirements_met: approval_status.requirements_met,
            missing_roles: approval_status
                .missing_roles
                .iter()
                .map(|r| r.to_string())
                .collect(),
            approver: approver_name,
            signed: sig_check.required && input.sign,
        })
    }

    /// Submit an entity for review (Draft → Review)
    pub fn submit(&self, id: &str, _input: SubmitEntityInput) -> ServiceResult<()> {
        let (file_path, _entity_id, _title, status) = self.resolve_entity(id)?;

        validate_status_for_operation(status, "submit", Status::Draft)?;

        workflow::update_entity_status(&file_path, Status::Review)
            .map_err(|e| ServiceError::Other(e.to_string()))
    }

    /// Reject an entity (Review → Draft)
    pub fn reject(&self, id: &str, input: RejectEntityInput) -> ServiceResult<()> {
        let (file_path, _entity_id, _title, status) = self.resolve_entity(id)?;

        validate_status_for_operation(status, "reject", Status::Review)?;

        // Determine rejector identity
        let rejector = self
            .guard
            .current_user()
            .map(|u| u.name)
            .or_else(|| self.git_user_name())
            .unwrap_or_else(|| "unknown".to_string());

        workflow::record_rejection(&file_path, &rejector, &input.reason)
            .map_err(|e| ServiceError::Other(e.to_string()))
    }

    /// Release an entity (Approved → Released)
    pub fn release(&self, id: &str, input: ReleaseEntityInput) -> ServiceResult<()> {
        let (file_path, _entity_id, _title, status) = self.resolve_entity(id)?;

        validate_status_for_operation(status, "release", Status::Approved)?;

        let prefix = self.parse_prefix(&_entity_id)?;

        // Authorization check (unless force)
        if !input.force {
            self.guard.check_release_auth()?;
        }

        // Signature check if configured
        if input.sign {
            self.guard.check_signature_required(prefix, input.sign)?;
        }

        // Determine releaser identity
        let releaser = self
            .guard
            .current_user()
            .map(|u| u.name)
            .or_else(|| self.git_user_name())
            .unwrap_or_else(|| "unknown".to_string());

        workflow::record_release(&file_path, &releaser)
            .map_err(|e| ServiceError::Other(e.to_string()))
    }

    /// Get the approval status for an entity
    pub fn get_status(&self, id: &str) -> ServiceResult<ApprovalStatus> {
        let (file_path, entity_id, _title, _status) = self.resolve_entity(id)?;

        let prefix = self.parse_prefix(&entity_id)?;
        let requirements = self.guard.approval_requirements(prefix);

        workflow::get_approval_status(&file_path, requirements)
            .map_err(|e| ServiceError::Other(e.to_string()))
    }

    // ========================================================================
    // Private helpers
    // ========================================================================

    /// Resolve an entity ID to (file_path, id, title, status)
    fn resolve_entity(&self, id: &str) -> ServiceResult<(PathBuf, String, String, Status)> {
        // Try cache first
        if let Some(cached) = self.cache.get_entity(id) {
            let path = self.project.root().join(&cached.file_path);
            if path.exists() {
                let (entity_id, title, status) = workflow::get_entity_info(&path)
                    .map_err(|e| ServiceError::Other(e.to_string()))?;
                return Ok((path, entity_id, title, status));
            }
        }

        // Fallback: search filesystem
        let path = self.find_entity_file(id)?;
        let (entity_id, title, status) = workflow::get_entity_info(&path)
            .map_err(|e| ServiceError::Other(e.to_string()))?;
        Ok((path, entity_id, title, status))
    }

    /// Find an entity file by ID by searching known directories
    fn find_entity_file(&self, id: &str) -> ServiceResult<PathBuf> {
        let filename = format!("{}.tdt.yaml", id);
        let root = self.project.root();

        // Known entity directories
        let dirs = [
            "requirements",
            "risks",
            "risks/hazards",
            "verification/protocols",
            "verification/results",
            "bom/components",
            "bom/assemblies",
            "bom/quotes",
            "bom/suppliers",
            "tolerances/features",
            "tolerances/mates",
            "tolerances/stackups",
            "manufacturing/processes",
            "manufacturing/controls",
            "manufacturing/work_instructions",
            "manufacturing/lots",
            "manufacturing/deviations",
            "manufacturing/ncrs",
            "manufacturing/capas",
            "actions",
        ];

        for dir in &dirs {
            let path = root.join(dir).join(&filename);
            if path.exists() {
                return Ok(path);
            }
        }

        Err(ServiceError::NotFound(format!(
            "Entity not found: {}",
            id
        )))
    }

    /// Parse an entity prefix from a full ID string
    fn parse_prefix(&self, id: &str) -> ServiceResult<EntityPrefix> {
        workflow::get_prefix_from_id(id).ok_or_else(|| {
            ServiceError::InvalidInput(format!("Cannot determine entity type from ID: {}", id))
        })
    }

    /// Run a git config query in the project root
    fn git_config(&self, key: &str) -> Option<String> {
        let mut cmd = std::process::Command::new("git");
        cmd.args(["config", key]);
        cmd.current_dir(self.project.root());
        cmd.output().ok().and_then(|o| {
            if o.status.success() {
                let val = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if val.is_empty() { None } else { Some(val) }
            } else {
                None
            }
        })
    }

    /// Get git user.name
    fn git_user_name(&self) -> Option<String> {
        self.git_config("user.name")
    }

    /// Get git user.email
    fn git_user_email(&self) -> Option<String> {
        self.git_config("user.email")
    }

    /// Get git user.signingkey
    fn git_signing_key(&self) -> Option<String> {
        self.git_config("user.signingkey")
    }
}
