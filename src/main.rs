#[macro_use]
extern crate log;
extern crate lazy_static;

mod archive;
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

#[tokio::main]
async fn main() -> Result<()> {
    sensible_env_logger::init_timed!();
    color_eyre::install()?;
    let args = Args::parse();

    let start = Instant::now();
    let file = fs::read(&args.file)
        .wrap_err_with(|| format!("opening {}", &args.file.to_string_lossy()))?;
    let config: Config = toml::from_slice(&file).wrap_err("parsing TOML")?;

    let current = Hash::new(&config.inputs, &file, env::args())?;
    if let Some((path, previous)) = Manifest::read(&current)? {
        let ago = format_duration(
            SystemTime::now().duration_since(previous.created)?,
            Lowest::Seconds,
        );
        info!("found local cache from {ago} ago, reprinting output...\n");

        let cache_dir = path
            .parent()
            .expect("manifest should have parent directory");
        let mut f = File::open(&cache_dir.join(config::OUTPUT_COLORS_TXT_FILE)).await?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).await?;

        println!("{}", buffer);

        if let Some(outputs) = config.outputs {
            archive::read_archive(&outputs, cache_dir)?;
        }
    } else {
        info!("no cache found, executing \"{}\"\n", &config.run);

        let path = Manifest::new(current, &config).write()?;
        let cache_dir = path
            .parent()
            .expect("manifest should have parent directory");

        // TODO: investigate why this doesn't catch a process exit with status > 0
        command_runner::run(&config.run, cache_dir).await?;

        if let Some(outputs) = config.outputs {
            archive::write_archive(&outputs, cache_dir)?;
        }
    };

    info!(
        "Finished in {}",
        format_duration(Instant::now() - start, Lowest::MilliSeconds)
    );

    Ok(())
}
