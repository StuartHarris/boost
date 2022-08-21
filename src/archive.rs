use crate::{cache, config_file::Selector};
use async_std::{
    fs::{self, File, OpenOptions},
    os::unix::fs::OpenOptionsExt,
    path::Path,
};
use async_tar::{Archive, Builder};
use bytesize::ByteSize;
use color_eyre::Result;
use futures_lite::{io::copy, StreamExt};
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;

pub async fn write_archive(selectors: &[Selector], cache_dir: &Path) -> Result<()> {
    let file = File::create(cache_dir.join(cache::OUTPUT_TAR_FILE)).await?;
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
            context.append_path(file).await?;
        }
    }

    Ok(())
}

pub async fn read_archive(selectors: &[Selector], cache_dir: &Path) -> Result<()> {
    for selector in selectors {
        fs::create_dir_all(&selector.root).await?;
    }

    if let Ok(file) = File::open(cache_dir.join(cache::OUTPUT_TAR_FILE)).await {
        let a = Archive::new(file);
        let mut entries = a.entries()?;

        while let Some(file) = entries.next().await {
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
                .open(path)
                .await?;
            copy(&mut in_file, &mut out_file).await?;
        }
    } else {
        info!("no archive found");
    }

    Ok(())
}
