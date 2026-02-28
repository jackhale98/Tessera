//! `tdt dsm` command - Design Structure Matrix for component interactions
//!
//! Generates a DSM showing relationships between components based on:
//! - Mates (physical interfaces via features)
//! - Tolerance stackups (dimensional chains linking components)
//! - Processes (shared manufacturing processes)
//! - Requirements (allocated to same requirement)

use console::style;
use miette::Result;
use std::collections::{HashMap, HashSet};

use crate::cli::helpers::format_short_id_str;
use crate::cli::GlobalOpts;
use tdt_core::core::cache::EntityCache;
use tdt_core::core::project::Project;

#[derive(clap::Args, Debug)]
pub struct DsmArgs {
    /// Assembly ID to scope the DSM (optional - defaults to all components)
    pub assembly: Option<String>,

    /// Relationship types to include (mate, tolerance, process, requirement, all)
    #[arg(long, short = 't', default_value = "all")]
    pub rel_type: String,

    /// Apply clustering optimization to group related components
    #[arg(long, short = 'c')]
    pub cluster: bool,

    /// Show full IDs instead of short IDs
    #[arg(long)]
    pub full_ids: bool,

    /// Show numeric dependency strength instead of relationship type symbols
    #[arg(long, short = 'w')]
    pub weighted: bool,

    /// Show coupling metrics (fan-in, fan-out, coupling coefficient)
    #[arg(long, short = 'm')]
    pub metrics: bool,

    /// Highlight and report dependency cycles
    #[arg(long)]
    pub cycles: bool,
}

/// Relationship between two components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    Mate,
    Tolerance,
    Process,
    Requirement,
}

impl RelationType {
    fn symbol(&self) -> &'static str {
        match self {
            RelationType::Mate => "M",
            RelationType::Tolerance => "T",
            RelationType::Process => "P",
            RelationType::Requirement => "R",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            RelationType::Mate => "Mate",
            RelationType::Tolerance => "Tolerance",
            RelationType::Process => "Process",
            RelationType::Requirement => "Requirement",
        }
    }
}

/// A cell in the DSM matrix
#[derive(Debug, Clone, Default)]
pub struct DsmCell {
    pub relationships: HashSet<RelationType>,
}

impl DsmCell {
    fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    fn symbol(&self) -> String {
        if self.relationships.is_empty() {
            return String::new();
        }
        let mut symbols: Vec<&str> = self.relationships.iter().map(|r| r.symbol()).collect();
        symbols.sort();
        symbols.join(",")
    }

    fn count(&self) -> usize {
        self.relationships.len()
    }
}

/// Component info for DSM
#[derive(Debug, Clone)]
pub struct DsmComponent {
    pub id: String,
    pub short_id: String,
    pub title: String,
    pub part_number: Option<String>,
}

/// The full DSM structure
pub struct Dsm {
    pub components: Vec<DsmComponent>,
    pub matrix: Vec<Vec<DsmCell>>,
    pub component_index: HashMap<String, usize>,
}

impl Dsm {
    fn new(components: Vec<DsmComponent>) -> Self {
        let n = components.len();
        let mut component_index = HashMap::new();
        for (i, cmp) in components.iter().enumerate() {
            component_index.insert(cmp.id.clone(), i);
        }

        let matrix = vec![vec![DsmCell::default(); n]; n];

        Self {
            components,
            matrix,
            component_index,
        }
    }

    fn add_relationship(&mut self, cmp1_id: &str, cmp2_id: &str, rel_type: RelationType) {
        if let (Some(&i), Some(&j)) = (
            self.component_index.get(cmp1_id),
            self.component_index.get(cmp2_id),
        ) {
            if i != j {
                // Symmetric - add to both cells
                self.matrix[i][j].relationships.insert(rel_type.clone());
                self.matrix[j][i].relationships.insert(rel_type);
            }
        }
    }

