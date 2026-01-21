//! `tdt cache` command - Manage the entity cache
//!
//! The cache is a local SQLite database that stores:
//! - Short ID mappings (PREFIX@N -> full entity ID)
//! - Entity metadata for fast lookups
//! - Feature dimensions for tolerance analysis
//!
//! The cache is user-local (gitignored) and auto-rebuilds from entity files.

use clap::Subcommand;
use console::style;
use miette::Result;
use std::fs;
use std::path::Path;

use crate::cli::args::OutputFormat;
use tdt_core::core::cache::EntityCache;
use tdt_core::core::project::Project;

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Rebuild the cache from scratch
    Rebuild,

    /// Sync cache with filesystem changes (incremental)
    Sync,

    /// Show cache statistics
    Status,

    /// Execute SQL query against cache (read-only)
    Query {
        /// SQL query to execute
        sql: String,

        /// Output format
        #[arg(short, long, default_value = "tsv")]
        format: OutputFormat,
    },

    /// Clear the cache completely
    Clear,

    /// Install git hooks for automatic cache sync after git operations
    InstallHooks {
        /// Force overwrite existing hooks
        #[arg(short, long)]
        force: bool,
    },

    /// Remove installed git hooks
    RemoveHooks,
}

pub fn run(cmd: CacheCommands) -> Result<()> {
    match cmd {
        CacheCommands::Rebuild => run_rebuild(),
        CacheCommands::Sync => run_sync(),
        CacheCommands::Status => run_status(),
        CacheCommands::Query { sql, format } => run_query(&sql, format),
        CacheCommands::Clear => run_clear(),
        CacheCommands::InstallHooks { force } => run_install_hooks(force),
        CacheCommands::RemoveHooks => run_remove_hooks(),
    }
}

fn run_rebuild() -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let mut cache = EntityCache::open_without_sync(&project)?;

    println!("{} Rebuilding cache...", style("→").blue());
    let stats = cache.rebuild()?;

    println!(
        "{} Cache rebuilt in {}ms",
        style("✓").green(),
        stats.duration_ms
    );
    println!("  Files scanned:   {}", stats.files_scanned);
    println!("  Entities cached: {}", stats.entities_added);

    Ok(())
}

fn run_sync() -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let mut cache = EntityCache::open_without_sync(&project)?;

    let stats = cache.sync()?;

    if stats.entities_added == 0 && stats.entities_updated == 0 && stats.entities_removed == 0 {
        println!("{} Cache is up to date", style("✓").green());
    } else {
        println!(
            "{} Cache synced in {}ms",
            style("✓").green(),
            stats.duration_ms
        );
        if stats.entities_added > 0 {
            println!("  Added:   {}", style(stats.entities_added).green());
        }
        if stats.entities_updated > 0 {
            println!("  Updated: {}", style(stats.entities_updated).yellow());
        }
        if stats.entities_removed > 0 {
            println!("  Removed: {}", style(stats.entities_removed).red());
        }
    }

    Ok(())
}

fn run_status() -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let cache = EntityCache::open(&project)?;

    let stats = cache.statistics()?;

    println!("{}", style("Cache Status").bold());
    println!("{}", style("─".repeat(40)).dim());
    println!(
        "  Location:        {}",
        project.root().join(".tdt/cache.db").display()
    );
    println!("  Total entities:  {}", style(stats.total_entities).cyan());
    println!("  Total short IDs: {}", style(stats.total_short_ids).cyan());
    println!(
        "  Database size:   {} KB",
        style(stats.db_size_bytes / 1024).cyan()
    );

    if !stats.by_prefix.is_empty() {
        println!();
        println!("  {} ", style("By Type:").bold());
        let mut prefixes: Vec<_> = stats.by_prefix.iter().collect();
        prefixes.sort_by_key(|(k, _)| *k);
        for (prefix, count) in prefixes {
            println!("    {:<6} {}", prefix, count);
        }
    }

    Ok(())
}

fn run_query(sql: &str, format: OutputFormat) -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let cache = EntityCache::open(&project)?;

    // Get columns
    let columns = cache.query_columns(sql)?;
    let rows = cache.query_raw(sql)?;

    match format {
        OutputFormat::Json => {
            // Output as JSON array of objects
            let json_rows: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let mut obj = serde_json::Map::new();
                    for (i, col) in columns.iter().enumerate() {
                        if let Some(val) = row.get(i) {
                            obj.insert(col.clone(), serde_json::Value::String(val.clone()));
                        }
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_rows).unwrap());
        }
        OutputFormat::Csv => {
            // CSV output
            println!("{}", columns.join(","));
            for row in rows {
                println!(
                    "{}",
                    row.iter()
                        .map(|s| escape_csv(s))
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
        }
        OutputFormat::Yaml => {
            // YAML output
            let yaml_rows: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    let mut obj = serde_json::Map::new();
                    for (i, col) in columns.iter().enumerate() {
                        if let Some(val) = row.get(i) {
                            obj.insert(col.clone(), serde_json::Value::String(val.clone()));
                        }
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();
            println!("{}", serde_yml::to_string(&yaml_rows).unwrap());
        }
        _ => {
            // Default: TSV table format
            println!("{}", columns.join("\t"));
            for row in rows {
                println!("{}", row.join("\t"));
            }
        }
    }

    Ok(())
}

fn run_clear() -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let cache_path = project.root().join(".tdt/cache.db");

    if cache_path.exists() {
        std::fs::remove_file(&cache_path)
            .map_err(|e| miette::miette!("Failed to remove cache: {}", e))?;

        // Also remove WAL and journal files if they exist
        let _ = std::fs::remove_file(project.root().join(".tdt/cache.db-journal"));
        let _ = std::fs::remove_file(project.root().join(".tdt/cache.db-wal"));
        let _ = std::fs::remove_file(project.root().join(".tdt/cache.db-shm"));

        println!("{} Cache cleared", style("✓").green());
    } else {
        println!("No cache to clear");
    }

    Ok(())
}

