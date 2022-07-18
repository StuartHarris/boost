mod cache;

use crate::cache::{Hash, Manifest};
use anyhow::{Context, Result};
use clap::Parser;
use globset::{Glob, GlobSetBuilder};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// name of TOML configuration file
    #[clap(short, long, value_parser)]
    file: PathBuf,
}

#[derive(Deserialize, Debug)]
struct Config {
    inputs: Vec<String>,
    outputs: Vec<String>,
    run: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = fs::read(args.file).context("opening file")?;
    let config: Config = toml::from_slice(&file).context("parsing TOML")?;

    let mut builder = GlobSetBuilder::new();
    for input in config.inputs {
        builder.add(Glob::new(&input)?);
    }
    let input_patterns = builder.build()?;
    let current = Hash::new(&input_patterns)?;
    let previous = Manifest::read(&current)?.unwrap_or_default().hash;
    if current != previous {
        println!("inputs have changed");
        Manifest::new(current).write()?;
    }

    println!("{:?}", config.outputs);
    println!("{:?}", config.run);
    Ok(())
}
