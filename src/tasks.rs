use crate::{
    archive, cache,
    cache::{Hash, Manifest},
    command_runner::CommandRunner,
    config_file::{self, ConfigFile},
    duration,
};
use async_std::fs::File;
use color_eyre::eyre::Result;
use duration::format_duration;
use futures_lite::AsyncReadExt;
use std::time::{Instant, SystemTime};
use tabled::{object::Columns, Format, Modify, Style, Table, Tabled};
use yansi::Paint;

#[derive(Tabled)]
struct Task {
    name: String,
    description: String,
    runs: String,
    #[tabled(rename = "depends on")]
    depends_on: String,
}

pub async fn show() -> Result<()> {
    let configs = config_file::find_all().await?;
    if configs.is_empty() {
        println!("no tasks found in the current directory");
    } else {
        let cyan = Format::new(|s| Paint::cyan(s).to_string());
        let blue = Format::new(|s| Paint::blue(s).to_string());
        let green = Format::new(|s| Paint::green(s).to_string());

        let tasks = configs.into_iter().map(|t| {
            let name = Paint::cyan(&t.id);
            let file = Paint::cyan(format!("(./{}.toml)", t.id));
            Task {
                name: format!("{} {}", Paint::wrapping(&name).bold(), file),
                description: t.config.description.unwrap_or_default(),
                runs: t.config.run,
                depends_on: t
                    .config
                    .depends_on
                    .unwrap_or_default()
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            }
        });

        let table = Table::new(tasks)
            .with(Style::rounded().lines([(1, Style::modern().get_horizontal())]))
            .with(Modify::new(Columns::single(0)).with(cyan))
            .with(Modify::new(Columns::single(1)).with(blue))
            .with(Modify::new(Columns::new(2..)).with(green));

        println!("\ntasks in the current directory");
        println!("{}\n", table);
    }
    Ok(())
}

pub async fn run_task(config_file: &ConfigFile) -> Result<String> {
    let start = Instant::now();
    let config = &config_file.config;

    let label = Paint::cyan(&config_file.id).bold();

    println!();
    info!(
        "{label}: using config \"{}\" ({})",
        config.description.as_deref().unwrap_or("<no description>"),
        config_file.path.to_string_lossy()
    );

    let current = Hash::new(&config.input, &config_file.bytes).await?;
    if let Some((path, previous)) = Manifest::read(&current).await? {
        let ago = format_duration(SystemTime::now().duration_since(previous.created)?);

        info!("{label}: found local cache from {ago} ago, reprinting output...\n");

        let cache_dir = path
            .parent()
            .expect("manifest should have parent directory");
        let mut f = File::open(&cache_dir.join(cache::OUTPUT_COLORS_TXT_FILE)).await?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).await?;

        println!("{}", buffer);

        if let Some(output) = config.output.as_ref() {
            archive::read_archive(output.files.as_deref().unwrap_or_default(), cache_dir).await?;
        }
    } else {
        info!("{label}: no cache found, executing \"{}\"\n", &config.run);

        let path = Manifest::new(current, config).write().await?;
        let cache_dir = path
            .parent()
            .expect("manifest should have parent directory");

        let runner = CommandRunner::get();
        runner.run(&config.run, cache_dir.into()).await?;

        if let Some(output) = &config.output {
            archive::write_archive(output.files.as_deref().unwrap_or_default(), cache_dir).await?;
        }
    };
    info!(
        "{label}: Finished in {}",
        format_duration(Instant::now() - start)
    );
    Ok("This is the resulting hash".to_string())
}
