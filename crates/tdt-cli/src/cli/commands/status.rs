//! `tdt status` command - Project status dashboard

use console::style;
use miette::Result;
use std::collections::HashMap;

use crate::cli::{GlobalOpts, OutputFormat};
use tdt_core::core::cache::EntityCache;
use tdt_core::core::project::Project;
use tdt_core::services::{
    AssemblyService, CapaService, ComponentService, FeatureService, MateService, NcrService,
    RequirementService, ResultService, RiskService, StackupService, TestService,
};

#[derive(clap::Args, Debug)]
pub struct StatusArgs {
    /// Show only specific section (requirements, risks, tests, quality, bom)
    #[arg(long)]
    pub section: Option<String>,

    /// Show detailed breakdown
    #[arg(long)]
    pub detailed: bool,
}

pub fn run(_args: StatusArgs, global: &GlobalOpts) -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;
    let cache = EntityCache::open(&project).map_err(|e| miette::miette!("{}", e))?;

    // Collect metrics using services (fast, uses cache)
    let req_metrics = collect_requirement_metrics(&project, &cache);
    let risk_metrics = collect_risk_metrics(&project, &cache);
    let test_metrics = collect_test_metrics(&project, &cache);
    let quality_metrics = collect_quality_metrics(&project, &cache);
    let bom_metrics = collect_bom_metrics(&project, &cache);
    let tol_metrics = collect_tolerance_metrics(&project, &cache);

    match global.output {
        OutputFormat::Json => {
            let status = serde_json::json!({
                "requirements": req_metrics,
                "risks": risk_metrics,
                "tests": test_metrics,
                "quality": quality_metrics,
                "bom": bom_metrics,
                "tolerances": tol_metrics,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&status).unwrap_or_default()
            );
        }
        _ => {
            // Human-readable dashboard
            let width = 68;

            println!("{}", style("Tessera Project Status").bold().underlined());
            println!("{}", "═".repeat(width));
            println!();

            // Requirements and Risks side by side
            print_two_columns(
                "REQUIREMENTS",
                &format_requirement_metrics(&req_metrics),
                "RISKS",
                &format_risk_metrics(&risk_metrics),
                width,
            );

            println!();

            // Tests and Quality side by side
            print_two_columns(
                "TESTS",
                &format_test_metrics(&test_metrics),
                "QUALITY",
                &format_quality_metrics(&quality_metrics),
                width,
            );

            println!();

            // BOM and Tolerances side by side
            print_two_columns(
                "BILL OF MATERIALS",
                &format_bom_metrics(&bom_metrics),
                "TOLERANCE ANALYSIS",
                &format_tolerance_metrics(&tol_metrics),
                width,
            );

            println!();
            println!("{}", "═".repeat(width));

            // Overall health indicator
            let health = calculate_health(
                &req_metrics,
                &risk_metrics,
                &test_metrics,
                &quality_metrics,
                &tol_metrics,
            );
            let health_style = match health.as_str() {
                "Healthy" => style(health.clone()).green().bold(),
                "Warning" => style(health.clone()).yellow().bold(),
                "Critical" => style(health.clone()).red().bold(),
                _ => style(health.clone()).dim(),
            };
            println!("Project Health: {}", health_style);
        }
    }

    Ok(())
}

#[derive(serde::Serialize, Default)]
struct RequirementMetrics {
    total: usize,
    by_status: HashMap<String, usize>,
    by_type: HashMap<String, usize>,
    verified: usize,
    unverified: usize,
    coverage_pct: f64,
}

#[derive(serde::Serialize, Default)]
struct RiskMetrics {
    total: usize,
    by_level: HashMap<String, usize>,
    avg_rpn: f64,
    max_rpn: u16,
    unmitigated: usize,
}

#[derive(serde::Serialize, Default)]
struct TestMetrics {
    protocols: usize,
    executed: usize,
    pending: usize,
    pass_count: usize,
    fail_count: usize,
    pass_rate: f64,
}

#[derive(serde::Serialize, Default)]
struct QualityMetrics {
    open_ncrs: usize,
    open_capas: usize,
    overdue: usize,
    ncr_by_severity: HashMap<String, usize>,
}

#[derive(serde::Serialize, Default)]
struct BomMetrics {
    components: usize,
    assemblies: usize,
    make_parts: usize,
    buy_parts: usize,
    single_source: usize,
    with_quotes: usize,
}