// ============================================================================
// Git Hooks
// ============================================================================

/// Hook script content marker to identify TDT-installed hooks
const TDT_HOOK_MARKER: &str = "# TDT-MANAGED-HOOK";

/// Post-merge hook script that syncs cache after git pull/merge
const POST_MERGE_HOOK: &str = r#"#!/bin/sh
# TDT-MANAGED-HOOK
# This hook was installed by `tdt cache install-hooks`
# It syncs the entity cache after git merge/pull operations

# Only run if tdt is available
if command -v tdt >/dev/null 2>&1; then
    # Check if we're in a TDT project (has .tdt directory)
    if [ -d ".tdt" ]; then
        tdt cache sync --quiet 2>/dev/null || true
    fi
fi
"#;

/// Post-checkout hook script that syncs cache after git checkout
const POST_CHECKOUT_HOOK: &str = r#"#!/bin/sh
# TDT-MANAGED-HOOK
# This hook was installed by `tdt cache install-hooks`
# It syncs the entity cache after git checkout operations

# Arguments: $1 = previous HEAD, $2 = new HEAD, $3 = flag (1 = branch checkout, 0 = file checkout)
# Only run for branch checkouts (flag = 1)
if [ "$3" = "1" ]; then
    # Only run if tdt is available
    if command -v tdt >/dev/null 2>&1; then
        # Check if we're in a TDT project (has .tdt directory)
        if [ -d ".tdt" ]; then
            tdt cache sync --quiet 2>/dev/null || true
        fi
    fi
fi
"#;

/// Post-rewrite hook script that syncs cache after git rebase/amend
const POST_REWRITE_HOOK: &str = r#"#!/bin/sh
# TDT-MANAGED-HOOK
# This hook was installed by `tdt cache install-hooks`
# It syncs the entity cache after git rebase/commit --amend

# Only run if tdt is available
if command -v tdt >/dev/null 2>&1; then
    # Check if we're in a TDT project (has .tdt directory)
    if [ -d ".tdt" ]; then
        tdt cache sync --quiet 2>/dev/null || true
    fi
fi
"#;

fn run_install_hooks(force: bool) -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;

    // Find the git hooks directory
    let git_dir = find_git_dir(project.root())?;
    let hooks_dir = git_dir.join("hooks");

    // Create hooks directory if it doesn't exist
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)
            .map_err(|e| miette::miette!("Failed to create hooks directory: {}", e))?;
    }

    let hooks = [
        ("post-merge", POST_MERGE_HOOK),
        ("post-checkout", POST_CHECKOUT_HOOK),
        ("post-rewrite", POST_REWRITE_HOOK),
    ];

    let mut installed = 0;
    let mut skipped = 0;

    for (hook_name, hook_content) in hooks {
        let hook_path = hooks_dir.join(hook_name);

        if hook_path.exists() {
            // Check if it's a TDT-managed hook
            let existing = fs::read_to_string(&hook_path).unwrap_or_default();
            if existing.contains(TDT_HOOK_MARKER) {
                // It's our hook, update it
                write_hook(&hook_path, hook_content)?;
                println!(
                    "  {} Updated {}",
                    style("↻").yellow(),
                    style(hook_name).cyan()
                );
                installed += 1;
            } else if force {
                // Not our hook but force flag is set
                write_hook(&hook_path, hook_content)?;
                println!(
                    "  {} Replaced {} (was not TDT-managed)",
                    style("!").yellow(),
                    style(hook_name).cyan()
                );
                installed += 1;
            } else {
                // Not our hook and no force flag
                println!(
                    "  {} Skipped {} (existing hook, use --force to replace)",
                    style("→").dim(),
                    style(hook_name).dim()
                );
                skipped += 1;
            }
        } else {
            // Hook doesn't exist, create it
            write_hook(&hook_path, hook_content)?;
            println!(
                "  {} Installed {}",
                style("✓").green(),
                style(hook_name).cyan()
            );
            installed += 1;
        }
    }

    println!();
    if installed > 0 {
        println!(
            "{} Installed {} git hook{}",
            style("✓").green(),
            installed,
            if installed == 1 { "" } else { "s" }
        );
        println!(
            "  Cache will auto-sync after: {}",
            style("git pull, merge, checkout, rebase").dim()
        );
    }
    if skipped > 0 {
        println!(
            "  {} hook{} skipped (use --force to replace)",
            skipped,
            if skipped == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

fn run_remove_hooks() -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;

    // Find the git hooks directory
    let git_dir = find_git_dir(project.root())?;
    let hooks_dir = git_dir.join("hooks");

    if !hooks_dir.exists() {
        println!("No git hooks directory found");
        return Ok(());
    }

    let hook_names = ["post-merge", "post-checkout", "post-rewrite"];
    let mut removed = 0;

    for hook_name in hook_names {
        let hook_path = hooks_dir.join(hook_name);

        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path).unwrap_or_default();
            if content.contains(TDT_HOOK_MARKER) {
                fs::remove_file(&hook_path)
                    .map_err(|e| miette::miette!("Failed to remove {}: {}", hook_name, e))?;
                println!(
                    "  {} Removed {}",
                    style("✓").green(),
                    style(hook_name).cyan()
                );
                removed += 1;
            }
        }
    }

    if removed > 0 {
        println!(
            "\n{} Removed {} TDT git hook{}",
            style("✓").green(),
            removed,
            if removed == 1 { "" } else { "s" }
        );
    } else {
        println!("No TDT-managed hooks found to remove");
    }

    Ok(())
}