    /// Apply clustering to minimize off-diagonal distance
    #[allow(clippy::needless_range_loop)]
    fn cluster(&mut self) {
        let n = self.components.len();
        if n <= 2 {
            return;
        }

        // Calculate coupling strength for each component pair
        let mut coupling: Vec<Vec<usize>> = vec![vec![0; n]; n];
        for i in 0..n {
            for j in 0..n {
                coupling[i][j] = self.matrix[i][j].count();
            }
        }

        // Simple greedy clustering
        let mut ordered: Vec<usize> = Vec::with_capacity(n);
        let mut remaining: HashSet<usize> = (0..n).collect();

        // Find component with most total connections
        let start = (0..n)
            .max_by_key(|&i| coupling[i].iter().sum::<usize>())
            .unwrap_or(0);
        ordered.push(start);
        remaining.remove(&start);

        // Greedily add most connected to current cluster
        while !remaining.is_empty() {
            let mut best_next = *remaining.iter().next().unwrap();
            let mut best_score = 0usize;

            for &candidate in &remaining {
                let score: usize = ordered.iter().map(|&o| coupling[candidate][o]).sum();
                if score > best_score {
                    best_score = score;
                    best_next = candidate;
                }
            }

            ordered.push(best_next);
            remaining.remove(&best_next);
        }

        // Reorder components and matrix
        let old_components = self.components.clone();
        let old_matrix = self.matrix.clone();

        for (new_i, &old_i) in ordered.iter().enumerate() {
            self.components[new_i] = old_components[old_i].clone();
            for (new_j, &old_j) in ordered.iter().enumerate() {
                self.matrix[new_i][new_j] = old_matrix[old_i][old_j].clone();
            }
        }

        // Rebuild index
        self.component_index.clear();
        for (i, cmp) in self.components.iter().enumerate() {
            self.component_index.insert(cmp.id.clone(), i);
        }
    }

    /// Identify clusters (groups of tightly coupled components)
    fn identify_clusters(&self) -> Vec<Vec<usize>> {
        let n = self.components.len();
        if n == 0 {
            return vec![];
        }

        let mut parent: Vec<usize> = (0..n).collect();

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], i: usize, j: usize) {
            let pi = find(parent, i);
            let pj = find(parent, j);
            if pi != pj {
                parent[pi] = pj;
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                if !self.matrix[i][j].is_empty() {
                    union(&mut parent, i, j);
                }
            }
        }

        let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            clusters.entry(root).or_default().push(i);
        }

        clusters.into_values().collect()
    }

    /// Calculate coupling metrics for each component
    fn calculate_metrics(&self) -> Vec<CouplingMetrics> {
        let n = self.components.len();
        let mut metrics = Vec::with_capacity(n);

        for i in 0..n {
            let mut unique_connections = 0; // Number of distinct components connected to
            let mut total_rel_types = 0; // Total relationship type instances

            for j in 0..n {
                if i != j && !self.matrix[i][j].is_empty() {
                    unique_connections += 1;
                    total_rel_types += self.matrix[i][j].count();
                }
            }

            let max_possible = n - 1; // Maximum unique component connections
            let coupling_coefficient = if max_possible > 0 {
                (unique_connections as f64) / (max_possible as f64)
            } else {
                0.0
            };

            metrics.push(CouplingMetrics {
                component_idx: i,
                unique_connections,
                total_rel_types,
                coupling_coefficient,
                // Hub: connected to majority of other components
                is_hub: unique_connections > (n - 1) / 2,
            });
        }

        metrics
    }

    /// Identify change propagation groups (connected components with size > 1)
    ///
    /// In a symmetric DSM (physical interfaces), any connected group of components
    /// forms a change propagation path — a change to one component may affect all others
    /// in the group. Larger groups indicate higher change risk.
    fn detect_propagation_groups(&self) -> Vec<Vec<usize>> {
        let clusters = self.identify_clusters();
        clusters.into_iter().filter(|c| c.len() > 1).collect()
    }
}

/// Coupling metrics for a single component
#[derive(Debug)]
pub struct CouplingMetrics {
    pub component_idx: usize,
    pub unique_connections: usize,
    pub total_rel_types: usize,
    pub coupling_coefficient: f64,
    pub is_hub: bool, // Connected to majority of other components
}

