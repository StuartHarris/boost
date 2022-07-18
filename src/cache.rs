use anyhow::{Context, Result};
use b2sum_rs::Blake2bSum;
use globset::GlobSet;
use ignore::Walk;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

const CACHE_DIR: &str = ".boost";
const MANIFEST: &str = "manifest.json";

#[derive(Serialize, Deserialize, Default, PartialEq)]
pub struct Hash(String);

impl AsRef<Path> for Hash {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Manifest {
    pub hash: Hash,
}

impl Manifest {
    pub fn new(hash: Hash) -> Self {
        Self { hash }
    }

    pub fn read(hash: &Hash) -> Result<Option<Self>> {
        fs::create_dir_all(CACHE_DIR)?;
        let path = PathBuf::from_str(CACHE_DIR)?.join(hash).join(MANIFEST);
        if let Ok(mut f) = File::open(&path) {
            let mut s = String::new();
            f.read_to_string(&mut s)
                .with_context(|| format!("reading {}", path.to_string_lossy()))?;
            let manifest = serde_json::from_str(&s)?;
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    pub fn write(&self) -> Result<()> {
        let path = PathBuf::from_str(CACHE_DIR)?.join(&self.hash);
        fs::create_dir_all(&path)?;
        let path = PathBuf::from(&path).join(MANIFEST);
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("opening {} for writing", path.to_string_lossy()))?;
        f.set_len(0)?;
        let writer = BufWriter::new(&f);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
}

impl Hash {
    pub fn new(patterns: &GlobSet) -> Result<Self> {
        let context = Blake2bSum::new(16);
        let mut all: Vec<u8> = Vec::new();
        for file in Walk::new("./")
            .flatten()
            .map(|f| f.into_path())
            .filter(|f| f.is_file() && patterns.is_match(f))
        {
            let hex = context.read(file);
            all.extend(Blake2bSum::as_bytes(&hex));
        }
        Ok(Self(context.read_bytes(&all)))
    }
}
