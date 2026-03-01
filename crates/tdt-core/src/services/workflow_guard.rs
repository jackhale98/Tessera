//! Workflow authorization guard
//!
//! Centralizes role-based authorization checks for approval and release operations.
//! Loads the team roster and workflow config from the project, then provides
//! uniform auth checks that any consumer (CLI, Tauri, service) can call.

use std::path::PathBuf;

use crate::core::entity::Status;
use crate::core::git::Git;
use crate::core::identity::EntityPrefix;
use crate::core::project::Project;
use crate::core::team::{Role, SigningFormat, TeamMember, TeamRoster};
use crate::core::workflow::{ApprovalRequirements, WorkflowConfig};
use crate::services::common::{ServiceError, ServiceResult};

/// An authorized user resolved from the team roster
#[derive(Debug, Clone)]
pub struct AuthorizedUser {
    pub name: String,
    pub email: String,
    pub roles: Vec<Role>,
    pub signing_format: Option<SigningFormat>,
}

impl AuthorizedUser {
    /// Get the first role that matches any of the required roles, or the first role
    pub fn primary_role(&self) -> Option<Role> {
        self.roles.first().copied()
    }

    /// Get the role string suitable for recording in approvals
    pub fn role_for_prefix(&self, roster: &TeamRoster, prefix: EntityPrefix) -> Option<Role> {
        if let Some(required) = roster.required_roles(prefix) {
            // Return the first role this user has that matches a required role
            self.roles.iter().find(|r| required.contains(r)).copied()
        } else {
            self.primary_role()
        }
    }
}

/// Result of a signature verification check
#[derive(Debug, Clone)]
pub struct SignatureCheck {
    /// Whether signature is required
    pub required: bool,
    /// The signing format to use (from the user's team config)
    pub format: Option<SigningFormat>,
    /// Whether a signing key is configured in git
    pub key_available: bool,
}

/// Workflow authorization guard
///
/// Loads team roster + workflow config and provides centralized authorization
/// checks for approval, release, and signature operations.
pub struct WorkflowGuard {
    config: WorkflowConfig,
    roster: Option<TeamRoster>,
    repo_root: Option<PathBuf>,
}

impl WorkflowGuard {
    /// Load workflow guard from a project
    pub fn load(project: &Project) -> Self {
        let config = Self::load_workflow_config(project);
        let roster = TeamRoster::load(project);
        let repo_root = if project.root().join(".git").exists() {
            Some(project.root().to_path_buf())
        } else {
            None
        };

        Self {
            config,
            roster,
            repo_root,
        }
    }

    /// Create a guard with explicit config and roster (for testing)
    pub fn new(config: WorkflowConfig, roster: Option<TeamRoster>) -> Self {
        Self {
            config,
            roster,
            repo_root: None,
        }
    }

    /// Set the repo root for git operations
    pub fn with_repo_root(mut self, root: PathBuf) -> Self {
        self.repo_root = Some(root);
        self
    }

