use crate::{
    archive, cache,
    cache::{Hash, Manifest},
    command_runner::CommandRunner,
    config_file::{self, ConfigFile},
    duration,
    reporter::Reporter,
};
use async_channel::unbounded;
use async_std::fs::File;
use bevy_tasks::AsyncComputeTaskPool;
use color_eyre::eyre::Result;
use duration::format_duration;
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use std::{
    str,
    time::{Instant, SystemTime},
};
use tabled::Tabled;
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

        Reporter::show_tasks(tasks);
    }
    Ok(())
}

pub async fn run_task(config_file: &ConfigFile) -> Result<String> {
    let start = Instant::now();
    let config = &config_file.config;
    let task_id = config_file.id.clone();

    let report = &Reporter::get(&task_id);

    report(&format!(
        "using config \"{}\" ({})",
        config.description.as_deref().unwrap_or("<no description>"),
        config_file.path.to_string_lossy()
    ));

    let current = Hash::new(&config.input, &config_file.bytes).await?;
    if let Some((path, previous)) = Manifest::read(&current).await? {
        let ago = format_duration(SystemTime::now().duration_since(previous.created)?);

        report(&format!(
            "found local cache from {ago} ago, reprinting output...\n"
        ));

        let cache_dir = path
            .parent()
            .expect("manifest should have parent directory");
        let mut f = File::open(&cache_dir.join(cache::OUTPUT_COLORS_TXT_FILE)).await?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).await?;

        report(&buffer.to_string());

        if let Some(output) = config.output.as_ref() {
            archive::read_archive(output.files.as_deref().unwrap_or_default(), cache_dir).await?;
        }
    } else {
        report(&format!("no cache found, executing \"{}\"\n", &config.run));

        let (tx, rx) = unbounded();

        let pool = AsyncComputeTaskPool::get();
        pool.scope(|s: &mut bevy_tasks::Scope<'_, Result<()>>| {
            s.spawn(async move { CommandRunner::get().run(&config.run, tx).await });
            s.spawn(async move { process_log_stream(current, config, rx, report).await });
        })
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    };
    report(&format!(
        "Finished in {}",
        format_duration(Instant::now() - start)
    ));
    Ok("This is the resulting hash".to_string())
}

async fn process_log_stream(
    current: Hash,
    config: &config_file::Config,
    rx: async_channel::Receiver<Vec<u8>>,
    report: &impl Fn(&str),
) -> Result<()> {
    let path = Manifest::new(current, config).write().await?;
    let cache_dir = path
        .parent()
        .expect("manifest should have parent directory");
    let mut writer_colors = File::create(cache_dir.join(cache::OUTPUT_COLORS_TXT_FILE)).await?;
    let mut writer_plain = File::create(cache_dir.join(cache::OUTPUT_PLAIN_TXT_FILE)).await?;
    while let Ok(msg) = rx.recv().await {
        if !msg.is_empty() {
            report(str::from_utf8(&msg)?);

            writer_colors.write_all(&msg).await?;
            let plain = strip_ansi_escapes::strip(msg)?;
            writer_plain.write_all(&plain).await?;
        }
    }
    if let Some(output) = &config.output {
        archive::write_archive(output.files.as_deref().unwrap_or_default(), cache_dir).await?;
    }
    Ok(())
}
