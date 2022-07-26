use crate::config::{self, Output};
use color_eyre::Result;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::{
    fs::{self, File},
    io::copy,
    path::Path,
};
use tar::{Archive, Builder};

pub fn write_archive(outputs: &[Output], cache_dir: &Path) -> Result<()> {
    let file = File::create(cache_dir.join(config::OUTPUT_TAR_FILE))?;
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
            info!("archiving \"{}\"", file.as_path().to_string_lossy());
            context.append_path(file)?;
        }
    }

    Ok(())
}

pub fn read_archive(outputs: &[Output], cache_dir: &Path) -> Result<()> {
    for ouput in outputs {
        fs::create_dir_all(&ouput.root)?;
    }

    let file = File::open(cache_dir.join(config::OUTPUT_TAR_FILE))?;
    let mut a = Archive::new(file);

    for file in a.entries()? {
        // Make sure there wasn't an I/O error
        let mut in_file = file?;

        let path = in_file.header().path()?;

        info!(
            "restoring {:?} ({})",
            path,
            bytesize::ByteSize(in_file.header().size()?)
        );

        let mut out_file = File::create(path)?;
        copy(&mut in_file, &mut out_file)?;
    }

    Ok(())
}
