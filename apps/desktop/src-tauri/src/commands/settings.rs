//! Settings management commands for config and team roster

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use tdt_core::core::config::Config;
use tdt_core::core::team::{Role, SigningFormat, TeamMember, TeamRoster};
use tdt_core::core::workflow::WorkflowConfig;

// =============================================================================
// Data Transfer Objects
// =============================================================================

/// General settings (author, editor, pager, default_format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub author: Option<String>,
    pub editor: Option<String>,
    pub pager: Option<String>,
    pub default_format: Option<String>,
}

/// Workflow settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSettings {
    pub enabled: bool,
    pub provider: String,
    pub require_branch: bool,
    pub auto_commit: bool,
    pub auto_merge: bool,
    pub base_branch: String,
    pub branch_pattern: String,
    pub submit_message: String,
    pub approve_message: String,
}

impl Default for WorkflowSettings {
    fn default() -> Self {
        let defaults = WorkflowConfig::with_defaults();
        Self {
            enabled: defaults.enabled,
            provider: defaults.provider.to_string(),
            require_branch: defaults.require_branch,
            auto_commit: defaults.auto_commit,
            auto_merge: defaults.auto_merge,
            base_branch: defaults.base_branch,
            branch_pattern: defaults.branch_pattern,
            submit_message: defaults.submit_message,
            approve_message: defaults.approve_message,
        }
    }
}

/// Manufacturing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingSettings {
    pub lot_branch_enabled: bool,
    pub base_branch: Option<String>,
    pub branch_pattern: Option<String>,
    pub create_tags: bool,
    pub sign_commits: bool,
}

impl Default for ManufacturingSettings {
    fn default() -> Self {
        Self {
            lot_branch_enabled: false,
            base_branch: None,
            branch_pattern: None,
            create_tags: true,
            sign_commits: false,
        }
    }
}

/// Complete settings bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllSettings {
    pub general: GeneralSettings,
    pub workflow: WorkflowSettings,
    pub manufacturing: ManufacturingSettings,
    pub config_paths: ConfigPaths,
}

/// Configuration file paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPaths {
    pub global_config: Option<String>,
    pub project_config: Option<String>,
}

/// Team member DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberDto {
    pub name: String,
    pub email: String,
    pub username: String,
    pub roles: Vec<String>,
    pub active: bool,
    pub signing_format: Option<String>,
}

impl From<&TeamMember> for TeamMemberDto {
    fn from(m: &TeamMember) -> Self {
        Self {
            name: m.name.clone(),
            email: m.email.clone(),
            username: m.username.clone(),
            roles: m.roles.iter().map(|r| r.to_string()).collect(),
            active: m.active,
            signing_format: m.signing_format.map(|s| s.to_string()),
        }
    }
}

impl TryFrom<TeamMemberDto> for TeamMember {
    type Error = String;

    fn try_from(dto: TeamMemberDto) -> Result<Self, Self::Error> {
        let roles: Result<Vec<Role>, _> = dto.roles.iter().map(|r| r.parse()).collect();
        let signing_format: Option<SigningFormat> = dto
            .signing_format
            .map(|s| s.parse())
            .transpose()
            .map_err(|e: String| e)?;

        Ok(TeamMember {
            name: dto.name,
            email: dto.email,
            username: dto.username,
            roles: roles.map_err(|e| e)?,
            active: dto.active,
            signing_format,
        })
    }
}

/// Team roster DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRosterDto {
    pub version: u32,
    pub members: Vec<TeamMemberDto>,
    pub approval_matrix: HashMap<String, Vec<String>>,
}

impl From<&TeamRoster> for TeamRosterDto {
    fn from(roster: &TeamRoster) -> Self {
        Self {
            version: roster.version,
            members: roster.members.iter().map(TeamMemberDto::from).collect(),
            approval_matrix: roster
                .approval_matrix
                .iter()
                .map(|(k, v)| (k.clone(), v.iter().map(|r| r.to_string()).collect()))
                .collect(),
        }
    }
}

impl TryFrom<TeamRosterDto> for TeamRoster {
    type Error = String;

