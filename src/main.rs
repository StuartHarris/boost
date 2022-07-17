use anyhow::{Context, Result};
use b2sum_rs::Blake2bSum;
use clap::Parser;
use globset::{Glob, GlobSetBuilder};
use ignore::Walk;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read},
    path::PathBuf,
    str::FromStr,
};

const CACHE_DIR: &str = ".boost";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// name of TOML configuration file
    #[clap(short, long, value_parser)]
    file: PathBuf,
}

#[derive(Deserialize, Debug)]
struct Config {
    inputs: Vec<String>,
    outputs: Vec<String>,
    run: Option<String>,
}

#[derive(Serialize, Deserialize, Default, PartialEq)]
struct Hash {
    value: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = fs::read(args.file).context("opening file")?;
    let config: Config = toml::from_slice(&file).context("parsing TOML")?;

    let mut builder = GlobSetBuilder::new();
    for input in config.inputs {
        builder.add(Glob::new(&input)?);
    }
    let set = builder.build()?;
    let context = Blake2bSum::new(16);
    let mut all: Vec<u8> = Vec::new();
    for file in Walk::new("./")
        .flatten()
        .map(|f| f.into_path())
        .filter(|f| f.is_file() && set.is_match(f))
    {
        let hex = context.read(file);
        all.extend(Blake2bSum::as_bytes(&hex));
    }

    fs::create_dir_all(CACHE_DIR)?;
    let path = PathBuf::from_str(CACHE_DIR)?.join("hash");

    let previous: Hash = {
        if let Ok(mut f) = File::open(&path) {
            let mut s = String::new();
            f.read_to_string(&mut s)
                .with_context(|| format!("reading {}", path.to_string_lossy()))?;
            serde_json::from_str(&s)?
        } else {
            Hash::default()
        }
    };

    let current = Hash {
        value: context.read_bytes(&all),
    };

    if current != previous {
        println!("inputs have changed");
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("opening {} for writing", path.to_string_lossy()))?;
        let writer = BufWriter::new(&f);
        f.set_len(0)?;
        serde_json::to_writer(writer, &current)?;
    }

    println!("{:?}", config.outputs);
    println!("{:?}", config.run);
    Ok(())
}