pub fn run(args: DsmArgs, global: &GlobalOpts) -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let cache = EntityCache::open(&project)?;

    // Get components
    let components = if let Some(ref asm_id) = args.assembly {
        get_assembly_components(&cache, &project, asm_id)?
    } else {
        get_all_components(&cache)?
    };

    if components.is_empty() {
        if args.assembly.is_some() {
            println!("No components found in specified assembly");
        } else {
            println!("No components found in project");
        }
        return Ok(());
    }

    // Build DSM
    let mut dsm = Dsm::new(components);

    // Determine which relationship types to include
    let include_mate = args.rel_type == "all" || args.rel_type == "mate";
    let include_tolerance = args.rel_type == "all" || args.rel_type == "tolerance";
    let include_process = args.rel_type == "all" || args.rel_type == "process";
    let include_req = args.rel_type == "all" || args.rel_type == "requirement";

    // Find relationships
    if include_mate {
        add_mate_relationships(&cache, &mut dsm)?;
    }

    if include_tolerance {
        add_tolerance_relationships(&cache, &mut dsm)?;
    }

    if include_process {
        add_process_relationships(&cache, &mut dsm)?;
    }

    if include_req {
        add_requirement_relationships(&cache, &mut dsm)?;
    }

    // Apply clustering if requested
    if args.cluster {
        dsm.cluster();
    }

    // Output
    let display_opts = DisplayOptions {
        full_ids: args.full_ids,
        clustered: args.cluster,
        weighted: args.weighted,
        show_metrics: args.metrics,
        show_cycles: args.cycles,
    };

    use crate::cli::OutputFormat;
    match global.output {
        OutputFormat::Csv => output_csv(&dsm, &display_opts),
        OutputFormat::Dot => output_dot(&dsm, &display_opts),
        OutputFormat::Json => output_json(&dsm, &display_opts),
        _ => output_table(&dsm, &display_opts),
    }

    Ok(())
}

/// Display options for DSM output
#[derive(Debug, Clone)]
struct DisplayOptions {
    full_ids: bool,
    clustered: bool,
    weighted: bool,
    show_metrics: bool,
    show_cycles: bool,
}

fn get_assembly_components(
    cache: &EntityCache,
    _project: &Project,
    asm_id: &str,
) -> Result<Vec<DsmComponent>> {
    // Resolve assembly ID
    let resolved_id = cache
        .resolve_short_id(asm_id)
        .unwrap_or_else(|| asm_id.to_string());

    // Use cache's recursive BOM traversal (handles subassemblies and cycles)
    let component_ids = cache.get_bom_components(&resolved_id);

    // Build a lookup of all components
    let all_components: HashMap<String, _> = cache
        .list_components(None, None, None, None, None, None)
        .into_iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    let mut components = Vec::new();
    for cmp_id in component_ids {
        if let Some(cmp) = all_components.get(&cmp_id) {
            let short_id = cache
                .get_short_id(&cmp.id)
                .unwrap_or_else(|| format_short_id_str(&cmp.id));
            components.push(DsmComponent {
                id: cmp.id.clone(),
                short_id,
                title: cmp.title.clone(),
                part_number: cmp.part_number.clone(),
            });
        }
    }

    Ok(components)
}

fn get_all_components(cache: &EntityCache) -> Result<Vec<DsmComponent>> {
    let cmp_list = cache.list_components(None, None, None, None, None, None);

    let mut components = Vec::new();
    for cmp in cmp_list {
        let short_id = cache
            .get_short_id(&cmp.id)
            .unwrap_or_else(|| format_short_id_str(&cmp.id));
        components.push(DsmComponent {
            id: cmp.id,
            short_id,
            title: cmp.title,
            part_number: cmp.part_number,
        });
    }

    Ok(components)
}

fn add_mate_relationships(cache: &EntityCache, dsm: &mut Dsm) -> Result<()> {
    // Build feature-to-component lookup from cache
    let feature_to_component = cache.get_all_features();

    // Load mates from cache
    let mates = cache.list_entities(&tdt_core::core::cache::EntityFilter {
        prefix: Some(tdt_core::core::identity::EntityPrefix::Mate),
        ..Default::default()
    });

    for mate_entity in mates {
        // Use cached links: MATE → FEAT via "feature_a" and "feature_b" link types
        let feat_a_ids = cache.get_links_from_of_type(&mate_entity.id, "feature_a");
        let feat_b_ids = cache.get_links_from_of_type(&mate_entity.id, "feature_b");

        let comp_a = feat_a_ids
            .first()
            .and_then(|feat_id| feature_to_component.get(feat_id))
            .map(|f| f.component_id.clone());
        let comp_b = feat_b_ids
            .first()
            .and_then(|feat_id| feature_to_component.get(feat_id))
            .map(|f| f.component_id.clone());

        if let (Some(cmp1), Some(cmp2)) = (comp_a, comp_b) {
            if cmp1 != cmp2 {
                dsm.add_relationship(&cmp1, &cmp2, RelationType::Mate);
            }
        }
    }

    Ok(())
}

