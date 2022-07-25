use crate::config::Output;
use color_eyre::Result;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::{fs::File, path::Path};
use tar::Builder;

const OUTPUT_TAR_FILE: &str = "output.tar";

pub fn write_archive(outputs: &[Output], cache_dir: &Path) -> Result<()> {
    let file = File::create(&cache_dir.join(OUTPUT_TAR_FILE))?;
    let mut context = Builder::new(file);

    for ouput in outputs {
        let mut builder = GlobSetBuilder::new();
        for filter in &ouput.filters {
            builder.add(Glob::new(filter)?);
        }
        let filters = builder.build()?;

        for file in WalkBuilder::new(&ouput.root)
            .hidden(false)
            .ignore(false)
            .build()
            .flatten()
            .map(|f| f.into_path())
            .filter(|f| f.is_file() && filters.is_match(f))
        {
            println!("{}", file.as_path().to_string_lossy());
            context.append_path(file)?;
        }
    }

    Ok(())
}
