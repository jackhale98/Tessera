//! Shared entity command infrastructure
//!
//! This module provides common patterns for entity CRUD operations,
//! reducing boilerplate across the 20+ command files.
//!
//! Note: Some of this infrastructure is not yet adopted - individual entity
//! commands have their own implementations. Kept for potential future refactoring.

#![allow(dead_code)]

use console::style;
use miette::{IntoDiagnostic, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::cli::{GlobalOpts, OutputFormat};
use tdt_core::core::identity::{EntityId, EntityPrefix};
use tdt_core::core::project::Project;
use tdt_core::core::shortid::ShortIdIndex;
use tdt_core::core::Config;

// =========================================================================
// Entity Configuration
// =========================================================================

/// Static configuration for an entity type
pub struct EntityConfig {
    /// Entity prefix (e.g., EntityPrefix::Sup)
    pub prefix: EntityPrefix,
    /// Directories where entities are stored (e.g., &["bom/suppliers"])
    pub dirs: &'static [&'static str],
    /// Singular name for messages (e.g., "supplier")
    pub name: &'static str,
    /// Plural name for messages (e.g., "suppliers")
    pub name_plural: &'static str,
}

// =========================================================================
// Common Show Implementation
// =========================================================================

/// Generic show command that handles YAML/JSON/ID output formats
///
/// For pretty output (default), call the entity-specific pretty printer after this.
pub fn run_show_generic<T>(
    id: &str,
    config: &EntityConfig,
    global: &GlobalOpts,
) -> Result<Option<(T, PathBuf)>>
where
    T: DeserializeOwned + Serialize,
{
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;

    // Resolve short ID if needed
    let short_ids = ShortIdIndex::load(&project);
    let resolved_id = short_ids.resolve(id).unwrap_or_else(|| id.to_string());

    // Find the entity file
    let path = find_entity_file(&project, &resolved_id, config.dirs)?;

    // Read and parse entity
    let content = fs::read_to_string(&path).into_diagnostic()?;
    let entity: T = serde_yml::from_str(&content).into_diagnostic()?;

    match global.output {
        OutputFormat::Yaml => {
            print!("{}", content);
            Ok(None)
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&entity).into_diagnostic()?;
            println!("{}", json);
            Ok(None)
        }
        OutputFormat::Id => {
            // For ID output, we need to extract the ID - caller handles this
            Ok(Some((entity, path)))
        }
        OutputFormat::ShortId => {
            // For ShortId output, caller handles this
            Ok(Some((entity, path)))
        }
        _ => {
            // Return entity for pretty printing
            Ok(Some((entity, path)))
        }
    }
}

/// Print entity ID in the requested format
pub fn print_entity_id(id: &EntityId, format: OutputFormat, project: &Project) {
    match format {
        OutputFormat::ShortId => {
            let short_ids = ShortIdIndex::load(project);
            let short_id = short_ids.get_short_id(&id.to_string()).unwrap_or_default();
            println!("{}", short_id);
        }
        _ => {
            println!("{}", id);
        }
    }
}

// =========================================================================
// Common Edit Implementation
// =========================================================================

/// Generic edit command
pub fn run_edit_generic(id: &str, config: &EntityConfig) -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let cli_config = Config::load();

    // Resolve short ID if needed
    let short_ids = ShortIdIndex::load(&project);
    let resolved_id = short_ids.resolve(id).unwrap_or_else(|| id.to_string());

    // Find the entity file
    let path = find_entity_file(&project, &resolved_id, config.dirs)?;

    println!(
        "Opening {} in {}...",
        style(path.display()).cyan(),
        style(cli_config.editor()).yellow()
    );

    cli_config.run_editor(&path).into_diagnostic()?;

    // Sync cache after editing (in case user saved changes)
    crate::cli::commands::utils::sync_cache(&project);

    Ok(())
}

// =========================================================================
// File Finding Utilities
// =========================================================================

/// Find an entity file in the given directories
pub fn find_entity_file(
    project: &Project,
    entity_id: &str,
    entity_dirs: &[&str],
) -> Result<PathBuf> {
    for dir in entity_dirs {
        let dir_path = project.root().join(dir);
        if !dir_path.exists() {
            continue;
        }

        for entry in fs::read_dir(&dir_path).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "yaml") {
                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if filename.contains(entity_id) || filename.starts_with(entity_id) {
                    return Ok(path);
                }
            }
        }
    }

    Err(miette::miette!("No entity found matching '{}'", entity_id))
}