    fn try_from(dto: TeamRosterDto) -> Result<Self, Self::Error> {
        let members: Result<Vec<TeamMember>, _> =
            dto.members.into_iter().map(TeamMember::try_from).collect();

        let approval_matrix: Result<HashMap<String, Vec<Role>>, _> = dto
            .approval_matrix
            .into_iter()
            .map(|(k, v)| {
                let roles: Result<Vec<Role>, _> = v.iter().map(|r| r.parse()).collect();
                roles.map(|r| (k, r))
            })
            .collect();

        Ok(TeamRoster {
            version: dto.version,
            members: members?,
            approval_matrix: approval_matrix.map_err(|e| e)?,
        })
    }
}

// =============================================================================
// Config Commands
// =============================================================================

/// Get all settings (general, workflow, manufacturing)
#[tauri::command]
pub async fn get_all_settings(state: State<'_, AppState>) -> CommandResult<AllSettings> {
    let config = Config::load();

    let project_guard = state.project.lock().unwrap();
    let project_config_path = project_guard
        .as_ref()
        .map(|p| p.tdt_dir().join("config.yaml").display().to_string());

    let general = GeneralSettings {
        author: config.author.clone(),
        editor: config.editor.clone(),
        pager: config.pager.clone(),
        default_format: config.default_format.clone(),
    };

    let workflow = WorkflowSettings {
        enabled: config.workflow.enabled,
        provider: config.workflow.provider.to_string(),
        require_branch: config.workflow.require_branch,
        auto_commit: config.workflow.auto_commit,
        auto_merge: config.workflow.auto_merge,
        base_branch: config.workflow.base_branch.clone(),
        branch_pattern: config.workflow.branch_pattern.clone(),
        submit_message: config.workflow.submit_message.clone(),
        approve_message: config.workflow.approve_message.clone(),
    };

    let manufacturing = config
        .manufacturing
        .as_ref()
        .map(|m| ManufacturingSettings {
            lot_branch_enabled: m.lot_branch_enabled,
            base_branch: m.base_branch.clone(),
            branch_pattern: m.branch_pattern.clone(),
            create_tags: m.create_tags,
            sign_commits: m.sign_commits,
        })
        .unwrap_or_default();

    Ok(AllSettings {
        general,
        workflow,
        manufacturing,
        config_paths: ConfigPaths {
            global_config: Config::global_config_path().map(|p| p.display().to_string()),
            project_config: project_config_path,
        },
    })
}

/// Get general settings only
#[tauri::command]
pub async fn get_general_settings() -> CommandResult<GeneralSettings> {
    let config = Config::load();
    Ok(GeneralSettings {
        author: config.author,
        editor: config.editor,
        pager: config.pager,
        default_format: config.default_format,
    })
}

/// Get workflow settings only
#[tauri::command]
pub async fn get_workflow_settings() -> CommandResult<WorkflowSettings> {
    let config = Config::load();
    Ok(WorkflowSettings {
        enabled: config.workflow.enabled,
        provider: config.workflow.provider.to_string(),
        require_branch: config.workflow.require_branch,
        auto_commit: config.workflow.auto_commit,
        auto_merge: config.workflow.auto_merge,
        base_branch: config.workflow.base_branch,
        branch_pattern: config.workflow.branch_pattern,
        submit_message: config.workflow.submit_message,
        approve_message: config.workflow.approve_message,
    })
}

/// Get manufacturing settings only
#[tauri::command]
pub async fn get_manufacturing_settings() -> CommandResult<ManufacturingSettings> {
    let config = Config::load();
    Ok(config
        .manufacturing
        .map(|m| ManufacturingSettings {
            lot_branch_enabled: m.lot_branch_enabled,
            base_branch: m.base_branch,
            branch_pattern: m.branch_pattern,
            create_tags: m.create_tags,
            sign_commits: m.sign_commits,
        })
        .unwrap_or_default())
}

