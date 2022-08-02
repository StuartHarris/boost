#[macro_use]
extern crate log;
extern crate lazy_static;

mod archive;
mod cache;
mod command_runner;
pub mod config;
mod duration;

use crate::cache::{Hash, Manifest};
use clap::Parser;
use color_eyre::eyre::Result;
use duration::format_duration;
use std::{
    env,
    time::{Instant, SystemTime},
};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// log with level DEBUG
    #[clap(short, long, global(true), parse(from_occurrences))]
    verbose: usize,

    /// Run the tasks specified (each will expect to find a TOML config file called "<name>.toml").
    /// If none specified, list all the tasks for which there is a valid configuration file in the current directory
    tasks: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    if args.verbose > 0 {
        env::set_var("RUST_LOG", "trace");
    }
    sensible_env_logger::init_timed!();

    if args.tasks.is_empty() {
        let commands = config::find_all()?;
        if commands.is_empty() {
            info!("no commands found")
        } else {
            info!(
                "found command{} {}",
                if commands.len() == 1 { "" } else { "s" },
                commands
                    .into_iter()
                    .map(|d| format!("\"{}\"", d.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    } else {
        for config_file in config::find(&args.tasks)? {
            let start = Instant::now();
            let config = config_file.config;

            println!();
            info!(
                "using config \"{}\" ({})",
                config.description.as_deref().unwrap_or("<no description>"),
                config_file.path.to_string_lossy()
            );

            let current = Hash::new(&config.input, &config_file.bytes)?;
            if let Some((path, previous)) = Manifest::read(&current)? {
                let ago = format_duration(SystemTime::now().duration_since(previous.created)?);
                info!("found local cache from {ago} ago, reprinting output...\n");

                let cache_dir = path
                    .parent()
                    .expect("manifest should have parent directory");
                let mut f = File::open(&cache_dir.join(cache::OUTPUT_COLORS_TXT_FILE)).await?;

                let mut buffer = String::new();
                f.read_to_string(&mut buffer).await?;

                println!("{}", buffer);

                if let Some(output) = config.output {
                    archive::read_archive(&output.files.unwrap_or_default(), cache_dir)?;
                }
            } else {
                info!("no cache found, executing \"{}\"\n", &config.run);

                let path = Manifest::new(current, &config).write()?;
                let cache_dir = path
                    .parent()
                    .expect("manifest should have parent directory");

                command_runner::run(&config.run, cache_dir).await?;

                if let Some(output) = config.output {
                    archive::write_archive(&output.files.unwrap_or_default(), cache_dir)?;
                }
            };
            info!(
                "Finished {}, in {}",
                config_file.name,
                format_duration(Instant::now() - start)
            );
        }
    }

    Ok(())
}