/// Find the .git directory, handling worktrees
fn find_git_dir(project_root: &Path) -> Result<std::path::PathBuf> {
    let git_path = project_root.join(".git");

    if git_path.is_dir() {
        // Normal git repository
        Ok(git_path)
    } else if git_path.is_file() {
        // Git worktree - .git is a file pointing to the actual git dir
        let content = fs::read_to_string(&git_path)
            .map_err(|e| miette::miette!("Failed to read .git file: {}", e))?;

        // Parse "gitdir: /path/to/git/dir"
        if let Some(path) = content.strip_prefix("gitdir:") {
            let git_dir = path.trim();
            // For worktrees, hooks are in the main repo's hooks directory
            // The gitdir points to .git/worktrees/<name>, but hooks are in .git/hooks
            let git_dir_path = std::path::PathBuf::from(git_dir);
            if git_dir_path.join("hooks").exists() {
                return Ok(git_dir_path);
            }
            // Try parent's hooks (for worktrees)
            if let Some(parent) = git_dir_path.parent() {
                if let Some(grandparent) = parent.parent() {
                    if grandparent.join("hooks").exists() {
                        return Ok(grandparent.to_path_buf());
                    }
                }
            }
            Ok(git_dir_path)
        } else {
            Err(miette::miette!("Invalid .git file format"))
        }
    } else {
        Err(miette::miette!(
            "Not a git repository (no .git directory found)"
        ))
    }
}

/// Write a hook script with proper permissions
fn write_hook(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|e| miette::miette!("Failed to write hook: {}", e))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|e| miette::miette!("Failed to get permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)
            .map_err(|e| miette::miette!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

/// Escape a string for CSV output
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_project() -> (tempfile::TempDir, Project) {
        let tmp = tempdir().unwrap();
        let project = Project::init(tmp.path()).unwrap();
        (tmp, project)
    }

    fn write_test_entity(project: &Project, rel_path: &str, content: &str) {
        let full_path = project.root().join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
    }

    #[test]
    fn test_cache_clear() {
        let (_tmp, project) = create_test_project();

        // Create cache
        {
            let _cache = EntityCache::open(&project).unwrap();
        }

        // Verify cache exists
        let cache_path = project.root().join(".tdt/cache.db");
        assert!(cache_path.exists());

        // Clear cache
        fs::remove_file(&cache_path).unwrap();
        assert!(!cache_path.exists());
    }

    #[test]
    fn test_cache_rebuild() {
        let (_tmp, project) = create_test_project();

        write_test_entity(
            &project,
            "requirements/inputs/REQ-01ABC.tdt.yaml",
            r#"
id: REQ-01ABC
title: Test Requirement
status: draft
author: Test
created: 2024-01-15T10:30:00Z
"#,
        );

        let mut cache = EntityCache::open_without_sync(&project).unwrap();
        let stats = cache.rebuild().unwrap();

        assert_eq!(stats.entities_added, 1);
    }

    #[test]
    fn test_cache_query() {
        let (_tmp, project) = create_test_project();

        write_test_entity(
            &project,
            "requirements/inputs/REQ-01ABC.tdt.yaml",
            r#"
id: REQ-01ABC
title: Test Requirement
status: draft
author: Test
created: 2024-01-15T10:30:00Z
"#,
        );

        let mut cache = EntityCache::open_without_sync(&project).unwrap();
        cache.rebuild().unwrap();

        let rows = cache.query_raw("SELECT id, title FROM entities").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], "REQ-01ABC");
    }
}
