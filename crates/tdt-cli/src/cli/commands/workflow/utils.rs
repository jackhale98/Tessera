//! Shared utilities for workflow commands
//!
//! Common helpers used by approve, reject, submit, and release commands.

use miette::{bail, IntoDiagnostic, Result};
use std::collections::HashSet;
use std::io::{self, BufRead};
use std::path::PathBuf;

use tdt_core::core::entity::Status;
use tdt_core::core::identity::EntityPrefix;
use tdt_core::core::workflow::{get_entity_info, get_prefix_from_id};
use tdt_core::core::{Config, Project, Provider, ProviderClient};

/// Collect entity IDs from CLI args, stdin, or PR
pub fn collect_entity_ids_from_args(
    ids: &[String],
    pr: Option<u64>,
    project: &Project,
    config: &Config,
    verbose: bool,
) -> Result<Vec<String>> {
    // Check for stdin
    if ids.len() == 1 && ids[0] == "-" {
        let stdin = io::stdin();
        return Ok(stdin
            .lock()
            .lines()
            .map_while(Result::ok)
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect());
    }

    // If --pr is set, fetch PR and extract entity IDs from branch name
    if let Some(pr_number) = pr {
        return extract_entities_from_pr(pr_number, project, config, verbose);
    }

    // Otherwise use provided IDs
    Ok(ids.to_vec())
}

/// Extract entity IDs from a PR's branch name or changed files
pub fn extract_entities_from_pr(
    pr_number: u64,
    project: &Project,
    config: &Config,
    verbose: bool,
) -> Result<Vec<String>> {
    if config.workflow.provider == Provider::None {
        bail!(
            "Cannot use --pr without a git provider configured.\n\
             Set workflow.provider to 'github' or 'gitlab' in .tdt/config.yaml"
        );
    }

    let provider = ProviderClient::new(config.workflow.provider, project.root());

    // Get PR info to access the branch name
    let pr_info = provider
        .get_pr(pr_number)
        .map_err(|e| miette::miette!("Failed to get PR #{}: {}", pr_number, e))?;

    if verbose {
        eprintln!(
            "  PR #{}: {} (branch: {})",
            pr_info.number, pr_info.title, pr_info.branch
        );
    }

    // Try to extract entity ID from branch name
    let branch = &pr_info.branch;

    if let Some(entity_id) = parse_entity_from_branch(branch) {
        if verbose {
            eprintln!("  Extracted entity from branch: {}", entity_id);
        }
        return Ok(vec![entity_id]);
    }

    // For batch branches or unknown patterns, fetch changed files from PR
    if verbose {
        eprintln!("  Branch doesn't contain entity ID, fetching changed files...");
    }

    extract_entities_from_pr_files(pr_number, project, config, verbose)
}

/// Parse entity ID from branch name like "review/REQ-01KCWY20"
///
/// Uses EntityPrefix::from_str for validation instead of a hardcoded list,
/// ensuring all entity types (including HAZ, ACT, etc.) are supported.
pub fn parse_entity_from_branch(branch: &str) -> Option<String> {
    // Strip "review/" prefix if present
    let branch = branch.strip_prefix("review/").unwrap_or(branch);

    // Skip batch branches
    if branch.starts_with("batch-") {
        return None;
    }

    // Try to parse as PREFIX-SHORTID format
    let parts: Vec<&str> = branch.splitn(2, '-').collect();
    if parts.len() == 2 {
        let prefix = parts[0].to_uppercase();
        // Use EntityPrefix::from_str instead of a hardcoded list
        if prefix.parse::<EntityPrefix>().is_ok() {
            // Return as PREFIX@shortid format for resolution
            return Some(format!("{}@{}", prefix, parts[1]));
        }
    }

    None
}

/// Extract entity IDs from files changed in a PR
pub fn extract_entities_from_pr_files(
    pr_number: u64,
    project: &Project,
    config: &Config,
    verbose: bool,
) -> Result<Vec<String>> {
    // Get list of changed files using gh/glab CLI
    let files = match config.workflow.provider {
        Provider::GitHub => {
            let output = std::process::Command::new("gh")
                .args(["pr", "diff", &pr_number.to_string(), "--name-only"])
                .current_dir(project.root())
                .output()
                .into_diagnostic()?;

            if !output.status.success() {
                bail!(
                    "Failed to get PR files: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            String::from_utf8_lossy(&output.stdout).to_string()
        }
        Provider::GitLab => {
            let output = std::process::Command::new("glab")
                .args(["mr", "diff", &pr_number.to_string(), "--name-only"])
                .current_dir(project.root())
                .output()
                .into_diagnostic()?;

            if !output.status.success() {
                bail!(
                    "Failed to get MR files: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            String::from_utf8_lossy(&output.stdout).to_string()
        }
        Provider::None => bail!("No provider configured"),
    };

    // Extract entity IDs from .tdt.yaml filenames
    let mut entity_ids: HashSet<String> = HashSet::new();

    for line in files.lines() {
        let filename = line.trim();
        // Match files like "requirements/REQ-01ABC123.tdt.yaml"
        if filename.ends_with(".tdt.yaml") {
            if let Some(basename) = std::path::Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
            {
                // Remove .tdt suffix to get entity ID
                if let Some(entity_id) = basename.strip_suffix(".tdt") {
                    entity_ids.insert(entity_id.to_string());
                }
            }
        }
    }

    if entity_ids.is_empty() {
        bail!(
            "No entity files found in PR #{}.\n\
             The PR may not contain any .tdt.yaml files.",
            pr_number
        );
    }

    if verbose {
        eprintln!("  Found {} entities in PR files", entity_ids.len());
    }

    Ok(entity_ids.into_iter().collect())
}

/// Find an entity file by ID in the project directory
pub fn find_entity_file(project: &Project, id: &str) -> Result<PathBuf> {
    use walkdir::WalkDir;

    let file_name = format!("{}.tdt.yaml", id);

    for entry in WalkDir::new(project.root())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name().to_string_lossy() == file_name {
            return Ok(entry.path().to_path_buf());
        }
    }

    bail!("Entity file not found: {}", id)
}

/// Scan project for entities matching a status and optional type filter
pub fn scan_entities_by_status(
    project: &Project,
    target_status: Status,
    entity_type: Option<&str>,
) -> Result<Vec<String>> {
    use walkdir::WalkDir;

    let target_prefix: Option<EntityPrefix> =
        entity_type.and_then(|t| t.to_uppercase().parse().ok());

    let mut ids = Vec::new();

    for entry in WalkDir::new(project.root())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "yaml")
                .unwrap_or(false)
        })
        .filter(|e| e.path().to_string_lossy().contains(".tdt.yaml"))
    {
        if let Ok((id, _, status)) = get_entity_info(entry.path()) {
            if status != target_status {
                continue;
            }

            if let Some(ref prefix_filter) = target_prefix {
                if let Some(prefix) = get_prefix_from_id(&id) {
                    if prefix != *prefix_filter {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            ids.push(id);
        }
    }

    Ok(ids)
}
