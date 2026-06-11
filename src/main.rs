use clap::Parser;
use miette::IntoDiagnostic;

use modcrawl::cmd::{self, Commands};

fn main() -> miette::Result<()> {
    let cli = cmd::Cli::parse();
    let result = match &cli.command {
        Commands::Dep(args) => cmd::dep::run(args),
        Commands::Metadata(args) => cmd::metadata::run(args),
        Commands::Type(args) => cmd::r#type::run(args),
        #[cfg(feature = "classfile")]
        Commands::Classes(args) => cmd::classes::run(args),
        #[cfg(feature = "classfile")]
        Commands::Grep(args) => cmd::grep::run(args),
    };
    result.into_diagnostic()?;
    Ok(())
}
