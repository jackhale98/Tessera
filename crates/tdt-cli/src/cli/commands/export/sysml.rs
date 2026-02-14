//! `tdt export sysml` - Export project entities as SysML v2

use console::style;
use miette::Result;
use std::path::PathBuf;

use crate::cli::commands::report::{
    load_all_components, load_all_requirements, load_all_results, load_all_tests, write_output,
};
use crate::cli::GlobalOpts;
use tdt_core::core::project::Project;
use tdt_core::sysml::export::{generate_sysml, ExportContext};

#[derive(clap::Args, Debug)]
pub struct SysmlExportArgs {
    /// Output file path (default: stdout)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    /// SysML package name (default: project directory name)
    #[arg(long = "package")]
    pub package: Option<String>,
}

pub fn run(args: SysmlExportArgs, _global: &GlobalOpts) -> Result<()> {
    let project = Project::discover().map_err(|e| miette::miette!("{}", e))?;

    // Determine package name
    let package_name = args.package.unwrap_or_else(|| {
        project
            .root()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "TesseraExport".to_string())
    });

    // Load all relevant entities
    let requirements = load_all_requirements(&project);
    let tests = load_all_tests(&project);
    let results = load_all_results(&project);
    let components = load_all_components(&project);

    if args.file.is_some() {
        eprintln!(
            "{} Exporting {} requirements, {} tests, {} results, {} components as SysML v2",
            style("->").blue(),
            style(requirements.len()).cyan(),
            style(tests.len()).cyan(),
            style(results.len()).cyan(),
            style(components.len()).cyan(),
        );
    }

    let ctx = ExportContext {
        package_name,
        requirements,
        tests,
        results,
        components,
    };

    let output = generate_sysml(&ctx);
    write_output(&output, args.file)?;

    Ok(())
}
