use std::path::PathBuf;

use crate::error::Result;

#[derive(clap::Args)]
pub struct Args {
    pub file: PathBuf,

    #[arg(
        long = "include-jar-in-jar",
        short = 'j',
        alias = "jij",
        help = "Include embedded dependencies (JAR-in-JAR) in dependency report"
    )]
    pub include_jar_in_jar: bool,
}

pub fn run(args: &Args) -> Result<()> {
    println!(
        "dep command: file={}, include-jar-in-jar={}",
        args.file.display(),
        args.include_jar_in_jar
    );
    Ok(())
}
