mod cmd;
mod error;

use clap::Parser;

fn main() -> miette::Result<()> {
    let cli = cmd::Cli::parse();
    match &cli.command {
        cmd::Commands::Dep(args) => cmd::dep::run(args),
        cmd::Commands::Metadata(args) => cmd::metadata::run(args),
        cmd::Commands::Type(args) => cmd::r#type::run(args),
    }
}