fn add_tolerance_relationships(cache: &EntityCache, dsm: &mut Dsm) -> Result<()> {
    // Build feature-to-component lookup from cache
    let feature_to_component = cache.get_all_features();

    // Load tolerance stackups from cache
    let stackups = cache.list_entities(&tdt_core::core::cache::EntityFilter {
        prefix: Some(tdt_core::core::identity::EntityPrefix::Tol),
        ..Default::default()
    });

    for stackup_entity in stackups {
        // Use cached links: TOL → FEAT via "contributor[N]" link types
        let all_links = cache.get_links_from(&stackup_entity.id);

        // Collect unique components from contributor feature links
        let mut stackup_components: Vec<String> = Vec::new();
        for link in &all_links {
            if link.link_type.starts_with("contributor[") {
                if let Some(feat) = feature_to_component.get(&link.target_id) {
                    if !stackup_components.contains(&feat.component_id) {
                        stackup_components.push(feat.component_id.clone());
                    }
                }
            }
        }

        // Create relationships for all component pairs in the stackup
        for i in 0..stackup_components.len() {
            for j in (i + 1)..stackup_components.len() {
                dsm.add_relationship(
                    &stackup_components[i],
                    &stackup_components[j],
                    RelationType::Tolerance,
                );
            }
        }
    }

    Ok(())
}

fn add_process_relationships(cache: &EntityCache, dsm: &mut Dsm) -> Result<()> {
    // Build a map of process -> components
    let mut process_components: HashMap<String, HashSet<String>> = HashMap::new();

    let processes = cache.list_processes(None, None, None, None, None, None);

    for proc in processes {
        // Get components linked to this process
        let links = cache.get_links_from(&proc.id);
        for link in links {
            if link.target_id.starts_with("CMP-") {
                process_components
                    .entry(proc.id.clone())
                    .or_default()
                    .insert(link.target_id);
            }
        }

        // Also check reverse links
        let reverse_links = cache.get_links_to(&proc.id);
        for link in reverse_links {
            if link.source_id.starts_with("CMP-") {
                process_components
                    .entry(proc.id.clone())
                    .or_default()
                    .insert(link.source_id);
            }
        }
    }

    // Components sharing a process are related
    for (_proc_id, cmp_ids) in process_components {
        let cmp_list: Vec<_> = cmp_ids.into_iter().collect();
        for i in 0..cmp_list.len() {
            for j in (i + 1)..cmp_list.len() {
                dsm.add_relationship(&cmp_list[i], &cmp_list[j], RelationType::Process);
            }
        }
    }

    Ok(())
}

fn add_requirement_relationships(cache: &EntityCache, dsm: &mut Dsm) -> Result<()> {
    // Build a map of requirement -> components
    let mut req_components: HashMap<String, HashSet<String>> = HashMap::new();

    let reqs = cache.list_requirements(None, None, None, None, None, None, None);

    for req in reqs {
        let links = cache.get_links_from(&req.id);
        for link in links {
            if link.target_id.starts_with("CMP-") {
                req_components
                    .entry(req.id.clone())
                    .or_default()
                    .insert(link.target_id);
            }
        }

        let reverse_links = cache.get_links_to(&req.id);
        for link in reverse_links {
            if link.source_id.starts_with("CMP-") {
                req_components
                    .entry(req.id.clone())
                    .or_default()
                    .insert(link.source_id);
            }
        }
    }

    for (_req_id, cmp_ids) in req_components {
        let cmp_list: Vec<_> = cmp_ids.into_iter().collect();
        for i in 0..cmp_list.len() {
            for j in (i + 1)..cmp_list.len() {
                dsm.add_relationship(&cmp_list[i], &cmp_list[j], RelationType::Requirement);
            }
        }
    }

    Ok(())
}