#[derive(serde::Serialize, Default)]
struct ToleranceMetrics {
    features: usize,
    mates: usize,
    mates_with_analysis: usize,
    mates_type_mismatch: usize,
    stackups: usize,
    stackups_with_analysis: usize,
    stackups_pass: usize,
    stackups_marginal: usize,
    stackups_fail: usize,
    contributors_linked: usize,
    contributors_unlinked: usize,
}

fn collect_requirement_metrics(project: &Project, cache: &EntityCache) -> RequirementMetrics {
    let service = RequirementService::new(project, cache);
    let stats = match service.stats() {
        Ok(s) => s,
        Err(_) => return RequirementMetrics::default(),
    };

    let verified = stats.total - stats.unverified;
    let coverage_pct = if stats.total > 0 {
        (verified as f64 / stats.total as f64) * 100.0
    } else {
        0.0
    };

    let mut by_status = HashMap::new();
    by_status.insert("draft".to_string(), stats.by_status.draft);
    by_status.insert("review".to_string(), stats.by_status.review);
    by_status.insert("approved".to_string(), stats.by_status.approved);
    by_status.insert("released".to_string(), stats.by_status.released);
    by_status.insert("obsolete".to_string(), stats.by_status.obsolete);

    let mut by_type = HashMap::new();
    by_type.insert("input".to_string(), stats.inputs);
    by_type.insert("output".to_string(), stats.outputs);

    RequirementMetrics {
        total: stats.total,
        by_status,
        by_type,
        verified,
        unverified: stats.unverified,
        coverage_pct,
    }
}

fn collect_risk_metrics(project: &Project, cache: &EntityCache) -> RiskMetrics {
    let service = RiskService::new(project, cache);
    let stats = match service.stats() {
        Ok(s) => s,
        Err(_) => return RiskMetrics::default(),
    };

    let mut by_level = HashMap::new();
    by_level.insert("critical".to_string(), stats.by_level.critical);
    by_level.insert("high".to_string(), stats.by_level.high);
    by_level.insert("medium".to_string(), stats.by_level.medium);
    by_level.insert("low".to_string(), stats.by_level.low);

    RiskMetrics {
        total: stats.total,
        by_level,
        avg_rpn: stats.rpn_stats.avg,
        max_rpn: stats.rpn_stats.max,
        unmitigated: stats.unmitigated,
    }
}

fn collect_test_metrics(project: &Project, cache: &EntityCache) -> TestMetrics {
    let test_service = TestService::new(project, cache);
    let result_service = ResultService::new(project, cache);

    let test_stats = test_service.stats().unwrap_or_default();
    let result_stats = result_service.stats().unwrap_or_default();

    let pass_count = result_stats.by_verdict.pass;
    let fail_count = result_stats.by_verdict.fail;
    let total_judged = pass_count + fail_count;
    let pass_rate = if total_judged > 0 {
        (pass_count as f64 / total_judged as f64) * 100.0
    } else {
        result_stats.pass_rate // Use service-computed pass rate as fallback
    };

    // Use total results as proxy for executed tests (not perfect but reasonable)
    let executed = result_stats.total;

    TestMetrics {
        protocols: test_stats.total,
        executed,
        pending: test_stats.total.saturating_sub(executed),
        pass_count,
        fail_count,
        pass_rate,
    }
}

fn collect_quality_metrics(project: &Project, cache: &EntityCache) -> QualityMetrics {
    let ncr_service = NcrService::new(project, cache);
    let capa_service = CapaService::new(project, cache);

    let ncr_stats = ncr_service.stats().unwrap_or_default();
    let capa_stats = capa_service.stats().unwrap_or_default();

    let mut ncr_by_severity = HashMap::new();
    ncr_by_severity.insert("critical".to_string(), ncr_stats.by_severity.critical);
    ncr_by_severity.insert("major".to_string(), ncr_stats.by_severity.major);
    ncr_by_severity.insert("minor".to_string(), ncr_stats.by_severity.minor);

    // Count open CAPAs (not closed)
    let open_capas = capa_stats.total - capa_stats.by_status.closed;

    QualityMetrics {
        open_ncrs: ncr_stats.open,
        open_capas,
        overdue: capa_stats.overdue_count,
        ncr_by_severity,
    }
}

