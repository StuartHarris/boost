use tabled::{format::Format, object::Columns, Alignment, Modify, Style, Table, Tabled};
use yansi::Paint;

pub struct Reporter;

impl Reporter {
    pub fn show_tasks<T: Tabled>(tasks: impl Iterator<Item = T>) {
        let cyan = Format::new(|s| Paint::cyan(s).to_string());
        let blue = Format::new(|s| Paint::blue(s).to_string());
        let green = Format::new(|s| Paint::green(s).to_string());

        let mut table = Table::new(tasks);
        table
            .with(Style::rounded())
            .with(Alignment::left())
            .with(Modify::new(Columns::single(0)).with(cyan))
            .with(Modify::new(Columns::single(1)).with(blue))
            .with(Modify::new(Columns::new(2..)).with(green));
        println!("\ntasks in the current directory");
        println!("{}\n", table);
    }

    pub(crate) fn get(task_id: &str) -> impl Fn(&str) {
        let task_id = task_id.to_string();
        move |msg| {
            if !msg.is_empty() {
                let label = Paint::cyan(task_id.clone()).bold();
                println!("{label}: {}", msg);
            }
        }
    }
}
