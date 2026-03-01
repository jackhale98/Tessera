//! `tdt check` — Validate entity approvals against workflow requirements
//!
//! Scans entities and verifies that approved/released entities have sufficient
//! approvals per the workflow configuration. Useful as a pre-push guard.

use console::style;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

use crate::cli::GlobalOpts;
use tdt_core::core::entity::Status;
use tdt_core::core::project::Project;
use tdt_core::core::workflow;
use tdt_core::services::WorkflowGuard;

/// Arguments for the check command
#[derive(clap::Args, Debug)]
pub struct CheckArgs {
    /// Only check entities changed on the current branch (vs base branch)
    #[arg(long)]
    pub branch: bool,

    /// Exit-code-only mode for git hooks (no output on success)
    #[arg(long)]
    pub push_guard: bool,
}

impl CheckArgs {
    pub fn run(&self, _global: &GlobalOpts) -> Result<()> {
        let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
        let guard = WorkflowGuard::load(&project);

        if !guard.is_enabled() {
            if !self.push_guard {
                println!(
                    "{} Workflow enforcement is not enabled (workflow.enabled: false or no team roster)",
                    style("ℹ").blue()
                );
            }
            return Ok(());
        }

        // Collect entity files to check
        let files = if self.branch {
            self.changed_entity_files(&project)?
        } else {
            self.all_entity_files(&project)?
        };

        if files.is_empty() {
            if !self.push_guard {
                println!("{} No entity files to check", style("✓").green());
            }
            return Ok(());
        }

        let mut violations: Vec<String> = Vec::new();
        let mut checked = 0;

        for file_path in &files {
            if !file_path.exists() {
                continue;
            }

            let (id, title, status) = match workflow::get_entity_info(file_path) {
                Ok(info) => info,
                Err(_) => continue, // Skip unparseable files
            };

            let prefix = match workflow::get_prefix_from_id(&id) {
                Some(p) => p,
                None => continue,
            };

            checked += 1;

            // Check approved/released entities have sufficient approvals
            if status == Status::Approved || status == Status::Released {
                let requirements = guard.approval_requirements(prefix);
                let approval_status = match workflow::get_approval_status(file_path, requirements) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if !approval_status.requirements_met {
                    let mut msg = format!(
                        "{} ({}): {} — {}/{} approvals",
                        id,
                        status,
                        title,
                        approval_status.current_approvals,
                        approval_status.required_approvals,
                    );

                    if !approval_status.missing_roles.is_empty() {
                        let roles: Vec<String> = approval_status
                            .missing_roles
                            .iter()
                            .map(|r| r.to_string())
                            .collect();
                        msg.push_str(&format!(", missing roles: {}", roles.join(", ")));
                    }

                    violations.push(msg);
                }
            }
        }

        if violations.is_empty() {
            if !self.push_guard {
                println!(
                    "{} All {} entities pass approval requirements",
                    style("✓").green(),
                    checked
                );
            }
            Ok(())
        } else {
            if !self.push_guard {
                println!(
                    "{} {} violation(s) found:\n",
                    style("✗").red(),
                    violations.len()
                );
                for v in &violations {
                    println!("  {} {}", style("•").red(), v);
                }
                println!();
                println!(
                    "  Entities are {} or {} but don't meet approval requirements.",
                    style("approved").cyan(),
                    style("released").cyan()
                );
            }
            // Return error exit code
            Err(miette::miette!(
                "{} entities with missing approvals",
                violations.len()
            ))
        }
    }

    /// Get all .tdt.yaml entity files in the project
    fn all_entity_files(&self, project: &Project) -> Result<Vec<PathBuf>> {
        let root = project.root();
        let dirs = entity_dirs();
        let mut files = Vec::new();

        for dir in &dirs {
            let full_dir = root.join(dir);
            if !full_dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&full_dir).into_diagnostic()? {
                let entry = entry.into_diagnostic()?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "yaml")
                    && path
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().contains(".tdt."))
                {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Get entity files changed on the current branch relative to the base branch
    fn changed_entity_files(&self, project: &Project) -> Result<Vec<PathBuf>> {
        let root = project.root();

        // Get base branch from workflow config or default to "main"
        let guard = WorkflowGuard::load(project);
        let base_branch = &guard.config().base_branch;
        let base = if base_branch.is_empty() {
            "main"
        } else {
            base_branch
        };

        // Use git diff to find changed .tdt.yaml files
        let output = std::process::Command::new("git")
            .args(["diff", "--name-only", &format!("{}...HEAD", base)])
            .current_dir(root)
            .output()
            .into_diagnostic()?;

        if !output.status.success() {
            // If we can't determine branch changes, fall back to all files
            if self.push_guard {
                // In push-guard mode, silently pass if git diff fails
                return Ok(Vec::new());
            }
            return self.all_entity_files(project);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<PathBuf> = stdout
            .lines()
            .filter(|line| line.ends_with(".tdt.yaml") || line.ends_with(".pdt.yaml"))
            .map(|line| root.join(line))
            .filter(|p| p.exists())
            .collect();

        Ok(files)
    }
}

/// All entity directories in the project
fn entity_dirs() -> Vec<&'static str> {
    vec![
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
    ]
}
