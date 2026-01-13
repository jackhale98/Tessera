mod cli;

use clap::Parser;
use miette::Result;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    // Reset SIGPIPE to default behavior (terminate silently) for proper Unix piping.
    // Without this, piping to `head`, `grep -q`, etc. causes a panic on broken pipe.
    // This is standard practice for CLI tools that output to stdout.
    #[cfg(unix)]
    {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }
    // Install miette's fancy error handler for beautiful diagnostics
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .context_lines(2)
                .tab_width(4)
                .build(),
        )
    }))?;

    let cli = Cli::parse();
    let global = cli.global;

    match cli.command {
        Commands::Init(args) => cli::commands::init::run(args),
        Commands::Req(cmd) => cli::commands::req::run(cmd, &global),
        Commands::Haz(cmd) => cli::commands::haz::run(cmd, &global),
        Commands::Risk(cmd) => cli::commands::risk::run(cmd, &global),
        Commands::Test(cmd) => cli::commands::test::run(cmd, &global),
        Commands::Rslt(cmd) => cli::commands::rslt::run(cmd, &global),
        Commands::Cmp(cmd) => cli::commands::cmp::run(cmd, &global),
        Commands::Asm(cmd) => cli::commands::asm::run(cmd, &global),
        Commands::Quote(cmd) => cli::commands::quote::run(cmd, &global),
        Commands::Sup(cmd) => cli::commands::sup::run(cmd, &global),
        Commands::Proc(cmd) => cli::commands::proc::run(cmd, &global),
        Commands::Ctrl(cmd) => cli::commands::ctrl::run(cmd, &global),
        Commands::Work(cmd) => cli::commands::work::run(cmd, &global),
        Commands::Lot(cmd) => cli::commands::lot::run(cmd, &global),
        Commands::Dev(cmd) => cli::commands::dev::run(cmd, &global),
        Commands::Ncr(cmd) => cli::commands::ncr::run(cmd, &global),
        Commands::Capa(cmd) => cli::commands::capa::run(cmd, &global),
        Commands::Feat(cmd) => cli::commands::feat::run(cmd, &global),
        Commands::Mate(cmd) => cli::commands::mate::run(cmd, &global),
        Commands::Tol(cmd) => cli::commands::tol::run(cmd, &global),
        Commands::Validate(args) => cli::commands::validate::run(args),
        Commands::Link(cmd) => cli::commands::link::run(cmd),
        Commands::Log(args) => cli::commands::log::run(args, &global),
        Commands::Trace(cmd) => cli::commands::trace::run(cmd, &global),
        Commands::Dsm(args) => cli::commands::dsm::run(args, &global),
        Commands::Dmm(args) => cli::commands::dmm::run(args, &global),
        Commands::Report(cmd) => cli::commands::report::run(cmd, &global),
        Commands::WhereUsed(args) => cli::commands::where_used::run(args, &global),
        Commands::History(args) => cli::commands::history::run(args),
        Commands::Blame(args) => cli::commands::blame::run(args),
        Commands::Diff(args) => cli::commands::diff::run(args),
        Commands::Baseline(cmd) => cli::commands::baseline::run(cmd),
        Commands::Submit(args) => args.run(&global),
        Commands::Approve(args) => args.run(&global),
        Commands::Reject(args) => args.run(&global),
        Commands::Release(args) => args.run(&global),
        Commands::Review(cmd) => cmd.run(&global),
        Commands::Team(cmd) => cmd.run(&global),
        Commands::Import(args) => cli::commands::import::run(args),
        Commands::Bulk(cmd) => cli::commands::bulk::run(cmd),
        Commands::Status(args) => cli::commands::status::run(args, &global),
        Commands::Cache(cmd) => cli::commands::cache::run(cmd),
        Commands::Config(cmd) => cli::commands::config::run(cmd, &global),
        Commands::Search(args) => cli::commands::search::run(args, &global),
        Commands::Recent(args) => cli::commands::recent::run(args, &global),
        Commands::Tags(cmd) => cli::commands::tags::run(cmd, &global),
        Commands::Schema(cmd) => cli::commands::schema::run(cmd),
        Commands::Completions(args) => cli::commands::completions::run(args),
    }
}
