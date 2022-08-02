use crate::{cache, config::Selector};
use bytesize::ByteSize;
use color_eyre::Result;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::{
    fs::{self, File, OpenOptions},
    io::copy,
    os::unix::prelude::OpenOptionsExt,
    path::Path,
};
use tar::{Archive, Builder};

pub fn write_archive(selectors: &[Selector], cache_dir: &Path) -> Result<()> {
    let file = File::create(cache_dir.join(cache::OUTPUT_TAR_FILE))?;
    let mut context = Builder::new(file);

    for selector in selectors {
        let mut builder = GlobSetBuilder::new();
        for filter in &selector.filters {
            builder.add(Glob::new(filter)?);
        }
        let filters = builder.build()?;

        for file in WalkBuilder::new(&selector.root)
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

pub fn read_archive(selectors: &[Selector], cache_dir: &Path) -> Result<()> {
    for selector in selectors {
        fs::create_dir_all(&selector.root)?;
    }

    if let Ok(file) = File::open(cache_dir.join(cache::OUTPUT_TAR_FILE)) {
        let mut a = Archive::new(file);

        for file in a.entries()? {
            // Make sure there wasn't an I/O error
            let mut in_file = file?;

            let path = in_file.header().path()?;

            info!(
                "restoring {:?} ({})",
                path,
                ByteSize(in_file.header().size()?)
            );

            let mode = in_file.header().mode()?;
            let mut out_file = OpenOptions::new()
                .write(true)
                .create(true)
                .mode(mode)
                .open(path)?;
            copy(&mut in_file, &mut out_file)?;
        }
    } else {
        info!("no archive found");
    }

    Ok(())
}