fn output_table(dsm: &Dsm, opts: &DisplayOptions) {
    let n = dsm.components.len();
    if n == 0 {
        return;
    }

    // Header
    let title = if opts.clustered {
        "Design Structure Matrix (Clustered)"
    } else {
        "Design Structure Matrix"
    };
    println!("{}", style(title).bold().cyan());
    println!();

    // Calculate row label width (left column)
    let id_width = if opts.full_ids {
        dsm.components
            .iter()
            .map(|c| format_short_id_str(&c.id).len())
            .max()
            .unwrap_or(8)
    } else {
        dsm.components
            .iter()
            .map(|c| c.short_id.len())
            .max()
            .unwrap_or(6)
            .max(6)
    };

    // Calculate column width based on longest label + padding
    let max_col_label = if opts.full_ids {
        dsm.components
            .iter()
            .map(|c| format_short_id_str(&c.id).len())
            .max()
            .unwrap_or(5)
    } else {
        dsm.components
            .iter()
            .map(|c| c.short_id.len())
            .max()
            .unwrap_or(5)
    };
    let cell_width = (max_col_label + 2).max(7); // At least 7 chars per column

    // Detect cycles if requested
    let cycles = if opts.show_cycles {
        dsm.detect_propagation_groups()
    } else {
        vec![]
    };

    // Build set of component indices that are in cycles
    let in_cycle: HashSet<usize> = cycles.iter().flatten().copied().collect();

    // Print column headers
    print!("{:width$} ", "", width = id_width);
    for cmp in &dsm.components {
        let label = if opts.full_ids {
            format_short_id_str(&cmp.id)
        } else {
            cmp.short_id.clone()
        };
        print!("{:^width$}", label, width = cell_width);
    }
    println!();

    // Print separator
    print!("{:-<width$} ", "", width = id_width);
    for _ in &dsm.components {
        print!("{:-<width$}", "", width = cell_width);
    }
    println!();

    // Print rows
    for (i, cmp) in dsm.components.iter().enumerate() {
        let label = if opts.full_ids {
            format_short_id_str(&cmp.id)
        } else {
            cmp.short_id.clone()
        };

        // Highlight row label if in cycle
        if opts.show_cycles && in_cycle.contains(&i) {
            print!("{:<width$} ", style(label).red().bold(), width = id_width);
        } else {
            print!("{:<width$} ", label, width = id_width);
        }

        for (j, _) in dsm.components.iter().enumerate() {
            if i == j {
                print!("{:^width$}", style("■").dim(), width = cell_width);
            } else {
                let cell = &dsm.matrix[i][j];
                if cell.is_empty() {
                    print!("{:^width$}", "·", width = cell_width);
                } else if opts.weighted {
                    // Weighted mode: show numeric count
                    let count = cell.count();
                    let styled = match count {
                        1 => style(count.to_string()).dim(),
                        2 => style(count.to_string()).yellow(),
                        _ => style(count.to_string()).red().bold(),
                    };
                    print!("{:^width$}", styled, width = cell_width);
                } else {
                    // Symbol mode
                    let symbol = cell.symbol();
                    let styled = match symbol.as_str() {
                        "M" => style(symbol).green(),
                        "T" => style(symbol).magenta(),
                        "P" => style(symbol).blue(),
                        "R" => style(symbol).yellow(),
                        _ => style(symbol).white(),
                    };
                    print!("{:^width$}", styled, width = cell_width);
                }
            }
        }
        println!();
    }

    // Legend
    println!();
    if opts.weighted {
        println!(
            "{}: Numbers show dependency strength (count of relationship types)",
            style("Legend").bold()
        );
    } else {
        println!(
            "{}: {} = Mate {} = Tolerance {} = Process {} = Requirement",
            style("Legend").bold(),
            style("M").green(),
            style("T").magenta(),
            style("P").blue(),
            style("R").yellow()
        );
    }

    // Statistics
    let mut mate_count = 0;
    let mut tolerance_count = 0;
    let mut process_count = 0;
    let mut req_count = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            let cell = &dsm.matrix[i][j];
            if cell.relationships.contains(&RelationType::Mate) {
                mate_count += 1;
            }
            if cell.relationships.contains(&RelationType::Tolerance) {
                tolerance_count += 1;
            }
            if cell.relationships.contains(&RelationType::Process) {
                process_count += 1;
            }
            if cell.relationships.contains(&RelationType::Requirement) {
                req_count += 1;
            }
        }
    }

    println!();
    println!(
        "{}: {} components, {} mate, {} tolerance, {} process, {} requirement",
        style("Summary").bold(),
        n,
        mate_count,
        tolerance_count,
        process_count,
        req_count
    );

    // Clusters (if enabled)
    if opts.clustered {
        let clusters = dsm.identify_clusters();
        if clusters.len() > 1 {
            println!();
            println!("{}", style("Identified Clusters:").bold());
            for (i, cluster) in clusters.iter().enumerate() {
                let names: Vec<_> = cluster
                    .iter()
                    .map(|&idx| dsm.components[idx].short_id.clone())
                    .collect();
                println!("  Cluster {}: {}", i + 1, names.join(", "));
            }
        }
    }

    // Change propagation groups (if enabled)
    if opts.show_cycles {
        println!();
        if cycles.is_empty() {
            println!(
                "{}: All components are independent",
                style("Change Propagation").bold()
            );
        } else {
            println!(
                "{}: {} group(s) detected",
                style("Change Propagation").bold().red(),
                cycles.len()
            );
            for (i, group) in cycles.iter().enumerate() {
                let names: Vec<_> = group
                    .iter()
                    .map(|&idx| dsm.components[idx].short_id.clone())
                    .collect();
                println!(
                    "  {} {}: {} ({} components)",
                    style("Group").red(),
                    i + 1,
                    names.join(", "),
                    names.len()
                );
            }
            println!();
            println!(
                "  {}",
                style("A change to any component in a group may affect all others in that group")
                    .dim()
            );
        }
    }

    // Metrics (if enabled)
    if opts.show_metrics {
        output_metrics(dsm);
    }
}

