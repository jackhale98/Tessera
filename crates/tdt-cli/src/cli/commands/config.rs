//! `tdt config` command - Configuration management
//!
//! Provides commands to view and modify TDT configuration.

use clap::Subcommand;
use console::style;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::PathBuf;

use crate::cli::GlobalOpts;
use tdt_core::core::project::Project;
use tdt_core::core::Config;

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration values
    Show(ShowArgs),

    /// Set a configuration value
    Set(SetArgs),

    /// Unset (remove) a configuration value
    Unset(UnsetArgs),

    /// Show paths to configuration files
    Path(PathArgs),

    /// List all available configuration keys
    Keys,
}

#[derive(clap::Args, Debug)]
pub struct ShowArgs {
    /// Show only this key's value
    pub key: Option<String>,

    /// Show only project-level config
    #[arg(long = "project-only")]
    pub project_only: bool,

    /// Show only global (user) config
    #[arg(long = "global-only")]
    pub global_only: bool,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    /// Configuration key (e.g., author, editor)
    pub key: String,

    /// Value to set
    pub value: String,

    /// Set in global (user) config instead of project config
    #[arg(long, short = 'g')]
    pub global: bool,
}

#[derive(clap::Args, Debug)]
pub struct UnsetArgs {
    /// Configuration key to remove
    pub key: String,

    /// Remove from global (user) config instead of project config
    #[arg(long, short = 'g')]
    pub global: bool,
}

#[derive(clap::Args, Debug)]
pub struct PathArgs {
    /// Show only project config path
    #[arg(long = "project-only")]
    pub project_only: bool,

    /// Show only global config path
    #[arg(long = "global-only")]
    pub global_only: bool,
}

/// Configuration key categories for organized display
const CONFIG_CATEGORIES: &[(&str, &[(&str, &str, &str)])] = &[
    (
        "General",
        &[
            ("author", "Default author for new entities", "\"Jane Smith\""),
            ("editor", "Editor command for `tdt edit`", "vim"),
            ("pager", "Pager command for long output", "less"),
            (
                "default_format",
                "Default output format",
                "yaml  # yaml, json, tsv",
            ),
        ],
    ),
    (
        "Workflow",
        &[
            (
                "workflow.enabled",
                "Enable workflow commands",
                "true  # true/false",
            ),
            (
                "workflow.provider",
                "Git hosting provider",
                "github  # github, gitlab, none",
            ),
            (
                "workflow.auto_commit",
                "Auto-commit on status changes",
                "true  # true/false",
            ),
            (
                "workflow.auto_merge",
                "Merge PR automatically on approval",
                "false  # true/false",
            ),
            (
                "workflow.base_branch",
                "Target branch for PRs",
                "main",
            ),
        ],
    ),
    (
        "Approval Requirements (defaults)",
        &[
            (
                "workflow.default_approvals.min_approvals",
                "Min approvals required",
                "1",
            ),
            (
                "workflow.default_approvals.require_unique_approvers",
                "Require different approvers",
                "true  # true/false",
            ),
            (
                "workflow.default_approvals.require_signature",
                "Require GPG-signed approvals (Part 11)",
                "false  # true/false",
            ),
        ],
    ),
    (
        "Approval Requirements (per entity type)",
        &[
            (
                "workflow.approvals.REQ.min_approvals",
                "Min approvals for REQ entities",
                "2",
            ),
            (
                "workflow.approvals.RISK.min_approvals",
                "Min approvals for RISK entities",
                "2",
            ),
            (
                "workflow.approvals.RISK.require_signature",
                "Require GPG signature for RISK",
                "true",
            ),
        ],
    ),
];

/// Run a config subcommand
pub fn run(cmd: ConfigCommands, _global: &GlobalOpts) -> Result<()> {
    match cmd {
        ConfigCommands::Show(args) => run_show(args),
        ConfigCommands::Set(args) => run_set(args),
        ConfigCommands::Unset(args) => run_unset(args),
        ConfigCommands::Path(args) => run_path(args),
        ConfigCommands::Keys => run_keys(),
    }
}

