//! A static site generator for blogs.
mod output;

mod model;
use model::Articles;

use anyhow::{Context, Result};
use clap::Parser;

/// Commandline arguments.
#[derive(Parser, Debug)]
struct Cli {
    /// The source directory.
    #[clap(long)]
    input: String,

    /// The output directory.
    #[clap(long)]
    output: String,
}

fn run_on_args(args: impl Iterator<Item = std::ffi::OsString>) -> Result<()> {
    let args = Cli::parse_from(args);
    let mut blogs = Articles::read(&args.input.as_ref())?;
    blogs.write(&output::templates()?, &args.output.as_ref())?;
    Ok(())
}

fn main() {
    if let Err(e) = run_on_args(std::env::args_os()) {
        println!("Error: {:?}", e);
    }
}