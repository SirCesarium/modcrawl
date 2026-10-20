use std::process;

use clap::Parser;
use miette::Result;

use modcrawl::cmd::{self, Commands};
use modcrawl::error::Error;

fn main() -> Result<()> {
    let cli = cmd::Cli::parse();
    let result = match &cli.command {
        Commands::Dep(args) => cmd::dep::run(args),
        Commands::Metadata(args) => cmd::metadata::run(args),
        Commands::Type(args) => cmd::r#type::run(args),
        #[cfg(feature = "classfile")]
        Commands::Classes(args) => cmd::classes::run(args),
        #[cfg(feature = "classfile")]
        Commands::Grep(args) => cmd::grep::run(args),
        #[cfg(feature = "classfile")]
        Commands::Mixins(args) => cmd::mixins::run(args),
        #[cfg(feature = "classfile")]
        Commands::Dupes(args) => cmd::dupes::run(args),
    };

    match result {
        Ok(()) => Ok(()),
        Err(Error::Usage(msg)) => {
            eprintln!("error: {msg}");
            process::exit(2);
        }
        Err(e) => Err(e.into()),
    }
}