/// Save general settings to project config (or global if no project)
#[tauri::command]
pub async fn save_general_settings(
    settings: GeneralSettings,
    save_to_global: bool,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let config_path = if save_to_global {
        Config::global_config_path().ok_or_else(|| {
            CommandError::Other("Could not determine global config path".to_string())
        })?
    } else {
        let project_guard = state.project.lock().unwrap();
        let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
        project.tdt_dir().join("config.yaml")
    };

    // Load existing config or create new
    let mut config_value = load_or_create_config(&config_path)?;

    // Update general settings
    if let Some(map) = config_value.as_mapping_mut() {
        update_optional_field(map, "author", &settings.author);
        update_optional_field(map, "editor", &settings.editor);
        update_optional_field(map, "pager", &settings.pager);
        update_optional_field(map, "default_format", &settings.default_format);
    }

    save_config(&config_path, &config_value)?;
    Ok(())
}

/// Save workflow settings to project config
#[tauri::command]
pub async fn save_workflow_settings(
    settings: WorkflowSettings,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let config_path = project.tdt_dir().join("config.yaml");

    let mut config_value = load_or_create_config(&config_path)?;

    if let Some(map) = config_value.as_mapping_mut() {
        let workflow_key = serde_yml::Value::String("workflow".to_string());

        let workflow = map
            .entry(workflow_key)
            .or_insert_with(|| serde_yml::Value::Mapping(serde_yml::Mapping::new()));

        if let Some(wf_map) = workflow.as_mapping_mut() {
            wf_map.insert(
                serde_yml::Value::String("enabled".to_string()),
                serde_yml::Value::Bool(settings.enabled),
            );
            wf_map.insert(
                serde_yml::Value::String("provider".to_string()),
                serde_yml::Value::String(settings.provider),
            );
            wf_map.insert(
                serde_yml::Value::String("require_branch".to_string()),
                serde_yml::Value::Bool(settings.require_branch),
            );
            wf_map.insert(
                serde_yml::Value::String("auto_commit".to_string()),
                serde_yml::Value::Bool(settings.auto_commit),
            );
            wf_map.insert(
                serde_yml::Value::String("auto_merge".to_string()),
                serde_yml::Value::Bool(settings.auto_merge),
            );
            wf_map.insert(
                serde_yml::Value::String("base_branch".to_string()),
                serde_yml::Value::String(settings.base_branch),
            );
            wf_map.insert(
                serde_yml::Value::String("branch_pattern".to_string()),
                serde_yml::Value::String(settings.branch_pattern),
            );
            wf_map.insert(
                serde_yml::Value::String("submit_message".to_string()),
                serde_yml::Value::String(settings.submit_message),
            );
            wf_map.insert(
                serde_yml::Value::String("approve_message".to_string()),
                serde_yml::Value::String(settings.approve_message),
            );
        }
    }

    save_config(&config_path, &config_value)?;
    Ok(())
}

/// Save manufacturing settings to project config
#[tauri::command]
pub async fn save_manufacturing_settings(
    settings: ManufacturingSettings,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;
    let config_path = project.tdt_dir().join("config.yaml");

    let mut config_value = load_or_create_config(&config_path)?;

    if let Some(map) = config_value.as_mapping_mut() {
        let mfg_key = serde_yml::Value::String("manufacturing".to_string());

        let manufacturing = map
            .entry(mfg_key)
            .or_insert_with(|| serde_yml::Value::Mapping(serde_yml::Mapping::new()));

        if let Some(mfg_map) = manufacturing.as_mapping_mut() {
            mfg_map.insert(
                serde_yml::Value::String("lot_branch_enabled".to_string()),
                serde_yml::Value::Bool(settings.lot_branch_enabled),
            );
            update_optional_field(mfg_map, "base_branch", &settings.base_branch);
            update_optional_field(mfg_map, "branch_pattern", &settings.branch_pattern);
            mfg_map.insert(
                serde_yml::Value::String("create_tags".to_string()),
                serde_yml::Value::Bool(settings.create_tags),
            );
            mfg_map.insert(
                serde_yml::Value::String("sign_commits".to_string()),
                serde_yml::Value::Bool(settings.sign_commits),
            );
        }
    }

    save_config(&config_path, &config_value)?;
    Ok(())
}

// =============================================================================
// Team Roster Commands
// =============================================================================

/// Get team roster
#[tauri::command]
pub async fn get_team_roster(state: State<'_, AppState>) -> CommandResult<Option<TeamRosterDto>> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let roster = TeamRoster::load(project);
    Ok(roster.as_ref().map(TeamRosterDto::from))
}

