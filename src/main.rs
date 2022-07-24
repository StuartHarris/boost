extern crate lazy_static;

mod cache;
mod command_runner;
pub mod config;
mod duration;

use crate::{
    cache::{Hash, Manifest},
    config::Config,
};
use clap::Parser;
use color_eyre::eyre::{Context, Result};
use duration::{format_duration, Lowest};
use std::{
    env, fs,
    path::PathBuf,
    time::{Instant, SystemTime},
};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// name of TOML configuration file
    #[clap(short, long, value_parser)]
    file: PathBuf,
}

const OUTPUT_FILE: &str = "output.txt";

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    color_eyre::install()?;
    let args = Args::parse();

    let file = fs::read(&args.file)
        .wrap_err_with(|| format!("opening {}", &args.file.to_string_lossy()))?;
    let config: Config = toml::from_slice(&file).wrap_err("parsing TOML")?;

    let current = Hash::new(&config.inputs, &file, env::args())?;
    if let Some((path, previous)) = Manifest::read(&current)? {
        let ago = format_duration(
            SystemTime::now().duration_since(previous.created)?,
            Lowest::Seconds,
        );
        println!("found local cache from {ago} ago, reprinting output...\n");

        let cache_dir = path
            .parent()
            .expect("manifest should have parent directory");
        let mut f = File::open(&cache_dir.join(OUTPUT_FILE)).await?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).await?;

        println!("{}", buffer);
    } else {
        println!("no cache found, executing \"{}\"\n", &config.run);

        let path = Manifest::new(current, &config).write()?;
        let cache_dir = path
            .parent()
            .expect("manifest should have parent directory");
        command_runner::run(&config.run, &cache_dir.join(OUTPUT_FILE)).await?;
    };

    println!(
        "Finished in {}",
        format_duration(Instant::now() - start, Lowest::MilliSeconds)
    );
    Ok(())
}