fn run_show(args: ShowArgs) -> Result<()> {
    let config = Config::load();

    // If a specific key is requested, show just that value
    if let Some(key) = &args.key {
        let value = get_config_value(&config, key);
        if let Some(v) = value {
            println!("{}", v);
        } else {
            return Err(miette::miette!("Key '{}' is not set", key));
        }
        return Ok(());
    }

    // Show all config values
    if args.project_only && args.global_only {
        return Err(miette::miette!(
            "Cannot specify both --project-only and --global-only"
        ));
    }

    if args.project_only {
        show_project_config()?;
    } else if args.global_only {
        show_global_config()?;
    } else {
        // Show merged/effective config
        println!("{}", style("Effective Configuration").bold().underlined());
        println!();

        println!("  {}", style("[General]").bold());
        print_config_value("author", config.author.as_deref());
        print_config_value("editor", config.editor.as_deref());
        print_config_value("pager", config.pager.as_deref());
        print_config_value("default_format", config.default_format.as_deref());

        println!();
        println!("  {}", style("[Workflow]").bold());
        print_config_value(
            "workflow.enabled",
            Some(if config.workflow.enabled {
                "true"
            } else {
                "false"
            }),
        );
        print_config_value(
            "workflow.provider",
            Some(&format!("{:?}", config.workflow.provider).to_lowercase()),
        );
        print_config_value("workflow.base_branch", Some(&config.workflow.base_branch));
        print_config_value(
            "workflow.auto_commit",
            Some(if config.workflow.auto_commit {
                "true"
            } else {
                "false"
            }),
        );
        print_config_value(
            "workflow.auto_merge",
            Some(if config.workflow.auto_merge {
                "true"
            } else {
                "false"
            }),
        );

        println!();
        println!("  {}", style("[Approval Defaults]").bold());
        print_config_value(
            "workflow.default_approvals.min_approvals",
            Some(&config.workflow.default_approvals.min_approvals.to_string()),
        );
        print_config_value(
            "workflow.default_approvals.require_unique_approvers",
            Some(if config.workflow.default_approvals.require_unique_approvers {
                "true"
            } else {
                "false"
            }),
        );
        print_config_value(
            "workflow.default_approvals.require_signature",
            Some(if config.workflow.default_approvals.require_signature {
                "true"
            } else {
                "false"
            }),
        );

        if !config.workflow.approvals.is_empty() {
            println!();
            println!("  {}", style("[Per-Entity Approvals]").bold());
            for (entity_type, reqs) in &config.workflow.approvals {
                print_config_value(
                    &format!("workflow.approvals.{}.min_approvals", entity_type),
                    Some(&reqs.min_approvals.to_string()),
                );
                if reqs.require_signature {
                    print_config_value(
                        &format!("workflow.approvals.{}.require_signature", entity_type),
                        Some("true"),
                    );
                }
            }
        }

        // Show source info
        println!();
        println!("{}", style("Config Sources (in priority order):").dim());
        println!("  1. Environment variables (TDT_AUTHOR, TDT_EDITOR)");
        println!("  2. Project config (.tdt/config.yaml)");
        println!("  3. Global config (~/.config/tdt/config.yaml)");
    }

    Ok(())
}

fn run_set(args: SetArgs) -> Result<()> {
    let config_path = if args.global {
        get_global_config_path()?
    } else {
        get_project_config_path()?
    };

    // Load existing config or create new
    let mut config_map: serde_yml::Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path).into_diagnostic()?;
        let parsed: serde_yml::Value =
            serde_yml::from_str(&content).unwrap_or(serde_yml::Value::Mapping(Default::default()));
        // If the file was empty or null, use an empty mapping
        if parsed.is_null() {
            serde_yml::Value::Mapping(Default::default())
        } else {
            parsed
        }
    } else {
        serde_yml::Value::Mapping(Default::default())
    };

    // Set the value (handle nested keys like "aliases.r")
    set_nested_value(&mut config_map, &args.key, &args.value)?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }

    // Write back
    let yaml = serde_yml::to_string(&config_map).into_diagnostic()?;
    fs::write(&config_path, yaml).into_diagnostic()?;

    let scope = if args.global { "global" } else { "project" };
    println!(
        "{} Set {} {} = {} in {} config",
        style("✓").green(),
        style(&args.key).cyan(),
        style("→").dim(),
        style(&args.value).yellow(),
        scope
    );

    Ok(())
}