// =========================================================================
// Status Filter Conversion
// =========================================================================

use crate::cli::filters::StatusFilter;

/// Convert StatusFilter to cache-compatible Option<&str>
pub fn status_filter_to_str(filter: StatusFilter) -> Option<&'static str> {
    match filter {
        StatusFilter::Draft => Some("draft"),
        StatusFilter::Review => Some("review"),
        StatusFilter::Approved => Some("approved"),
        StatusFilter::Released => Some("released"),
        StatusFilter::Obsolete => Some("obsolete"),
        StatusFilter::Active | StatusFilter::All => None,
    }
}

/// Check if a status string matches the filter
pub fn status_matches_filter(status: &str, filter: StatusFilter) -> bool {
    match filter {
        StatusFilter::Draft => status == "draft",
        StatusFilter::Review => status == "review",
        StatusFilter::Approved => status == "approved",
        StatusFilter::Released => status == "released",
        StatusFilter::Obsolete => status == "obsolete",
        StatusFilter::Active => status != "obsolete",
        StatusFilter::All => true,
    }
}

/// Check if a Status enum matches the filter
pub fn status_enum_matches_filter(
    status: &tdt_core::core::entity::Status,
    filter: StatusFilter,
) -> bool {
    use tdt_core::core::entity::Status;
    match filter {
        StatusFilter::Draft => *status == Status::Draft,
        StatusFilter::Review => *status == Status::Review,
        StatusFilter::Approved => *status == Status::Approved,
        StatusFilter::Released => *status == Status::Released,
        StatusFilter::Obsolete => *status == Status::Obsolete,
        StatusFilter::Active => *status != Status::Obsolete,
        StatusFilter::All => true,
    }
}

/// Convert StatusFilter to Option<Status> for EntityFilter
pub fn status_filter_to_status(filter: StatusFilter) -> Option<tdt_core::core::entity::Status> {
    use tdt_core::core::entity::Status;
    match filter {
        StatusFilter::Draft => Some(Status::Draft),
        StatusFilter::Review => Some(Status::Review),
        StatusFilter::Approved => Some(Status::Approved),
        StatusFilter::Released => Some(Status::Released),
        StatusFilter::Obsolete => Some(Status::Obsolete),
        StatusFilter::Active | StatusFilter::All => None,
    }
}

// =========================================================================
// Common New Output Helpers
// =========================================================================

/// Output format for newly created entity
pub fn output_new_entity(
    id: &EntityId,
    file_path: &std::path::Path,
    short_id: Option<String>,
    entity_name: &str,
    title: &str,
    extra_info: Option<&str>,
    added_links: &[(String, String)],
    global: &GlobalOpts,
) {
    use crate::cli::helpers::format_short_id;

    match global.output {
        OutputFormat::Id => {
            println!("{}", id);
        }
        OutputFormat::ShortId => {
            println!("{}", short_id.unwrap_or_else(|| format_short_id(id)));
        }
        OutputFormat::Path => {
            println!("{}", file_path.display());
        }
        _ => {
            let display_id = short_id.unwrap_or_else(|| format_short_id(id));
            println!(
                "{} Created {} {}",
                style("✓").green(),
                entity_name,
                style(&display_id).cyan()
            );
            println!("   {}", style(file_path.display()).dim());

            if let Some(info) = extra_info {
                println!("   {}", info);
            } else {
                println!("   {}", style(title).yellow());
            }

            // Show added links
            for (link_type, target) in added_links {
                if let Ok(target_id) = EntityId::parse(target) {
                    println!(
                        "   {} --[{}]--> {}",
                        style("→").dim(),
                        style(link_type).cyan(),
                        style(format_short_id(&target_id)).yellow()
                    );
                }
            }
        }
    }
}

// =========================================================================
// Common List Output Helpers
// =========================================================================

/// Print "No X found" message
pub fn print_no_results(name_plural: &str) {
    println!("No {} found.", name_plural);
}

/// Print list footer with count
pub fn print_list_footer(count: usize, prefix: EntityPrefix) {
    println!();
    println!(
        "{} {}(es) found. Use {} to reference by short ID.",
        style(count).cyan(),
        prefix,
        style(format!("{}@N", prefix)).cyan()
    );
}

// =========================================================================
// Generic List Implementation
// =========================================================================