fn output_metrics(dsm: &Dsm) {
    let metrics = dsm.calculate_metrics();

    println!();
    println!("{}", style("Coupling Metrics").bold().cyan());
    println!();

    // Header
    println!(
        "  {:<12} {:>12} {:>12} {:>12}",
        "Component", "Connections", "Rel Types", "Coupling %"
    );
    println!("  {:-<12} {:->12} {:->12} {:->12}", "", "", "", "");

    // Metrics for each component
    for m in &metrics {
        let cmp = &dsm.components[m.component_idx];
        let coupling_pct = format!("{:.1}%", m.coupling_coefficient * 100.0);

        let hub_marker = if m.is_hub {
            style(" hub").yellow().to_string()
        } else {
            String::new()
        };

        println!(
            "  {:<12} {:>12} {:>12} {:>12}{}",
            cmp.short_id, m.unique_connections, m.total_rel_types, coupling_pct, hub_marker
        );
    }

    // Summary stats
    // Each unique connection is counted in both components, so divide by 2
    let total_unique: usize = metrics.iter().map(|m| m.unique_connections).sum::<usize>() / 2;
    let avg_coupling: f64 =
        metrics.iter().map(|m| m.coupling_coefficient).sum::<f64>() / metrics.len() as f64;
    let hubs: Vec<_> = metrics
        .iter()
        .filter(|m| m.is_hub)
        .map(|m| dsm.components[m.component_idx].short_id.clone())
        .collect();

    println!();
    println!(
        "  Total unique connections: {} | Avg coupling: {:.1}%",
        total_unique,
        avg_coupling * 100.0
    );
    if !hubs.is_empty() {
        println!(
            "  {} {} (connected to majority of components)",
            style("Hubs:").yellow(),
            hubs.join(", ")
        );
    }
}

