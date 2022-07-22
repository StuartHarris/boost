mod cache;
pub mod config;

use crate::{
    cache::{Hash, Manifest},
    config::Config,
};
use anyhow::{Context, Result};
use clap::Parser;
use humantime::format_duration;
use std::{
    env, fs,
    path::PathBuf,
    time::{Duration, SystemTime},
};

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

    let current = Hash::new(&config.inputs, &file, env::args())?;
    if let Some(previous) = Manifest::read(&current)? {
        let duration = SystemTime::now().duration_since(previous.created)?;
        let truncated = Duration::new(duration.as_secs(), 0);
        println!("found local cache from {} ago", format_duration(truncated));
    } else {
        println!("no cache found");
        Manifest::new(current).write()?;
    };

    println!("{:?}", config.outputs);
    println!("{:?}", config.run);
    Ok(())
}
