mod cache;
pub mod config;

use crate::{
    cache::{Hash, Manifest},
    config::Config,
};
use anyhow::{Context, Result};
use clap::Parser;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// name of TOML configuration file
    #[clap(short, long, value_parser)]
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = fs::read(args.file).context("opening file")?;
    let config: Config = toml::from_slice(&file).context("parsing TOML")?;

    let current = Hash::new(&config.inputs)?;
    let previous = Manifest::read(&current)?.unwrap_or_default().hash;
    if current != previous {
        println!("inputs have changed");
        Manifest::new(current).write()?;
    }

    println!("{:?}", config.outputs);
    println!("{:?}", config.run);
    Ok(())
}