use crate::cli::table::{ColumnDef, TableConfig, TableFormatter, TableRow};
use tdt_core::core::entity::Entity;
use tdt_core::services::{ListableService, SortDirection};

/// Configuration for generic list commands
pub struct ListConfig<E, C, S> {
    /// Column definitions for table output
    pub columns: &'static [ColumnDef],
    /// Entity name for messages (e.g., "requirement")
    pub entity_name: &'static str,
    /// Entity prefix string for table footer (e.g., "REQ")
    pub prefix_str: &'static str,
    /// Convert full entity to table row
    pub entity_to_row: fn(&E, &ShortIdIndex) -> TableRow,
    /// Convert cached entity to table row
    pub cached_to_row: fn(&C, &ShortIdIndex) -> TableRow,
    /// Optional function to sort cached entities
    pub cached_sort: Option<fn(&mut Vec<C>, S, SortDirection)>,
}

/// Common list arguments extracted from CLI args
#[derive(Debug, Clone)]
pub struct CommonListArgs {
    pub columns: Vec<String>,
    pub limit: Option<usize>,
    pub reverse: bool,
    pub count: bool,
    pub wrap: Option<usize>,
}

/// Run a generic list command using a service
///
/// This function handles:
/// - Two-tier caching (cache for table output, full entities for JSON/YAML)
/// - All output formats (table, tsv, csv, md, json, yaml, id, shortid)
/// - Sorting, limiting, reversing
/// - Short ID assignment
///
/// # Arguments
/// * `config` - Entity-specific list configuration
/// * `service` - The service implementing ListableService
/// * `filter` - Entity-specific filter
/// * `sort_field` - Field to sort by
/// * `sort_dir` - Sort direction
/// * `args` - Common list arguments (columns, limit, etc.)
/// * `global` - Global CLI options (output format)
/// * `project` - Project reference
/// * `needs_full_for_filter` - True if post-filtering requires full entities
pub fn run_list_generic<E, C, F, S, Svc>(
    config: &ListConfig<E, C, S>,
    service: &Svc,
    filter: &F,
    sort_field: S,
    sort_dir: SortDirection,
    args: &CommonListArgs,
    global: &GlobalOpts,
    project: &Project,
    needs_full_for_filter: bool,
) -> Result<()>
where
    E: Entity + Serialize + Clone,
    C: Clone,
    S: Copy,
    Svc: ListableService<E, C, F, S>,
{
    let format = match global.output {
        OutputFormat::Auto => OutputFormat::Tsv,
        f => f,
    };

    let needs_full_output = matches!(format, OutputFormat::Json | OutputFormat::Yaml);
    let needs_full = needs_full_output || needs_full_for_filter;

    if needs_full {
        // Full entity path - required for JSON/YAML or cross-entity filters
        let result = service
            .list(filter, sort_field, sort_dir)
            .map_err(|e| miette::miette!("{}", e))?;

        let mut items = result.items;

        // Apply reverse and limit
        if args.reverse {
            items.reverse();
        }
        if let Some(limit) = args.limit {
            items.truncate(limit);
        }

        // Count only mode
        if args.count {
            println!("{}", items.len());
            return Ok(());
        }

        if items.is_empty() {
            return output_empty_list(config.entity_name, format);
        }

        // Update short IDs
        let mut short_ids = ShortIdIndex::load(project);
        short_ids.ensure_all(items.iter().map(|e| e.id().to_string()));
        super::commands::utils::save_short_ids(&mut short_ids, project);

        output_full_entities(&items, config, &short_ids, args, format)
    } else {
        // Cache path - fast, no YAML parsing needed
        let result = service
            .list_cached(filter)
            .map_err(|e| miette::miette!("{}", e))?;

        let mut items = result.items;

        // Sort cached entities if a sort function is provided
        if let Some(sort_fn) = config.cached_sort {
            sort_fn(&mut items, sort_field, sort_dir);
        }

        // Apply reverse and limit
        if args.reverse {
            items.reverse();
        }
        if let Some(limit) = args.limit {
            items.truncate(limit);
        }

        // Count only mode
        if args.count {
            println!("{}", items.len());
            return Ok(());
        }

        if items.is_empty() {
            println!("No {} found.", config.entity_name);
            return Ok(());
        }

        // Update short IDs for cached items (need to extract IDs differently)
        let mut short_ids = ShortIdIndex::load(project);
        // Note: for cached items, we get the ID from the row conversion
        // We'll update short IDs based on the rows
        let rows: Vec<TableRow> = items
            .iter()
            .map(|e| (config.cached_to_row)(e, &short_ids))
            .collect();
        short_ids.ensure_all(rows.iter().map(|r| r.full_id.clone()));
        super::commands::utils::save_short_ids(&mut short_ids, project);

        output_cached_entities(&items, config, &short_ids, args, format)
    }
}

