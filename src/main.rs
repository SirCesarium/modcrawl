//! Command-line entry point for `modcrawl`.

use std::io::ErrorKind;
use std::panic;
use std::process;

use clap::Parser;
use miette::Result;

use modcrawl::cmd::{self, Commands};
use modcrawl::error::Error;

fn main() -> Result<()> {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let is_broken_pipe = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.contains("Broken pipe")
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.contains("Broken pipe")
        } else {
            false
        };
        if is_broken_pipe {
            process::exit(0);
        }
        default_hook(info);
    }));

    let cli = cmd::Cli::parse();
    let result = match &cli.command {
        Commands::Dep(args) => cmd::dep::run(args),
        Commands::Metadata(args) => cmd::metadata::run(args),
        Commands::Type(args) => cmd::r#type::run(args),
        Commands::Side(args) => cmd::side::run(args),
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
        Err(Error::Io(ref e)) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
        Err(Error::Usage(msg)) => {
            eprintln!("error: {msg}");
            process::exit(2);
        }
        Err(Error::Failed) => {
            process::exit(1);
        }
        Err(e) => Err(e.into()),
    }
}