/// Save team roster
#[tauri::command]
pub async fn save_team_roster(
    roster: TeamRosterDto,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let team_roster = TeamRoster::try_from(roster)
        .map_err(|e| CommandError::InvalidInput(format!("Invalid team roster: {}", e)))?;

    team_roster.save(project)?;
    Ok(())
}

/// Initialize team roster with default template
#[tauri::command]
pub async fn init_team_roster(state: State<'_, AppState>) -> CommandResult<TeamRosterDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let roster = TeamRoster::default();
    roster.save(project)?;

    Ok(TeamRosterDto::from(&roster))
}

/// Add a team member
#[tauri::command]
pub async fn add_team_member(
    member: TeamMemberDto,
    state: State<'_, AppState>,
) -> CommandResult<TeamRosterDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mut roster = TeamRoster::load(project).unwrap_or_default();

    // Check for duplicate username
    if roster.find_member(&member.username).is_some() {
        return Err(CommandError::InvalidInput(format!(
            "Team member with username '{}' already exists",
            member.username
        )));
    }

    let team_member = TeamMember::try_from(member)
        .map_err(|e| CommandError::InvalidInput(format!("Invalid team member: {}", e)))?;

    roster.add_member(team_member);
    roster.save(project)?;

    Ok(TeamRosterDto::from(&roster))
}

/// Update a team member
#[tauri::command]
pub async fn update_team_member(
    username: String,
    member: TeamMemberDto,
    state: State<'_, AppState>,
) -> CommandResult<TeamRosterDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mut roster = TeamRoster::load(project).ok_or_else(|| {
        CommandError::NotFound("Team roster not found. Initialize it first.".to_string())
    })?;

    // Find and update the member
    let member_idx = roster
        .members
        .iter()
        .position(|m| m.username.eq_ignore_ascii_case(&username))
        .ok_or_else(|| CommandError::NotFound(format!("Team member '{}' not found", username)))?;

    let team_member = TeamMember::try_from(member)
        .map_err(|e| CommandError::InvalidInput(format!("Invalid team member: {}", e)))?;

    roster.members[member_idx] = team_member;
    roster.save(project)?;

    Ok(TeamRosterDto::from(&roster))
}

/// Remove a team member
#[tauri::command]
pub async fn remove_team_member(
    username: String,
    state: State<'_, AppState>,
) -> CommandResult<TeamRosterDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mut roster = TeamRoster::load(project).ok_or_else(|| {
        CommandError::NotFound("Team roster not found. Initialize it first.".to_string())
    })?;

    if !roster.remove_member(&username) {
        return Err(CommandError::NotFound(format!(
            "Team member '{}' not found",
            username
        )));
    }

    roster.save(project)?;

    Ok(TeamRosterDto::from(&roster))
}

/// Set team member active status
#[tauri::command]
pub async fn set_team_member_active(
    username: String,
    active: bool,
    state: State<'_, AppState>,
) -> CommandResult<TeamRosterDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mut roster = TeamRoster::load(project).ok_or_else(|| {
        CommandError::NotFound("Team roster not found. Initialize it first.".to_string())
    })?;

    let member = roster
        .members
        .iter_mut()
        .find(|m| m.username.eq_ignore_ascii_case(&username))
        .ok_or_else(|| CommandError::NotFound(format!("Team member '{}' not found", username)))?;

    member.active = active;
    roster.save(project)?;

    Ok(TeamRosterDto::from(&roster))
}

/// Update approval matrix
#[tauri::command]
pub async fn update_approval_matrix(
    entity_prefix: String,
    roles: Vec<String>,
    state: State<'_, AppState>,
) -> CommandResult<TeamRosterDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mut roster = TeamRoster::load(project).ok_or_else(|| {
        CommandError::NotFound("Team roster not found. Initialize it first.".to_string())
    })?;

    let parsed_roles: Result<Vec<Role>, _> = roles.iter().map(|r| r.parse()).collect();
    let parsed_roles = parsed_roles
        .map_err(|e: String| CommandError::InvalidInput(format!("Invalid role: {}", e)))?;

    if parsed_roles.is_empty() {
        roster.approval_matrix.remove(&entity_prefix);
    } else {
        roster.approval_matrix.insert(entity_prefix, parsed_roles);
    }

    roster.save(project)?;

    Ok(TeamRosterDto::from(&roster))
}

