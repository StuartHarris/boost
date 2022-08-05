use crate::config;
use color_eyre::eyre::Result;
use tabled::{object::Columns, Format, Modify, Style, Table, Tabled};
use yansi::Paint;

#[derive(Tabled)]
struct Task {
    name: String,
    description: String,
    runs: String,
}

pub fn show() -> Result<()> {
    let configs = config::find_all()?;
    if configs.is_empty() {
        println!("no tasks found in the current directory");
    } else {
        let cyan = Format::new(|s| Paint::cyan(s).to_string());
        let blue = Format::new(|s| Paint::blue(s).to_string());
        let green = Format::new(|s| Paint::green(s).to_string());

        let tasks = configs.into_iter().map(|t| {
            let name = Paint::cyan(&t.name);
            let file = Paint::cyan(format!("(./{}.toml)", t.name));
            Task {
                name: format!("{} {}", Paint::wrapping(&name).bold(), file),
                description: t.config.description.unwrap_or_default(),
                runs: t.config.run,
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