fn collect_bom_metrics(project: &Project, cache: &EntityCache) -> BomMetrics {
    let cmp_service = ComponentService::new(project, cache);
    let asm_service = AssemblyService::new(project, cache);

    let cmp_stats = cmp_service.stats().unwrap_or_default();
    let asm_stats = asm_service.stats().unwrap_or_default();

    // Count quotes from cache
    let quotes = cache.list_quotes(None, None, None, None, None, None, None);
    let with_quotes = quotes.iter().filter(|q| q.component_id.is_some()).count();

    // Count single-source components from cache
    // Group quotes by component_id and count unique suppliers per component
    let mut component_suppliers: HashMap<String, std::collections::HashSet<String>> =
        HashMap::new();
    for quote in &quotes {
        if let (Some(cmp_id), Some(sup_id)) = (&quote.component_id, &quote.supplier_id) {
            component_suppliers
                .entry(cmp_id.clone())
                .or_default()
                .insert(sup_id.clone());
        }
    }
    let single_source = component_suppliers
        .values()
        .filter(|suppliers| suppliers.len() == 1)
        .count();

    BomMetrics {
        components: cmp_stats.total,
        assemblies: asm_stats.total,
        make_parts: cmp_stats.make_count,
        buy_parts: cmp_stats.buy_count,
        single_source,
        with_quotes,
    }
}

fn collect_tolerance_metrics(project: &Project, cache: &EntityCache) -> ToleranceMetrics {
    let feat_service = FeatureService::new(project, cache);
    let mate_service = MateService::new(project, cache);
    let stackup_service = StackupService::new(project, cache);

    let feat_stats = feat_service.stats().unwrap_or_default();
    let mate_stats = mate_service.stats().unwrap_or_default();
    let stackup_stats = stackup_service.stats().unwrap_or_default();

    // Calculate mate type mismatches: analyzed mates that don't match intent
    let mates_type_mismatch = if mate_stats.analyzed_count > 0 {
        mate_stats.analyzed_count - mate_stats.matches_intent
    } else {
        0
    };

    ToleranceMetrics {
        features: feat_stats.total,
        mates: mate_stats.total,
        mates_with_analysis: mate_stats.analyzed_count,
        mates_type_mismatch,
        stackups: stackup_stats.total,
        stackups_with_analysis: stackup_stats.analyzed_count,
        stackups_pass: stackup_stats.by_result.pass,
        stackups_marginal: stackup_stats.by_result.marginal,
        stackups_fail: stackup_stats.by_result.fail,
        contributors_linked: 0,   // Not tracked in service stats yet
        contributors_unlinked: 0, // Not tracked in service stats yet
    }
}

fn format_requirement_metrics(m: &RequirementMetrics) -> Vec<String> {
    vec![
        format!("Total:      {}", m.total),
        format!("Verified:   {} ({:.0}%)", m.verified, m.coverage_pct),
        format!("Unverified: {}", m.unverified),
        format!("Draft:      {}", m.by_status.get("draft").unwrap_or(&0)),
        format!("Approved:   {}", m.by_status.get("approved").unwrap_or(&0)),
    ]
}

fn format_risk_metrics(m: &RiskMetrics) -> Vec<String> {
    let mut lines = vec![format!("Total:      {}", m.total)];

    let critical = *m.by_level.get("critical").unwrap_or(&0);
    let high = *m.by_level.get("high").unwrap_or(&0);

    if critical > 0 {
        lines.push(format!("Critical:   {} {}", critical, style("⚠").red()));
    }
    if high > 0 {
        lines.push(format!("High:       {}", high));
    }
    lines.push(format!(
        "Medium:     {}",
        m.by_level.get("medium").unwrap_or(&0)
    ));
    lines.push(format!("Avg RPN:    {:.0}", m.avg_rpn));

    lines
}

fn format_test_metrics(m: &TestMetrics) -> Vec<String> {
    vec![
        format!("Protocols:  {}", m.protocols),
        format!("Executed:   {}", m.executed),
        format!("Pending:    {}", m.pending),
        format!("Pass Rate:  {:.0}%", m.pass_rate),
        format!("Failures:   {}", m.fail_count),
    ]
}

fn format_quality_metrics(m: &QualityMetrics) -> Vec<String> {
    let mut lines = vec![
        format!("Open NCRs:  {}", m.open_ncrs),
        format!("Open CAPAs: {}", m.open_capas),
    ];

    if m.overdue > 0 {
        lines.push(format!("Overdue:    {} {}", m.overdue, style("⚠").red()));
    }

    lines
}

fn format_bom_metrics(m: &BomMetrics) -> Vec<String> {
    let mut lines = vec![
        format!(
            "Components: {}  (Make: {}, Buy: {})",
            m.components, m.make_parts, m.buy_parts
        ),
        format!("Assemblies: {}", m.assemblies),
        format!("With Quotes: {}", m.with_quotes),
    ];

    if m.single_source > 0 {
        lines.push(format!(
            "Single-source: {} {}",
            m.single_source,
            style("⚠").yellow()
        ));
    }

    lines
}