fn run_unset(args: UnsetArgs) -> Result<()> {
    let config_path = if args.global {
        get_global_config_path()?
    } else {
        get_project_config_path()?
    };

    if !config_path.exists() {
        return Err(miette::miette!(
            "Config file does not exist: {}",
            config_path.display()
        ));
    }

    let content = fs::read_to_string(&config_path).into_diagnostic()?;
    let mut config_map: serde_yml::Value =
        serde_yml::from_str(&content).unwrap_or(serde_yml::Value::Mapping(Default::default()));

    // Remove the value (handle nested keys like "aliases.r")
    let removed = unset_nested_value(&mut config_map, &args.key)?;

    if !removed {
        return Err(miette::miette!("Key '{}' not found in config", args.key));
    }

    let yaml = serde_yml::to_string(&config_map).into_diagnostic()?;
    fs::write(&config_path, yaml).into_diagnostic()?;

    let scope = if args.global { "global" } else { "project" };
    println!(
        "{} Removed {} from {} config",
        style("✓").green(),
        style(&args.key).cyan(),
        scope
    );

    Ok(())
}

fn run_path(args: PathArgs) -> Result<()> {
    if args.project_only && args.global_only {
        return Err(miette::miette!(
            "Cannot specify both --project-only and --global-only"
        ));
    }

    if args.project_only {
        let path = get_project_config_path()?;
        println!("{}", path.display());
    } else if args.global_only {
        let path = get_global_config_path()?;
        println!("{}", path.display());
    } else {
        let global_path = get_global_config_path()?;
        let project_path = get_project_config_path();

        println!("{}", style("Configuration file paths:").bold());
        println!();
        println!("  {} {}", style("Global:").cyan(), global_path.display());
        if global_path.exists() {
            println!("         {}", style("(exists)").green());
        } else {
            println!("         {}", style("(not created)").dim());
        }

        if let Ok(path) = project_path {
            println!();
            println!("  {} {}", style("Project:").cyan(), path.display());
            if path.exists() {
                println!("          {}", style("(exists)").green());
            } else {
                println!("          {}", style("(not created)").dim());
            }
        } else {
            println!();
            println!(
                "  {} {}",
                style("Project:").cyan(),
                style("(not in a TDT project)").dim()
            );
        }
    }

    Ok(())
}

fn run_keys() -> Result<()> {
    println!("{}", style("Configuration Keys").bold().underlined());
    println!();

    for (category, keys) in CONFIG_CATEGORIES {
        println!("  {}", style(format!("[{}]", category)).bold());
        for (key, description, example) in *keys {
            println!(
                "    {:<50} {}",
                style(*key).cyan(),
                style(*description).dim()
            );
            println!(
                "    {:<50} {}",
                "",
                style(format!("example: tdt config set {} {}", key, example)).dim()
            );
        }
        println!();
    }

    println!("{}", style("Quick Start — Enable Workflow").bold());
    println!();
    println!("  tdt config set workflow.enabled true");
    println!("  tdt config set workflow.provider github");
    println!();
    println!(
        "  {}",
        style("Then use: tdt submit, tdt approve, tdt reject, tdt release").dim()
    );
    println!();
    println!("{}", style("Other Commands").bold());
    println!();
    println!("  tdt config show              Show effective configuration");
    println!("  tdt config set <key> <value>  Set a project config value");
    println!("  tdt config set -g <key> <val> Set a global (user) config value");
    println!("  tdt config unset <key>        Remove a config value");
    println!("  tdt config path               Show config file locations");

    Ok(())
}

// Helper functions

fn get_global_config_path() -> Result<PathBuf> {
    directories::ProjectDirs::from("", "", "tdt")
        .map(|dirs| dirs.config_dir().join("config.yaml"))
        .ok_or_else(|| miette::miette!("Could not determine global config directory"))
}

fn get_project_config_path() -> Result<PathBuf> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    Ok(project.tdt_dir().join("config.yaml"))
}