    /// Check if workflow enforcement is enabled
    ///
    /// Returns true when both `workflow.enabled: true` in config AND a team roster exists.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.roster.is_some()
    }

    /// Get the workflow config
    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    /// Get the team roster
    pub fn roster(&self) -> Option<&TeamRoster> {
        self.roster.as_ref()
    }

    /// Get approval requirements for a given entity prefix
    pub fn approval_requirements(&self, prefix: EntityPrefix) -> &ApprovalRequirements {
        self.config.get_approval_requirements(prefix)
    }

    /// Resolve the current git user against the team roster
    pub fn current_user(&self) -> Option<AuthorizedUser> {
        let roster = self.roster.as_ref()?;
        let member = roster.current_user_in_repo(self.repo_root.as_deref())?;
        Some(member_to_authorized(member))
    }

    /// Check if the current user is authorized to approve entities of the given type.
    ///
    /// Returns:
    /// - `Ok(None)` when workflow is disabled — caller should proceed without auth
    /// - `Ok(Some(user))` when the user is authorized
    /// - `Err(ServiceError)` when the user is unauthorized or not in roster
    pub fn check_approval_auth(
        &self,
        prefix: EntityPrefix,
    ) -> ServiceResult<Option<AuthorizedUser>> {
        if !self.is_enabled() {
            return Ok(None);
        }

        let roster = self.roster.as_ref().unwrap(); // safe: is_enabled checks this
        let member = roster
            .current_user_in_repo(self.repo_root.as_deref())
            .ok_or_else(|| {
                ServiceError::Other(
                    "Current user not found in team roster. \
                     Ensure git user.name or user.email matches a roster entry."
                        .to_string(),
                )
            })?;

        if !roster.can_approve(member, prefix) {
            let required = roster
                .required_roles(prefix)
                .map(|roles| {
                    roles
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<_>>()
                        .join(" or ")
                })
                .unwrap_or_else(|| "team member".to_string());

            return Err(ServiceError::Other(format!(
                "Authorization required: {} approval requires role {}. \
                 Your roles: [{}]",
                prefix.as_str(),
                required,
                member
                    .roles
                    .iter()
                    .map(|r| r.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }

        Ok(Some(member_to_authorized(member)))
    }

    /// Check if the current user is authorized to release entities.
    ///
    /// Returns:
    /// - `Ok(None)` when workflow is disabled
    /// - `Ok(Some(user))` when authorized
    /// - `Err(ServiceError)` when unauthorized
    pub fn check_release_auth(&self) -> ServiceResult<Option<AuthorizedUser>> {
        if !self.is_enabled() {
            return Ok(None);
        }

        let roster = self.roster.as_ref().unwrap();
        let member = roster
            .current_user_in_repo(self.repo_root.as_deref())
            .ok_or_else(|| {
                ServiceError::Other(
                    "Current user not found in team roster. \
                     Ensure git user.name or user.email matches a roster entry."
                        .to_string(),
                )
            })?;

        if !roster.can_release(member) {
            return Err(ServiceError::Other(
                "Authorization required: release requires management role or admin.".to_string(),
            ));
        }

        Ok(Some(member_to_authorized(member)))
    }

    /// Check signature requirements for a given entity type.
    ///
    /// When `require_signature` is configured for the entity type and `sign` is false,
    /// returns an error. Also validates that the user's signing format is available.
    ///
    /// Returns a `SignatureCheck` with details about what's needed.
    pub fn check_signature_required(
        &self,
        prefix: EntityPrefix,
        sign: bool,
    ) -> ServiceResult<SignatureCheck> {
        let requirements = self.config.get_approval_requirements(prefix);

        if !requirements.require_signature {
            return Ok(SignatureCheck {
                required: false,
                format: None,
                key_available: true,
            });
        }

        // Signature is required
        if !sign {
            return Err(ServiceError::Other(format!(
                "Signature required for {} approvals. Use --sign / -S flag. \
                 (Configured via workflow.approvals.{}.require_signature)",
                prefix.as_str(),
                prefix
            )));
        }

        // Check that the user has a signing method available
        let signing_format = self.current_user().and_then(|u| u.signing_format);

        let key_available = self.check_signing_key_available(signing_format);

        if !key_available {
            let format_name = signing_format
                .map(|f| f.to_string())
                .unwrap_or_else(|| "gpg (default)".to_string());
            return Err(ServiceError::Other(format!(
                "Signing key not configured. Format: {}. \
                 Configure with: git config user.signingkey <KEY_ID>\n\
                 For SSH: git config gpg.format ssh && git config user.signingkey ~/.ssh/id_ed25519.pub\n\
                 For gitsign: git config gpg.format gitsign",
                format_name
            )));
        }

        Ok(SignatureCheck {
            required: true,
            format: signing_format,
            key_available: true,
        })
    }

    /// Verify a commit's signature after it's been created
    pub fn verify_commit_signature(&self, commit_sha: &str) -> ServiceResult<Option<String>> {
        let Some(ref root) = self.repo_root else {
            return Ok(None);
        };

        let git = Git::new(root);
        match git.verify_commit_signature(commit_sha) {
            Ok(signer) => Ok(signer),
            Err(e) => Err(ServiceError::Other(format!(
                "Signature verification failed: {}",
                e
            ))),
        }
    }

    /// Check if a signing key is available for the given format
    fn check_signing_key_available(&self, format: Option<SigningFormat>) -> bool {
        let Some(ref root) = self.repo_root else {
            return false;
        };

        let git = Git::new(root);
        match format.unwrap_or(SigningFormat::Gpg) {
            SigningFormat::Gpg => {
                // Check if user.signingkey is configured
                git.signing_configured()
            }
            SigningFormat::Ssh => {
                // SSH signing: check if user.signingkey is set (will be a path to .pub file)
                git.signing_configured()
            }
            SigningFormat::Gitsign => {
                // Gitsign: keyless, just need gitsign binary available
                std::process::Command::new("gitsign")
                    .arg("--version")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            }
        }
    }

    /// Load workflow config from project's .tdt/config.yaml
    fn load_workflow_config(project: &Project) -> WorkflowConfig {
        let config_path = project.tdt_dir().join("config.yaml");
        if !config_path.exists() {
            return WorkflowConfig::with_defaults();
        }

        let contents = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return WorkflowConfig::with_defaults(),
        };

        // Parse the full config and extract the workflow section
        let value: serde_yml::Value = match serde_yml::from_str(&contents) {
            Ok(v) => v,
            Err(_) => return WorkflowConfig::with_defaults(),
        };

        if let Some(workflow) = value.get("workflow") {
            serde_yml::from_value(workflow.clone())
                .unwrap_or_else(|_| WorkflowConfig::with_defaults())
        } else {
            WorkflowConfig::with_defaults()
        }
    }
}

/// Convert a TeamMember to an AuthorizedUser
fn member_to_authorized(member: &TeamMember) -> AuthorizedUser {
    AuthorizedUser {
        name: member.name.clone(),
        email: member.email.clone(),
        roles: member.roles.clone(),
        signing_format: member.signing_format,
    }
}

/// Validate that a status transition is allowed for a workflow operation
pub fn validate_status_for_operation(
    current: Status,
    operation: &str,
    expected: Status,
) -> ServiceResult<()> {
    if current != expected {
        return Err(ServiceError::ValidationFailed(format!(
            "Cannot {} entity in '{}' status (expected '{}')",
            operation, current, expected
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::team::TeamMember;

    fn make_roster() -> TeamRoster {
        let mut roster = TeamRoster::default();
        roster.members.push(TeamMember {
            name: "Alice Engineer".to_string(),
            email: "alice@example.com".to_string(),
            username: "alice".to_string(),
            roles: vec![Role::Engineering],
            active: true,
            signing_format: Some(SigningFormat::Gpg),
        });
        roster.members.push(TeamMember {
            name: "Bob Quality".to_string(),
            email: "bob@example.com".to_string(),
            username: "bob".to_string(),
            roles: vec![Role::Quality],
            active: true,
            signing_format: Some(SigningFormat::Ssh),
        });
        roster.members.push(TeamMember {
            name: "Charlie Manager".to_string(),
            email: "charlie@example.com".to_string(),
            username: "charlie".to_string(),
            roles: vec![Role::Management],
            active: true,
            signing_format: Some(SigningFormat::Gitsign),
        });
        roster.members.push(TeamMember {
            name: "Admin User".to_string(),
            email: "admin@example.com".to_string(),
            username: "admin".to_string(),
            roles: vec![Role::Admin],
            active: true,
            signing_format: None,
        });
        roster
            .approval_matrix
            .insert("REQ".to_string(), vec![Role::Engineering, Role::Quality]);
        roster
            .approval_matrix
            .insert("LOT".to_string(), vec![Role::Quality, Role::Management]);
        roster
            .approval_matrix
            .insert("DEV".to_string(), vec![Role::Quality, Role::Management]);
        roster
            .approval_matrix
            .insert("_release".to_string(), vec![Role::Management]);
        roster
    }

    fn make_config(enabled: bool) -> WorkflowConfig {
        WorkflowConfig {
            enabled,
            ..WorkflowConfig::with_defaults()
        }
    }

    fn make_config_with_signature(enabled: bool, prefix: &str) -> WorkflowConfig {
        let mut config = make_config(enabled);
        config.approvals.insert(
            prefix.to_string(),
            ApprovalRequirements {
                min_approvals: 1,
                required_roles: vec![],
                require_unique_approvers: true,
                require_signature: true,
            },
        );
        config
    }

    #[test]
    fn test_disabled_guard_permits_all() {
        let guard = WorkflowGuard::new(make_config(false), Some(make_roster()));
        assert!(!guard.is_enabled());
        assert!(matches!(
            guard.check_approval_auth(EntityPrefix::Req),
            Ok(None)
        ));
        assert!(matches!(guard.check_release_auth(), Ok(None)));
    }

    #[test]
    fn test_no_roster_disables_guard() {
        let guard = WorkflowGuard::new(make_config(true), None);
        assert!(!guard.is_enabled());
        assert!(matches!(
            guard.check_approval_auth(EntityPrefix::Req),
            Ok(None)
        ));
    }

    #[test]
    fn test_signature_not_required_passes() {
        let guard = WorkflowGuard::new(make_config(true), Some(make_roster()));
        let check = guard
            .check_signature_required(EntityPrefix::Req, false)
            .unwrap();
        assert!(!check.required);
    }

    #[test]
    fn test_signature_required_without_sign_flag_errors() {
        let guard =
            WorkflowGuard::new(make_config_with_signature(true, "REQ"), Some(make_roster()));
        let result = guard.check_signature_required(EntityPrefix::Req, false);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Signature required"));
    }

    #[test]
    fn test_validate_status_for_operation() {
        assert!(validate_status_for_operation(Status::Review, "approve", Status::Review).is_ok());
        assert!(validate_status_for_operation(Status::Draft, "approve", Status::Review).is_err());
    }

    #[test]
    fn test_authorized_user_role_for_prefix() {
        let roster = make_roster();
        let user = AuthorizedUser {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            roles: vec![Role::Engineering, Role::Quality],
            signing_format: Some(SigningFormat::Gpg),
        };

        // REQ requires Engineering or Quality — Alice has Engineering first
        let role = user.role_for_prefix(&roster, EntityPrefix::Req);
        assert!(role == Some(Role::Engineering) || role == Some(Role::Quality));

        // For a prefix with no matrix entry, returns primary role
        let role = user.role_for_prefix(&roster, EntityPrefix::Cmp);
        assert_eq!(role, Some(Role::Engineering));
    }

    #[test]
    fn test_approval_requirements_returns_config() {
        let mut config = make_config(true);
        config.approvals.insert(
            "RISK".to_string(),
            ApprovalRequirements {
                min_approvals: 3,
                required_roles: vec![],
                require_unique_approvers: true,
                require_signature: false,
            },
        );
        let guard = WorkflowGuard::new(config, Some(make_roster()));

        let reqs = guard.approval_requirements(EntityPrefix::Risk);
        assert_eq!(reqs.min_approvals, 3);
        assert!(reqs.require_unique_approvers);
    }

    #[test]
    fn test_approval_requirements_defaults_for_unconfigured_prefix() {
        let guard = WorkflowGuard::new(make_config(true), Some(make_roster()));

        // CMP is not configured, should get defaults
        let reqs = guard.approval_requirements(EntityPrefix::Cmp);
        assert_eq!(reqs.min_approvals, 1);
    }

    #[test]
    fn test_validate_status_all_operations() {
        // submit: Draft → Review
        assert!(validate_status_for_operation(Status::Draft, "submit", Status::Draft).is_ok());
        assert!(validate_status_for_operation(Status::Review, "submit", Status::Draft).is_err());

        // approve: Review → Approved
        assert!(validate_status_for_operation(Status::Review, "approve", Status::Review).is_ok());
        assert!(
            validate_status_for_operation(Status::Approved, "approve", Status::Review).is_err()
        );

        // reject: Review → Draft
        assert!(validate_status_for_operation(Status::Review, "reject", Status::Review).is_ok());
        assert!(validate_status_for_operation(Status::Draft, "reject", Status::Review).is_err());

        // release: Approved → Released
        assert!(
            validate_status_for_operation(Status::Approved, "release", Status::Approved).is_ok()
        );
        assert!(
            validate_status_for_operation(Status::Review, "release", Status::Approved).is_err()
        );
    }

    #[test]
    fn test_validate_status_error_message() {
        let err =
            validate_status_for_operation(Status::Draft, "approve", Status::Review).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("approve"));
        assert!(msg.contains("draft"));
        assert!(msg.contains("review"));
    }

    #[test]
    fn test_signature_required_with_sign_flag_but_no_key() {
        // When signature is required and --sign is passed, but no signing key is available,
        // the check should fail with a helpful message
        let guard =
            WorkflowGuard::new(make_config_with_signature(true, "REQ"), Some(make_roster()));
        // No repo_root set, so signing key check will fail
        let result = guard.check_signature_required(EntityPrefix::Req, true);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Signing key not configured"));
    }

    #[test]
    fn test_authorized_user_primary_role() {
        let user = AuthorizedUser {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            roles: vec![Role::Engineering, Role::Quality],
            signing_format: None,
        };
        assert_eq!(user.primary_role(), Some(Role::Engineering));

        let empty_user = AuthorizedUser {
            name: "Nobody".to_string(),
            email: "nobody@example.com".to_string(),
            roles: vec![],
            signing_format: None,
        };
        assert_eq!(empty_user.primary_role(), None);
    }

    #[test]
    fn test_member_to_authorized_conversion() {
        let member = TeamMember {
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
            username: "tester".to_string(),
            roles: vec![Role::Engineering, Role::Quality],
            active: true,
            signing_format: Some(SigningFormat::Ssh),
        };
        let authorized = member_to_authorized(&member);
        assert_eq!(authorized.name, "Test");
        assert_eq!(authorized.email, "test@example.com");
        assert_eq!(authorized.roles, vec![Role::Engineering, Role::Quality]);
        assert_eq!(authorized.signing_format, Some(SigningFormat::Ssh));
    }

    #[test]
    fn test_guard_config_accessor() {
        let config = make_config(true);
        let guard = WorkflowGuard::new(config, None);
        assert!(guard.config().enabled);
    }

    #[test]
    fn test_guard_roster_accessor() {
        let guard_with = WorkflowGuard::new(make_config(true), Some(make_roster()));
        assert!(guard_with.roster().is_some());
        assert_eq!(guard_with.roster().unwrap().members.len(), 4);

        let guard_without = WorkflowGuard::new(make_config(true), None);
        assert!(guard_without.roster().is_none());
    }

    #[test]
    fn test_with_repo_root() {
        let guard = WorkflowGuard::new(make_config(true), None)
            .with_repo_root(std::path::PathBuf::from("/tmp/test"));
        // Should not panic; just sets the repo root
        assert!(!guard.is_enabled()); // Still disabled because no roster
    }
}
