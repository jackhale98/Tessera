//! Reject command - Reject entities back to draft

use clap::Args;
use miette::{bail, IntoDiagnostic, Result};
use std::path::PathBuf;

use crate::cli::args::GlobalOpts;
use tdt_core::core::entity::Status;
use tdt_core::core::shortid::ShortIdIndex;
use tdt_core::core::workflow::{get_entity_info, record_rejection, truncate_id};
use tdt_core::core::{Config, Git, Project, Provider, ProviderClient};

/// Reject entities back to draft status
#[derive(Debug, Args)]
pub struct RejectArgs {
    /// Entity IDs to reject (accepts multiple, or - for stdin)
    #[arg(required_unless_present = "pr")]
    pub ids: Vec<String>,

    /// Reject all entities in a PR by PR number
    #[arg(long)]
    pub pr: Option<u64>,

    /// Rejection reason (required)
    #[arg(long, short = 'r')]
    pub reason: String,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,

    /// Show what would be done without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Print commands as they run
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Skip git commit (just update YAML files)
    #[arg(long)]
    pub no_commit: bool,
}

impl RejectArgs {
    pub fn run(&self, _global: &GlobalOpts) -> Result<()> {
        let project = Project::discover().into_diagnostic()?;
        let config = Config::load();

        // Check if workflow is enabled
        if !config.workflow.enabled {
            bail!(
                "Workflow features are not enabled.\n\
                 Add the following to .tdt/config.yaml:\n\n\
                 workflow:\n\
                 \x20 enabled: true\n\
                 \x20 provider: github  # or gitlab, or none"
            );
        }

        let git = Git::new(project.root());
        let has_git = git.is_repo();

        // Get rejector name - from git config, env var, or fallback
        let rejector_name = if has_git {
            git.user_name().unwrap_or_else(|_| "Unknown".to_string())
        } else {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "Unknown".to_string())
        };

        // Collect entity IDs
        let ids = self.collect_entity_ids(&project, &config)?;
        if ids.is_empty() {
            bail!("No entities to reject. Specify IDs or use --pr");
        }

        // Resolve short IDs to full IDs and validate
        let short_index = ShortIdIndex::load(&project);
        let mut entities: Vec<(PathBuf, String, String, Status)> = Vec::new();

        for id in &ids {
            let full_id = short_index
                .resolve(id)
                .ok_or_else(|| miette::miette!("Cannot resolve ID: {}", id))?;
            let file_path = self.find_entity_file(&project, &full_id)?;
            let (entity_id, title, status) = get_entity_info(&file_path).into_diagnostic()?;

            if status != Status::Review {
                bail!(
                    "Entity {} is not in review status (current: {})",
                    entity_id,
                    status
                );
            }

            entities.push((file_path, entity_id, title, status));
        }

        // Show what we're about to do
        println!(
            "Rejecting {} entities as {}...",
            entities.len(),
            rejector_name
        );
        println!("Reason: {}", self.reason);
        if self.verbose || self.dry_run {
            for (_, id, title, _) in &entities {
                println!("  {}  {}", truncate_id(id), title);
            }
        }

        if self.dry_run {
            self.print_dry_run(&entities, &config, has_git)?;
            println!("\nNo changes made (dry run).");
            return Ok(());
        }

        // Confirm if not --yes
        if !self.yes {
            print!("Proceed? [y/N] ");
            std::io::Write::flush(&mut std::io::stdout()).into_diagnostic()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).into_diagnostic()?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        // Execute the rejection
        self.execute_reject(&project, &config, &git, has_git, &entities, &rejector_name)?;

        Ok(())
    }

    fn collect_entity_ids(&self, project: &Project, config: &Config) -> Result<Vec<String>> {
        super::utils::collect_entity_ids_from_args(&self.ids, self.pr, project, config, self.verbose)
    }

    fn find_entity_file(&self, project: &Project, id: &str) -> Result<PathBuf> {
        super::utils::find_entity_file(project, id)
    }

    fn print_dry_run(
        &self,
        entities: &[(PathBuf, String, String, Status)],
        config: &Config,
        has_git: bool,
    ) -> Result<()> {
        println!("\nWould execute:");

        for (path, _id, _, _) in entities {
            let rel_path = path
                .strip_prefix(std::env::current_dir().into_diagnostic()?)
                .unwrap_or(path)
                .display();
            println!("  [record rejection in {}]", rel_path);
            if has_git && !self.no_commit {
                println!("  git add {}", rel_path);
            }
        }

        if has_git && !self.no_commit {
            let commit_message = if entities.len() == 1 {
                let (_, id, _, _) = &entities[0];
                format!("Reject {}: {}", truncate_id(id), self.reason)
            } else {
                format!("Reject {} entities: {}", entities.len(), self.reason)
            };
            println!("  git commit -m \"{}\"", commit_message);
        }

        if config.workflow.provider != Provider::None {
            if let Some(pr) = self.pr {
                let provider =
                    ProviderClient::new(config.workflow.provider, std::path::Path::new("."));
                println!(
                    "  {}",
                    provider.format_command(&["pr", "close", &pr.to_string()])
                );
            }
        }

        Ok(())
    }

    fn execute_reject(
        &self,
        project: &Project,
        config: &Config,
        git: &Git,
        has_git: bool,
        entities: &[(PathBuf, String, String, Status)],
        rejector_name: &str,
    ) -> Result<()> {
        // Record rejection in each entity
        for (path, id, _, _) in entities {
            record_rejection(path, rejector_name, &self.reason).into_diagnostic()?;
            if self.verbose {
                eprintln!("  Recorded rejection in {}", truncate_id(id));
            }
        }
        println!(
            "  Rejected {} entities (status: review → draft)",
            entities.len()
        );

        // Git operations are optional - only if we have git and not --no-commit
        let should_commit = has_git && !self.no_commit;

        if should_commit {
            // Stage files
            let paths: Vec<&std::path::Path> =
                entities.iter().map(|(p, _, _, _)| p.as_path()).collect();
            git.stage_files(&paths).into_diagnostic()?;

            // Commit
            let commit_message = if entities.len() == 1 {
                let (_, id, _, _) = &entities[0];
                format!("Reject {}: {}", truncate_id(id), self.reason)
            } else {
                format!("Reject {} entities: {}", entities.len(), self.reason)
            };
            let _hash = git.commit(&commit_message).into_diagnostic()?;
            println!("  Committed: \"{}\"", commit_message);

            // Close PR if provider is configured
            if config.workflow.provider != Provider::None {
                let provider = ProviderClient::new(config.workflow.provider, project.root())
                    .with_verbose(self.verbose);

                // Find PR for current branch
                let current_branch = git.current_branch().unwrap_or_default();
                if let Ok(Some(pr_info)) = provider.get_pr_for_branch(&current_branch) {
                    let comment = format!("Rejected: {}", self.reason);
                    if let Err(e) = provider.close_pr(pr_info.number, Some(&comment)) {
                        eprintln!("  Warning: Failed to close PR: {}", e);
                    } else {
                        println!("  Closed PR #{}", pr_info.number);
                    }
                }
            }
        }

        println!("\n{} entities rejected.", entities.len());

        Ok(())
    }
}
