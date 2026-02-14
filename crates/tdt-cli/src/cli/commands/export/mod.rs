//! `tdt export` command - Export project data to interchange formats

mod sysml;

use clap::Subcommand;
use miette::Result;

use crate::cli::GlobalOpts;

pub use sysml::SysmlExportArgs;

#[derive(Subcommand, Debug)]
pub enum ExportCommands {
    /// Export project data as SysML v2 textual notation
    Sysml(SysmlExportArgs),
}

pub fn run(cmd: ExportCommands, global: &GlobalOpts) -> Result<()> {
    match cmd {
        ExportCommands::Sysml(args) => sysml::run(args, global),
    }
}