fn get_config_value(config: &Config, key: &str) -> Option<String> {
    match key {
        "author" => config.author.clone(),
        "editor" => config.editor.clone(),
        "pager" => config.pager.clone(),
        "default_format" => config.default_format.clone(),
        "workflow.enabled" => Some(config.workflow.enabled.to_string()),
        "workflow.provider" => Some(format!("{:?}", config.workflow.provider).to_lowercase()),
        "workflow.auto_commit" => Some(config.workflow.auto_commit.to_string()),
        "workflow.auto_merge" => Some(config.workflow.auto_merge.to_string()),
        "workflow.base_branch" => Some(config.workflow.base_branch.clone()),
        "workflow.default_approvals.min_approvals" => {
            Some(config.workflow.default_approvals.min_approvals.to_string())
        }
        "workflow.default_approvals.require_unique_approvers" => Some(
            config
                .workflow
                .default_approvals
                .require_unique_approvers
                .to_string(),
        ),
        "workflow.default_approvals.require_signature" => Some(
            config
                .workflow
                .default_approvals
                .require_signature
                .to_string(),
        ),
        _ => {
            // Handle dynamic per-entity approval keys like workflow.approvals.RISK.min_approvals
            if let Some(rest) = key.strip_prefix("workflow.approvals.") {
                let parts: Vec<&str> = rest.splitn(2, '.').collect();
                if parts.len() == 2 {
                    if let Some(reqs) = config.workflow.approvals.get(parts[0]) {
                        return match parts[1] {
                            "min_approvals" => Some(reqs.min_approvals.to_string()),
                            "require_unique_approvers" => {
                                Some(reqs.require_unique_approvers.to_string())
                            }
                            "require_signature" => Some(reqs.require_signature.to_string()),
                            _ => None,
                        };
                    }
                }
            }
            None
        }
    }
}

fn print_config_value(key: &str, value: Option<&str>) {
    if let Some(v) = value {
        println!("  {}: {}", style(key).cyan(), style(v).yellow());
    } else {
        println!("  {}: {}", style(key).cyan(), style("(not set)").dim());
    }
}

fn show_project_config() -> Result<()> {
    let path = get_project_config_path()?;

    println!(
        "{} {}",
        style("Project config:").bold(),
        style(path.display()).dim()
    );
    println!();

    if path.exists() {
        let content = fs::read_to_string(&path).into_diagnostic()?;
        print!("{}", content);
    } else {
        println!("{}", style("(not created)").dim());
    }

    Ok(())
}

fn show_global_config() -> Result<()> {
    let path = get_global_config_path()?;

    println!(
        "{} {}",
        style("Global config:").bold(),
        style(path.display()).dim()
    );
    println!();

    if path.exists() {
        let content = fs::read_to_string(&path).into_diagnostic()?;
        print!("{}", content);
    } else {
        println!("{}", style("(not created)").dim());
    }

    Ok(())
}

fn set_nested_value(root: &mut serde_yml::Value, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    let mut current = root;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let serde_yml::Value::Mapping(map) = current {
                map.insert(
                    serde_yml::Value::String(part.to_string()),
                    serde_yml::Value::String(value.to_string()),
                );
            }
        } else {
            // Intermediate part - navigate or create
            if let serde_yml::Value::Mapping(map) = current {
                let key = serde_yml::Value::String(part.to_string());
                if !map.contains_key(&key) {
                    map.insert(key.clone(), serde_yml::Value::Mapping(Default::default()));
                }
                current = map.get_mut(&key).unwrap();
            }
        }
    }

    Ok(())
}

fn unset_nested_value(root: &mut serde_yml::Value, key: &str) -> Result<bool> {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.len() == 1 {
        // Simple key
        if let serde_yml::Value::Mapping(map) = root {
            let key = serde_yml::Value::String(parts[0].to_string());
            return Ok(map.remove(&key).is_some());
        }
    } else {
        // Nested key - navigate to parent and remove
        let mut current = root;
        for part in &parts[..parts.len() - 1] {
            if let serde_yml::Value::Mapping(map) = current {
                let key = serde_yml::Value::String(part.to_string());
                if let Some(next) = map.get_mut(&key) {
                    current = next;
                } else {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        // Remove the final key
        if let serde_yml::Value::Mapping(map) = current {
            let key = serde_yml::Value::String(parts.last().unwrap().to_string());
            return Ok(map.remove(&key).is_some());
        }
    }

    Ok(false)
}