/// Output empty list in appropriate format
fn output_empty_list(entity_name: &str, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("[]"),
        OutputFormat::Yaml => println!("[]"),
        _ => println!("No {} found.", entity_name),
    }
    Ok(())
}

/// Output full entities (for JSON/YAML or when full data needed)
pub fn output_full_entities<E, C, S>(
    items: &[E],
    config: &ListConfig<E, C, S>,
    short_ids: &ShortIdIndex,
    args: &CommonListArgs,
    format: OutputFormat,
) -> Result<()>
where
    E: Serialize + Clone,
{
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(items).map_err(|e| miette::miette!("{}", e))?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yml::to_string(items).map_err(|e| miette::miette!("{}", e))?;
            print!("{}", yaml);
        }
        OutputFormat::Id | OutputFormat::ShortId => {
            for item in items {
                let row = (config.entity_to_row)(item, short_ids);
                if format == OutputFormat::ShortId {
                    println!("{}", row.short_id);
                } else {
                    println!("{}", row.full_id);
                }
            }
        }
        _ => {
            // Table formats (table, tsv, csv, md)
            let rows: Vec<TableRow> = items
                .iter()
                .map(|e| (config.entity_to_row)(e, short_ids))
                .collect();

            let columns: Vec<&str> = args.columns.iter().map(|s| s.as_str()).collect();

            let table_config = TableConfig {
                wrap_width: args.wrap,
                show_summary: true,
            };
            let formatter =
                TableFormatter::new(config.columns, config.entity_name, config.prefix_str)
                    .with_config(table_config);
            formatter.output(rows, format, &columns);
        }
    }
    Ok(())
}

/// Output cached entities (fast path for table output)
pub fn output_cached_entities<E, C, S>(
    items: &[C],
    config: &ListConfig<E, C, S>,
    short_ids: &ShortIdIndex,
    args: &CommonListArgs,
    format: OutputFormat,
) -> Result<()>
where
    C: Clone,
{
    match format {
        OutputFormat::Id | OutputFormat::ShortId => {
            for item in items {
                let row = (config.cached_to_row)(item, short_ids);
                if format == OutputFormat::ShortId {
                    println!("{}", row.short_id);
                } else {
                    println!("{}", row.full_id);
                }
            }
        }
        _ => {
            // Table formats (table, tsv, csv, md)
            let rows: Vec<TableRow> = items
                .iter()
                .map(|e| (config.cached_to_row)(e, short_ids))
                .collect();

            let columns: Vec<&str> = args.columns.iter().map(|s| s.as_str()).collect();

            let table_config = TableConfig {
                wrap_width: args.wrap,
                show_summary: true,
            };
            let formatter =
                TableFormatter::new(config.columns, config.entity_name, config.prefix_str)
                    .with_config(table_config);
            formatter.output(rows, format, &columns);
        }
    }
    Ok(())
}

// =========================================================================
// Link Handling
// =========================================================================

use tdt_core::core::links::add_inferred_link;

/// Process --link flags and add inferred links to a newly created entity
pub fn process_link_flags(
    file_path: &std::path::Path,
    source_prefix: EntityPrefix,
    link_targets: &[String],
    short_ids: &ShortIdIndex,
) -> Vec<(String, String)> {
    let mut added_links = Vec::new();

    for link_target in link_targets {
        let resolved_target = short_ids
            .resolve(link_target)
            .unwrap_or_else(|| link_target.clone());

        if let Ok(target_entity_id) = EntityId::parse(&resolved_target) {
            match add_inferred_link(
                file_path,
                source_prefix,
                &resolved_target,
                target_entity_id.prefix(),
            ) {
                Ok(link_type) => {
                    added_links.push((link_type, resolved_target.clone()));
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to add link to {}: {}",
                        style("!").yellow(),
                        link_target,
                        e
                    );
                }
            }
        } else {
            eprintln!("{} Invalid entity ID: {}", style("!").yellow(), link_target);
        }
    }

    added_links
}