/// Remove an entity from approval matrix
#[tauri::command]
pub async fn remove_approval_matrix_entry(
    entity_prefix: String,
    state: State<'_, AppState>,
) -> CommandResult<TeamRosterDto> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let mut roster = TeamRoster::load(project).ok_or_else(|| {
        CommandError::NotFound("Team roster not found. Initialize it first.".to_string())
    })?;

    roster.approval_matrix.remove(&entity_prefix);
    roster.save(project)?;

    Ok(TeamRosterDto::from(&roster))
}

/// Get list of available roles
#[tauri::command]
pub async fn get_available_roles() -> CommandResult<Vec<String>> {
    Ok(vec![
        "engineering".to_string(),
        "quality".to_string(),
        "management".to_string(),
        "admin".to_string(),
    ])
}

/// Get list of available signing formats
#[tauri::command]
pub async fn get_available_signing_formats() -> CommandResult<Vec<String>> {
    Ok(vec![
        "gpg".to_string(),
        "ssh".to_string(),
        "gitsign".to_string(),
    ])
}

/// Get entity prefixes for approval matrix
#[tauri::command]
pub async fn get_entity_prefixes_for_approval() -> CommandResult<Vec<EntityPrefixInfo>> {
    Ok(vec![
        EntityPrefixInfo {
            prefix: "REQ".to_string(),
            name: "Requirement".to_string(),
        },
        EntityPrefixInfo {
            prefix: "RISK".to_string(),
            name: "Risk".to_string(),
        },
        EntityPrefixInfo {
            prefix: "TEST".to_string(),
            name: "Test".to_string(),
        },
        EntityPrefixInfo {
            prefix: "RSLT".to_string(),
            name: "Result".to_string(),
        },
        EntityPrefixInfo {
            prefix: "CMP".to_string(),
            name: "Component".to_string(),
        },
        EntityPrefixInfo {
            prefix: "ASM".to_string(),
            name: "Assembly".to_string(),
        },
        EntityPrefixInfo {
            prefix: "DEV".to_string(),
            name: "Deviation".to_string(),
        },
        EntityPrefixInfo {
            prefix: "NCR".to_string(),
            name: "NCR".to_string(),
        },
        EntityPrefixInfo {
            prefix: "CAPA".to_string(),
            name: "CAPA".to_string(),
        },
        EntityPrefixInfo {
            prefix: "LOT".to_string(),
            name: "Lot".to_string(),
        },
        EntityPrefixInfo {
            prefix: "_release".to_string(),
            name: "Release Authorization".to_string(),
        },
    ])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPrefixInfo {
    pub prefix: String,
    pub name: String,
}

/// Get current git user info (for prefilling new team member)
#[tauri::command]
pub async fn get_current_git_user() -> CommandResult<GitUserInfo> {
    let name = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !s.is_empty() {
                    Some(s)
                } else {
                    None
                }
            } else {
                None
            }
        });

    let email = std::process::Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !s.is_empty() {
                    Some(s)
                } else {
                    None
                }
            } else {
                None
            }
        });

    Ok(GitUserInfo { name, email })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitUserInfo {
    pub name: Option<String>,
    pub email: Option<String>,
}

// =============================================================================
// Helper Functions
// =============================================================================

fn load_or_create_config(path: &PathBuf) -> CommandResult<serde_yml::Value> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        serde_yml::from_str(&contents)
            .map_err(|e| CommandError::Other(format!("Failed to parse config: {}", e)))
    } else {
        Ok(serde_yml::Value::Mapping(serde_yml::Mapping::new()))
    }
}

fn save_config(path: &PathBuf, value: &serde_yml::Value) -> CommandResult<()> {
    let contents = serde_yml::to_string(value)
        .map_err(|e| CommandError::Other(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(path, contents)?;
    Ok(())
}

fn update_optional_field(map: &mut serde_yml::Mapping, key: &str, value: &Option<String>) {
    let yaml_key = serde_yml::Value::String(key.to_string());
    if let Some(v) = value {
        map.insert(yaml_key, serde_yml::Value::String(v.clone()));
    } else {
        map.remove(&yaml_key);
    }
}