fn format_tolerance_metrics(m: &ToleranceMetrics) -> Vec<String> {
    let mut lines = Vec::new();

    // Features
    lines.push(format!("Features:   {}", m.features));

    // Mates
    if m.mates > 0 {
        let mates_unanalyzed = m.mates - m.mates_with_analysis;
        if m.mates_type_mismatch > 0 {
            // Show type mismatch warning (fit doesn't match intent)
            lines.push(format!(
                "Mates:      {} ({} fit mismatch) {}",
                m.mates,
                m.mates_type_mismatch,
                style("⚠").red()
            ));
        } else if mates_unanalyzed > 0 {
            lines.push(format!(
                "Mates:      {} ({} unanalyzed)",
                m.mates, mates_unanalyzed
            ));
        } else {
            lines.push(format!("Mates:      {}", m.mates));
        }
    } else {
        lines.push(format!("Mates:      {}", m.mates));
    }

    // Stackups with pass/fail breakdown
    if m.stackups > 0 {
        lines.push(format!("Stackups:   {}", m.stackups));

        if m.stackups_with_analysis > 0 {
            if m.stackups_fail > 0 {
                lines.push(format!(
                    "  Failing:  {} {}",
                    m.stackups_fail,
                    style("⚠").red()
                ));
            }
            if m.stackups_marginal > 0 {
                lines.push(format!(
                    "  Marginal: {} {}",
                    m.stackups_marginal,
                    style("⚠").yellow()
                ));
            }
            lines.push(format!("  Passing:  {}", m.stackups_pass));
        }

        let unanalyzed = m.stackups - m.stackups_with_analysis;
        if unanalyzed > 0 {
            lines.push(format!("  Unanalyzed: {}", unanalyzed));
        }
    } else {
        lines.push(format!("Stackups:   {}", m.stackups));
    }

    // Contributors linkage
    let total_contributors = m.contributors_linked + m.contributors_unlinked;
    if total_contributors > 0 {
        let linked_pct = (m.contributors_linked as f64 / total_contributors as f64) * 100.0;
        if m.contributors_unlinked > 0 {
            lines.push(format!(
                "Contributors: {} linked ({:.0}%)",
                m.contributors_linked, linked_pct
            ));
        }
    }

    lines
}

fn print_two_columns(
    title1: &str,
    lines1: &[String],
    title2: &str,
    lines2: &[String],
    _width: usize,
) {
    let col_width = 34;

    println!(
        "{:<col_width$} {}",
        style(title1).bold(),
        style(title2).bold()
    );
    println!("{:-<col_width$} {:-<col_width$}", "", "");

    let max_lines = lines1.len().max(lines2.len());

    for i in 0..max_lines {
        let l1 = lines1.get(i).map(|s| s.as_str()).unwrap_or("");
        let l2 = lines2.get(i).map(|s| s.as_str()).unwrap_or("");
        println!("  {:<32} {}", l1, l2);
    }
}

fn calculate_health(
    req: &RequirementMetrics,
    risk: &RiskMetrics,
    test: &TestMetrics,
    quality: &QualityMetrics,
    tol: &ToleranceMetrics,
) -> String {
    let mut score = 100i32;

    // Requirements coverage
    if req.coverage_pct < 50.0 {
        score -= 20;
    } else if req.coverage_pct < 80.0 {
        score -= 10;
    }

    // Critical risks
    let critical = *risk.by_level.get("critical").unwrap_or(&0);
    if critical > 0 {
        score -= 15 * critical as i32;
    }

    // Unmitigated risks
    if risk.unmitigated > 5 {
        score -= 10;
    }

    // Test failures
    if test.fail_count > 0 {
        score -= 5 * test.fail_count as i32;
    }

    // Open quality issues
    if quality.open_ncrs > 5 || quality.open_capas > 3 {
        score -= 15;
    }

    // Overdue items
    if quality.overdue > 0 {
        score -= 10 * quality.overdue as i32;
    }

    // Tolerance analysis failures
    if tol.stackups_fail > 0 {
        score -= 15 * tol.stackups_fail as i32;
    }

    // Marginal stackups
    if tol.stackups_marginal > 0 {
        score -= 5 * tol.stackups_marginal as i32;
    }

    match score {
        80..=100 => "Healthy".to_string(),
        50..=79 => "Warning".to_string(),
        _ => "Critical".to_string(),
    }
}
