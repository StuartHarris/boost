#[macro_use]
extern crate log;
extern crate lazy_static;

mod archive;
mod cache;
mod command_runner;
mod config_file;
mod duration;
mod reporter;
mod task_plugin;
mod tasks;

use atty::Stream;
use bevy_app::App;
use bevy_hierarchy::HierarchyPlugin;
use bevy_internal::MinimalPlugins;
use clap::Parser;
use color_eyre::eyre::Result;
use std::env;
use task_plugin::TaskPlugin;
use yansi::Paint;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = Some(SPLASH))]
struct Args {
    /// log with level DEBUG
    #[arg(short, long, global(true), action = clap::ArgAction::Count)]
    verbose: u8,

    /// Run the tasks specified (each will expect to find a TOML config file called "<name>.toml").
    /// If none specified, list all the tasks for which there is a valid configuration file in the current directory
    tasks: Vec<String>,
}

#[async_std::main]
async fn main() -> Result<()> {
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
        tasks::show().await?;
    } else {
        let config = config_file::build_tree(&args.tasks).await?;
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugin(HierarchyPlugin)
            .insert_resource(config)
            .add_plugin(TaskPlugin)
            .run();
    }

    Ok(())
}

const SPLASH: &str = r#"
________                   _____ 
___  __ )____________________  /_
__  __  |  __ \  __ \_  ___/  __/
_  /_/ // /_/ / /_/ /(__  )/ /_  
/_____/ \____/\____//____/ \__/  
"#;

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
