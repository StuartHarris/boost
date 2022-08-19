#[macro_use]
extern crate log;
extern crate lazy_static;

mod archive;
mod cache;
mod command_runner_plugin;
mod config_file;
mod duration;
mod task_plugin;
mod tasks;

use atty::Stream;
use bevy_app::App;
use bevy_hierarchy::HierarchyPlugin;
use bevy_internal::MinimalPlugins;
use clap::Parser;
use color_eyre::eyre::Result;
use command_runner_plugin::CommandRunnerPlugin;
use std::env;
use task_plugin::TaskPlugin;
use yansi::Paint;

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

fn main() -> Result<()> {
    if !atty::is(Stream::Stdout) {
        Paint::disable();
    }

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
        tasks::show()?;
    } else {
        let config = config_file::build_tree(&args.tasks)?;
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugin(HierarchyPlugin)
            .add_plugin(CommandRunnerPlugin)
            .insert_resource(config)
            .add_plugin(TaskPlugin)
            .run();
    }

    Ok(())
}
