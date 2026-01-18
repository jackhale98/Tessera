//! Version control and git history commands

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tauri::State;
use tdt_core::core::Git;
use tdt_core::core::workflow::ApprovalRecord;

// ============================================================================
// Types
// ============================================================================

/// Git repository status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusInfo {
    /// Current branch name
    pub current_branch: String,
    /// Whether the working directory is clean
    pub is_clean: bool,
    /// Whether on main/master branch
    pub is_main_branch: bool,
    /// List of uncommitted files
    pub uncommitted_files: Vec<UncommittedFile>,
    /// Whether the repo is a git repository
    pub is_repo: bool,
}

/// Information about an uncommitted file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncommittedFile {
    /// File path relative to repo root
    pub path: String,
    /// Status: "modified", "added", "deleted", "untracked", "renamed"
    pub status: String,
    /// Entity ID if this is an entity file
    pub entity_id: Option<String>,
    /// Entity title if available
    pub entity_title: Option<String>,
}

/// Git user information for version control operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcGitUserInfo {
    /// Git user.name
    pub name: Option<String>,
    /// Git user.email
    pub email: Option<String>,
    /// GPG signing key ID
    pub signing_key: Option<String>,
    /// Whether GPG signing is configured
    pub signing_configured: bool,
}

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitInfo {
    /// Full commit hash
    pub hash: String,
    /// Short commit hash (7 chars)
    pub short_hash: String,
    /// Commit message (first line)
    pub message: String,
    /// Author name
    pub author: String,
    /// Author email
    pub author_email: Option<String>,
    /// Commit date
    pub date: String,
    /// Whether the commit is GPG signed
    pub is_signed: bool,
}

/// Workflow history for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHistory {
    /// Entity ID
    pub entity_id: String,
    /// Entity title
    pub title: String,
    /// Current status
    pub current_status: String,
    /// Revision number
    pub revision: Option<u32>,
    /// Workflow events (created, approved, released, etc.)
    pub events: Vec<WorkflowEvent>,
    /// Git tags associated with this entity
    pub tags: Vec<String>,
}

/// A workflow event in an entity's history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Event type: "created", "approved", "released", "rejected"
    pub event_type: String,
    /// Actor who performed the action
    pub actor: String,
    /// When the event occurred
    pub timestamp: String,
    /// Role of the actor (for approvals)
    pub role: Option<String>,
    /// Comment or message
    pub comment: Option<String>,
    /// Whether signature was verified (for 21 CFR Part 11)
    pub signature_verified: Option<bool>,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Whether this is the current branch
    pub is_current: bool,
    /// Whether this is a remote branch
    pub is_remote: bool,
    /// Last commit hash on this branch
    pub last_commit: Option<String>,
    /// Last commit message
    pub last_message: Option<String>,
}

/// Tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// Tag name
    pub name: String,
    /// Tag message (for annotated tags)
    pub message: Option<String>,
    /// Tagger name
    pub tagger: Option<String>,
    /// Tag date
    pub date: Option<String>,
    /// Commit the tag points to
    pub commit: Option<String>,
}

/// Result of a commit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResult {
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Number of files committed
    pub files_changed: usize,
}

/// Result of a push operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    /// Branch that was pushed
    pub branch: String,
    /// Whether upstream was set
    pub upstream_set: bool,
}

// ============================================================================
// Git Status Commands
// ============================================================================