fn output_csv(dsm: &Dsm, opts: &DisplayOptions) {
    let n = dsm.components.len();
    if n == 0 {
        return;
    }

    // Header row
    print!("Component");
    for cmp in &dsm.components {
        let label = if opts.full_ids {
            &cmp.id
        } else {
            &cmp.short_id
        };
        print!(",{}", label);
    }
    println!();

    // Data rows
    for (i, cmp) in dsm.components.iter().enumerate() {
        let label = if opts.full_ids {
            &cmp.id
        } else {
            &cmp.short_id
        };
        print!("{}", label);

        for (j, _) in dsm.components.iter().enumerate() {
            if i == j {
                print!(",X");
            } else {
                let cell = &dsm.matrix[i][j];
                if cell.is_empty() {
                    print!(",");
                } else if opts.weighted {
                    print!(",{}", cell.count());
                } else {
                    print!(",{}", cell.symbol());
                }
            }
        }
        println!();
    }
}

fn output_dot(dsm: &Dsm, opts: &DisplayOptions) {
    println!("graph DSM {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box];");
    println!();

    // Detect cycles for highlighting
    let cycles = if opts.show_cycles {
        dsm.detect_propagation_groups()
    } else {
        vec![]
    };
    let in_cycle: HashSet<usize> = cycles.iter().flatten().copied().collect();

    for (i, cmp) in dsm.components.iter().enumerate() {
        let label = format!("{}\\n{}", cmp.short_id, cmp.title);
        let color = if opts.show_cycles && in_cycle.contains(&i) {
            " fillcolor=lightcoral style=filled"
        } else {
            ""
        };
        println!("  \"{}\" [label=\"{}\"{}];", cmp.id, label, color);
    }
    println!();

    let n = dsm.components.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let cell = &dsm.matrix[i][j];
            if !cell.is_empty() {
                let color = if cell.relationships.contains(&RelationType::Mate) {
                    "green"
                } else if cell.relationships.contains(&RelationType::Tolerance) {
                    "magenta"
                } else if cell.relationships.contains(&RelationType::Process) {
                    "blue"
                } else {
                    "orange"
                };
                let label = if opts.weighted {
                    cell.count().to_string()
                } else {
                    cell.symbol()
                };
                let penwidth = if cell.count() > 1 { " penwidth=2" } else { "" };
                println!(
                    "  \"{}\" -- \"{}\" [label=\"{}\" color={}{}];",
                    dsm.components[i].id, dsm.components[j].id, label, color, penwidth
                );
            }
        }
    }

    println!("}}");
}

fn output_json(dsm: &Dsm, opts: &DisplayOptions) {
    let mut components_json: Vec<serde_json::Value> = Vec::new();

    // Calculate metrics if requested
    let metrics = if opts.show_metrics {
        Some(dsm.calculate_metrics())
    } else {
        None
    };

    for (i, cmp) in dsm.components.iter().enumerate() {
        let mut cmp_json = serde_json::json!({
            "id": cmp.id,
            "short_id": cmp.short_id,
            "title": cmp.title,
            "part_number": cmp.part_number,
        });

        if let Some(ref m) = metrics {
            cmp_json["metrics"] = serde_json::json!({
                "unique_connections": m[i].unique_connections,
                "total_rel_types": m[i].total_rel_types,
                "coupling_coefficient": m[i].coupling_coefficient,
                "is_hub": m[i].is_hub,
            });
        }

        components_json.push(cmp_json);
    }

    let n = dsm.components.len();
    let mut relationships: Vec<serde_json::Value> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let cell = &dsm.matrix[i][j];
            if !cell.is_empty() {
                let types: Vec<&str> = cell.relationships.iter().map(|r| r.name()).collect();
                relationships.push(serde_json::json!({
                    "source": dsm.components[i].id,
                    "target": dsm.components[j].id,
                    "types": types,
                    "weight": cell.count(),
                }));
            }
        }
    }

    // Build output
    let mut output = serde_json::json!({
        "components": components_json,
        "relationships": relationships,
        "matrix_size": n,
    });

    // Add change propagation groups if requested
    if opts.show_cycles {
        let groups = dsm.detect_propagation_groups();
        let groups_json: Vec<Vec<String>> = groups
            .iter()
            .map(|g| {
                g.iter()
                    .map(|&idx| dsm.components[idx].short_id.clone())
                    .collect()
            })
            .collect();
        output["propagation_groups"] = serde_json::json!(groups_json);
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );
}
