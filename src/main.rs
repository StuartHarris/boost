use anyhow::{Context, Result};
use b2sum_rs::Blake2bSum;
use clap::Parser;
use globset::{Glob, GlobSetBuilder};
use ignore::Walk;
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
    let set = builder.build()?;
    let context = Blake2bSum::new(16);
    let mut all: Vec<u8> = Vec::new();
    let files = Walk::new("./")
        .flatten()
        .map(|f| f.into_path())
        .filter(|f| f.is_file() && set.is_match(f))
        .map(|f| {
            (f.clone(), {
                let hex = context.read(f);
                all.extend(Blake2bSum::as_bytes(&hex));
                hex
            })
        })
        .collect::<Vec<_>>();
    println!("{:#?}", files);
    println!("{:#?}", context.read_bytes(&all));

    println!("{:?}", config.outputs);
    println!("{:?}", config.run);
    Ok(())
}