/// Get the current git repository status
#[tauri::command]
pub async fn get_git_status(state: State<'_, AppState>) -> CommandResult<GitStatusInfo> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard.as_ref();

    let git = Git::new(project.root());

    if !git.is_repo() {
        return Ok(GitStatusInfo {
            current_branch: String::new(),
            is_clean: true,
            is_main_branch: false,
            uncommitted_files: Vec::new(),
            is_repo: false,
        });
    }

    let current_branch = git.current_branch().unwrap_or_else(|_| "HEAD".to_string());
    let is_clean = git.is_clean();
    let is_main_branch = git.is_main_branch();

    // Get uncommitted files
    let uncommitted_files = match git.uncommitted_files() {
        Ok(files) => files
            .into_iter()
            .map(|line| {
                // Parse git status --porcelain format: "XY filename"
                let status = if line.starts_with("M ") || line.starts_with(" M") {
                    "modified"
                } else if line.starts_with("A ") || line.starts_with("AM") {
                    "added"
                } else if line.starts_with("D ") || line.starts_with(" D") {
                    "deleted"
                } else if line.starts_with("R") {
                    "renamed"
                } else if line.starts_with("??") {
                    "untracked"
                } else {
                    "modified"
                };

                // Extract the file path (skip the status prefix)
                let path = line.get(3..).unwrap_or(&line).trim().to_string();

                // Try to extract entity info from .tdt.yaml files
                let (entity_id, entity_title) = if path.ends_with(".tdt.yaml") {
                    extract_entity_info_from_path(project.root(), &path, cache)
                } else {
                    (None, None)
                };

                UncommittedFile {
                    path,
                    status: status.to_string(),
                    entity_id,
                    entity_title,
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    Ok(GitStatusInfo {
        current_branch,
        is_clean,
        is_main_branch,
        uncommitted_files,
        is_repo: true,
    })
}

/// Get git user information including signing configuration for version control
#[tauri::command]
pub async fn get_vc_git_user(state: State<'_, AppState>) -> CommandResult<VcGitUserInfo> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    let name = git.user_name().ok();
    let email = git.user_email().ok();
    let signing_key = git.signing_key();
    let signing_configured = git.signing_configured();

    Ok(VcGitUserInfo {
        name,
        email,
        signing_key,
        signing_configured,
    })
}

// ============================================================================
// Entity History Commands
// ============================================================================

/// Get git commit history for a specific entity
#[tauri::command]
pub async fn get_entity_history(
    id: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<GitCommitInfo>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard.as_ref();

    // Find the entity file
    let entity_file = find_entity_file(project.root(), &id, cache)?;

    // Build git log command
    let limit_arg = limit.unwrap_or(50);
    let output = Command::new("git")
        .args([
            "log",
            "--format=%H|%h|%s|%an|%ae|%aI|%G?",
            "--follow",
            &format!("-{}", limit_arg),
            "--",
            &entity_file.to_string_lossy(),
        ])
        .current_dir(project.root())
        .output()
        .map_err(|e| CommandError::Other(format!("Failed to run git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not have any commits yet") {
            return Ok(Vec::new());
        }
        return Err(CommandError::Other(format!("Git error: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<GitCommitInfo> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 7 {
                Some(GitCommitInfo {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    message: parts[2].to_string(),
                    author: parts[3].to_string(),
                    author_email: Some(parts[4].to_string()),
                    date: parts[5].to_string(),
                    is_signed: parts[6] == "G" || parts[6] == "U",
                })
            } else {
                None
            }
        })
        .collect();

    Ok(commits)
}

/// Get workflow history (approvals, releases) for an entity
#[tauri::command]
pub async fn get_entity_workflow_history(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<WorkflowHistory> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard.as_ref();

    // Find and read the entity file
    let entity_file = find_entity_file(project.root(), &id, cache)?;
    let content = std::fs::read_to_string(&entity_file)
        .map_err(|e| CommandError::Io(format!("Cannot read entity file: {}", e)))?;

    // Parse the entity YAML
    let entity: EntityData = serde_yml::from_str(&content)
        .map_err(|e| CommandError::Other(format!("Cannot parse entity: {}", e)))?;

    // Build workflow events
    let mut events = Vec::new();

    // Created event
    events.push(WorkflowEvent {
        event_type: "created".to_string(),
        actor: entity.author.clone().unwrap_or_else(|| "Unknown".to_string()),
        timestamp: entity.created.to_rfc3339(),
        role: None,
        comment: None,
        signature_verified: None,
    });

    // Approval events
    for approval in &entity.approvals {
        events.push(WorkflowEvent {
            event_type: "approved".to_string(),
            actor: approval.approver.clone(),
            timestamp: approval.timestamp.to_rfc3339(),
            role: approval.role.clone(),
            comment: approval.comment.clone(),
            signature_verified: approval.signature_verified,
        });
    }

    // Release event
    if let (Some(releaser), Some(released_at)) = (&entity.released_by, entity.released_at) {
        events.push(WorkflowEvent {
            event_type: "released".to_string(),
            actor: releaser.clone(),
            timestamp: released_at.to_rfc3339(),
            role: None,
            comment: None,
            signature_verified: None,
        });
    }

    // Rejection events
    for rejection in &entity.rejections {
        events.push(WorkflowEvent {
            event_type: "rejected".to_string(),
            actor: rejection.rejector.clone(),
            timestamp: rejection.timestamp.to_rfc3339(),
            role: None,
            comment: Some(rejection.reason.clone()),
            signature_verified: None,
        });
    }

    // Sort by timestamp
    events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    // Get git tags for this entity
    let git = Git::new(project.root());
    let short_id = truncate_id(&id);
    let mut tags = Vec::new();

    if let Ok(approval_tags) = git.list_tags(Some(&format!("approve/{}/*", short_id))) {
        tags.extend(approval_tags);
    }
    if let Ok(release_tags) = git.list_tags(Some(&format!("release/{}/*", short_id))) {
        tags.extend(release_tags);
    }

    Ok(WorkflowHistory {
        entity_id: id,
        title: entity.title,
        current_status: entity.status,
        revision: entity.revision,
        events,
        tags,
    })
}

/// Get diff for a specific commit affecting an entity
#[tauri::command]
pub async fn get_entity_file_diff(
    id: String,
    commit_hash: String,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard.as_ref();

    // Find the entity file
    let entity_file = find_entity_file(project.root(), &id, cache)?;

    // Get the diff for that commit
    let output = Command::new("git")
        .args([
            "show",
            "--format=",
            &commit_hash,
            "--",
            &entity_file.to_string_lossy(),
        ])
        .current_dir(project.root())
        .output()
        .map_err(|e| CommandError::Other(format!("Failed to run git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CommandError::Other(format!("Git error: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ============================================================================
// Branch Commands
// ============================================================================

/// List all branches (local and remote)
#[tauri::command]
pub async fn list_git_branches(state: State<'_, AppState>) -> CommandResult<Vec<BranchInfo>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());
    let current_branch = git.current_branch().unwrap_or_default();

    // Get local branches
    let output = Command::new("git")
        .args(["branch", "-v", "--format=%(refname:short)|%(objectname:short)|%(subject)"])
        .current_dir(project.root())
        .output()
        .map_err(|e| CommandError::Other(format!("Failed to run git: {}", e)))?;

    let mut branches: Vec<BranchInfo> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            let name = parts.first().unwrap_or(&"").to_string();
            BranchInfo {
                is_current: name == current_branch,
                name,
                is_remote: false,
                last_commit: parts.get(1).map(|s| s.to_string()),
                last_message: parts.get(2).map(|s| s.to_string()),
            }
        })
        .collect();

    // Get remote branches
    let output = Command::new("git")
        .args(["branch", "-r", "-v", "--format=%(refname:short)|%(objectname:short)|%(subject)"])
        .current_dir(project.root())
        .output();

    if let Ok(output) = output {
        let remote_branches: Vec<BranchInfo> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.is_empty() && !line.contains("HEAD"))
            .map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                BranchInfo {
                    name: parts.first().unwrap_or(&"").to_string(),
                    is_current: false,
                    is_remote: true,
                    last_commit: parts.get(1).map(|s| s.to_string()),
                    last_message: parts.get(2).map(|s| s.to_string()),
                }
            })
            .collect();
        branches.extend(remote_branches);
    }

    Ok(branches)
}

/// List git tags with optional pattern filter
#[tauri::command]
pub async fn list_git_tags(
    pattern: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<TagInfo>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    let tag_names = git.list_tags(pattern.as_deref())
        .map_err(|e| CommandError::Other(format!("Failed to list tags: {}", e)))?;

    // Get detailed info for each tag
    let mut tags = Vec::new();
    for name in tag_names {
        // Get tag info using git show
        let output = Command::new("git")
            .args(["tag", "-l", "--format=%(taggername)|%(taggerdate:iso)|%(contents:subject)|%(objectname:short)", &name])
            .current_dir(project.root())
            .output();

        let (tagger, date, message, commit) = if let Ok(output) = output {
            let line = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = line.trim().split('|').collect();
            (
                parts.first().filter(|s| !s.is_empty()).map(|s| s.to_string()),
                parts.get(1).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                parts.get(2).filter(|s| !s.is_empty()).map(|s| s.to_string()),
                parts.get(3).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            )
        } else {
            (None, None, None, None)
        };

        tags.push(TagInfo {
            name,
            message,
            tagger,
            date,
            commit,
        });
    }

    Ok(tags)
}

/// Checkout a branch
#[tauri::command]
pub async fn checkout_git_branch(
    branch: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    // Check for uncommitted changes
    if !git.is_clean() {
        return Err(CommandError::Other(
            "Cannot checkout: you have uncommitted changes. Commit or stash them first.".to_string()
        ));
    }

    // Try to checkout the branch
    git.fetch_and_checkout_branch(&branch)
        .map_err(|e| CommandError::Other(format!("Failed to checkout branch: {}", e)))?;

    Ok(())
}

/// Create a new branch
#[tauri::command]
pub async fn create_git_branch(
    name: String,
    checkout: bool,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    if checkout {
        git.create_and_checkout_branch(&name)
            .map_err(|e| CommandError::Other(format!("Failed to create branch: {}", e)))?;
    } else {
        git.create_branch(&name)
            .map_err(|e| CommandError::Other(format!("Failed to create branch: {}", e)))?;
    }

    Ok(())
}

// ============================================================================
// Commit Commands
// ============================================================================

/// Stage files for commit
#[tauri::command]
pub async fn stage_files(
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    let path_refs: Vec<&Path> = paths.iter().map(|p| Path::new(p)).collect();
    git.stage_files(&path_refs)
        .map_err(|e| CommandError::Other(format!("Failed to stage files: {}", e)))?;

    Ok(())
}

/// Stage a specific entity file
#[tauri::command]
pub async fn stage_entity(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let cache_guard = state.cache.lock().unwrap();
    let cache = cache_guard.as_ref();

    let git = Git::new(project.root());

    // Find the entity file
    let entity_file = find_entity_file(project.root(), &id, cache)?;

    git.stage_file(&entity_file)
        .map_err(|e| CommandError::Other(format!("Failed to stage entity: {}", e)))?;

    Ok(())
}

/// Commit staged changes
#[tauri::command]
pub async fn commit_changes(
    message: String,
    sign: bool,
    state: State<'_, AppState>,
) -> CommandResult<CommitResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    // Get the number of staged files before commit
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(project.root())
        .output()
        .map_err(|e| CommandError::Other(format!("Failed to get staged files: {}", e)))?;

    let files_changed = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .count();

    if files_changed == 0 {
        return Err(CommandError::Other("No staged changes to commit".to_string()));
    }

    // Commit
    let hash = if sign {
        git.commit_signed(&message)
    } else {
        git.commit(&message)
    }
    .map_err(|e| CommandError::Other(format!("Failed to commit: {}", e)))?;

    Ok(CommitResult {
        hash,
        message,
        files_changed,
    })
}

/// Push changes to remote
#[tauri::command]
pub async fn push_changes(
    branch: Option<String>,
    set_upstream: bool,
    state: State<'_, AppState>,
) -> CommandResult<PushResult> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    let branch_name = branch.unwrap_or_else(|| git.current_branch().unwrap_or_else(|_| "HEAD".to_string()));

    git.push(&branch_name, set_upstream)
        .map_err(|e| CommandError::Other(format!("Failed to push: {}", e)))?;

    Ok(PushResult {
        branch: branch_name,
        upstream_set: set_upstream,
    })
}

/// Pull changes from remote
#[tauri::command]
pub async fn pull_changes(state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    git.pull()
        .map_err(|e| CommandError::Other(format!("Failed to pull: {}", e)))?;

    // Refresh the cache after pull
    drop(project_guard);
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let mut cache_guard = state.cache.lock().unwrap();

    let new_cache = tdt_core::core::cache::EntityCache::open(project)
        .map_err(|e| CommandError::Service(e.to_string()))?;
    *cache_guard = Some(new_cache);

    Ok(())
}

/// Fetch from remote
#[tauri::command]
pub async fn fetch_changes(state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let git = Git::new(project.root());

    git.fetch()
        .map_err(|e| CommandError::Other(format!("Failed to fetch: {}", e)))?;

    Ok(())
}

/// Get recent commits from the repository
#[tauri::command]
pub async fn get_recent_commits(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<GitCommitInfo>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let limit_arg = limit.unwrap_or(50);

    let output = Command::new("git")
        .args([
            "log",
            "--format=%H|%h|%s|%an|%ae|%aI|%G?",
            &format!("-{}", limit_arg),
        ])
        .current_dir(project.root())
        .output()
        .map_err(|e| CommandError::Other(format!("Failed to run git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not have any commits yet") {
            return Ok(Vec::new());
        }
        return Err(CommandError::Other(format!("Git error: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<GitCommitInfo> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 7 {
                Some(GitCommitInfo {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    message: parts[2].to_string(),
                    author: parts[3].to_string(),
                    author_email: Some(parts[4].to_string()),
                    date: parts[5].to_string(),
                    is_signed: parts[6] == "G" || parts[6] == "U",
                })
            } else {
                None
            }
        })
        .collect();

    Ok(commits)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Entity data for parsing workflow history
#[derive(Debug, Deserialize)]
struct EntityData {
    #[allow(dead_code)]
    id: String,
    title: String,
    status: String,
    created: DateTime<Utc>,
    author: Option<String>,
    #[serde(default)]
    approvals: Vec<ApprovalRecord>,
    #[serde(default)]
    rejections: Vec<RejectionRecord>,
    released_by: Option<String>,
    released_at: Option<DateTime<Utc>>,
    revision: Option<u32>,
    #[serde(flatten)]
    _extra: HashMap<String, serde_yml::Value>,
}

/// Rejection record
#[derive(Debug, Deserialize)]
struct RejectionRecord {
    rejector: String,
    reason: String,
    timestamp: DateTime<Utc>,
}

/// Find an entity file by ID
fn find_entity_file(
    root: &Path,
    id: &str,
    cache: Option<&tdt_core::core::cache::EntityCache>,
) -> CommandResult<std::path::PathBuf> {
    // Try cache-based lookup first
    if let Some(cache) = cache {
        // Try resolving short ID
        let full_id = if id.contains('@') {
            cache.resolve_short_id(id)
        } else {
            None
        };

        let lookup_id = full_id.as_deref().unwrap_or(id);

        if let Some(entity) = cache.get_entity(lookup_id) {
            return Ok(entity.file_path);
        }
    }

    // Fallback: filesystem search based on prefix
    let search_dirs: Vec<(&str, &str)> = vec![
        ("REQ-", "requirements"),
        ("RISK-", "risks"),
        ("TEST-", "verification"),
        ("RSLT-", "verification"),
        ("CMP-", "bom/components"),
        ("ASM-", "bom/assemblies"),
        ("SUP-", "procurement/suppliers"),
        ("QUOTE-", "procurement/quotes"),
        ("PROC-", "manufacturing/processes"),
        ("CTRL-", "manufacturing/controls"),
        ("WORK-", "manufacturing/work_instructions"),
        ("LOT-", "manufacturing/lots"),
        ("DEV-", "manufacturing/deviations"),
        ("NCR-", "manufacturing/ncrs"),
        ("CAPA-", "manufacturing/capas"),
        ("FEAT-", "tolerances/features"),
        ("MATE-", "tolerances/mates"),
        ("TOL-", "tolerances/stackups"),
        ("HAZ-", "safety/hazards"),
    ];

    for (prefix, base_dir) in &search_dirs {
        if id.starts_with(prefix) {
            let dir = root.join(base_dir);
            if dir.exists() {
                if let Some(found) = search_dir_for_entity(&dir, id) {
                    return Ok(found);
                }
            }
        }
    }

    Err(CommandError::NotFound(format!("Entity file not found: {}", id)))
}

/// Recursively search a directory for an entity file by ID
fn search_dir_for_entity(dir: &Path, id: &str) -> Option<std::path::PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            // Recurse into subdirectory
            if let Some(found) = search_dir_for_entity(&path, id) {
                return Some(found);
            }
        } else if path.is_file() && path.to_string_lossy().ends_with(".tdt.yaml") {
            // Check if this file contains the entity ID
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains(&format!("id: {}", id))
                    || content.contains(&format!("id: \"{}\"", id))
                {
                    return Some(path);
                }
            }
        }
    }

    None
}

/// Extract entity info from a file path
fn extract_entity_info_from_path(
    root: &Path,
    relative_path: &str,
    _cache: Option<&tdt_core::core::cache::EntityCache>,
) -> (Option<String>, Option<String>) {
    let full_path = root.join(relative_path);

    // Try to read the file and extract ID
    if let Ok(content) = std::fs::read_to_string(&full_path) {
        // Quick parse for id and title
        let mut entity_id = None;
        let mut entity_title = None;

        for line in content.lines().take(20) {
            if line.starts_with("id:") {
                entity_id = line.get(3..).map(|s| s.trim().trim_matches('"').to_string());
            }
            if line.starts_with("title:") {
                entity_title = line.get(6..).map(|s| s.trim().trim_matches('"').to_string());
                break; // Title usually comes after id
            }
        }

        return (entity_id, entity_title);
    }

    (None, None)
}

/// Truncate entity ID to short form (e.g., REQ-abc123)
fn truncate_id(id: &str) -> String {
    // Find the first '-' and take up to 8 chars after it
    if let Some(dash_pos) = id.find('-') {
        let prefix = &id[..dash_pos];
        let suffix = &id[dash_pos + 1..];
        let short_suffix = if suffix.len() > 8 {
            &suffix[..8]
        } else {
            suffix
        };
        format!("{}-{}", prefix, short_suffix.to_lowercase())
    } else {
        id.to_string()
    }
}
