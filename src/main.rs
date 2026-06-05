mod cmd;
mod error;
mod core;

use clap::Parser;
use miette::IntoDiagnostic;

fn main() -> miette::Result<()> {
    let cli = cmd::Cli::parse();
    let result = match &cli.command {
        cmd::Commands::Dep(args) => cmd::dep::run(args),
        cmd::Commands::Metadata(args) => cmd::metadata::run(args),
        cmd::Commands::Type(args) => cmd::r#type::run(args),
    };

    result.into_diagnostic()?;

    Ok(())
}
