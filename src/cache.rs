use crate::config::{Config, Input, Selector};
use b2sum_rs::Blake2bSum;
use color_eyre::eyre::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use ignore::Walk;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    time::SystemTime,
};

const CACHE_DIR: &str = ".boost";
const MANIFEST: &str = "manifest.json";

#[derive(Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Hash(String);

impl AsRef<Path> for Hash {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub created: SystemTime,
    pub hash: Hash,
    pub config: Config,
}

impl Manifest {
    pub fn new(hash: Hash, config: &Config) -> Self {
        Self {
            created: SystemTime::now(),
            hash,
            config: config.clone(),
        }
    }

    pub fn read(hash: &Hash) -> Result<Option<(PathBuf, Self)>> {
        let path = hash.create_cache_dir()?.join(MANIFEST);
        if let Ok(mut f) = File::open(&path) {
            let mut buf = String::new();
            f.read_to_string(&mut buf)
                .wrap_err_with(|| format!("reading {}", path.to_string_lossy()))?;
            let manifest = serde_json::from_str(&buf)?;
            Ok(Some((path, manifest)))
        } else {
            Ok(None)
        }
    }

    pub fn write(&self) -> Result<PathBuf> {
        let path = self.hash.create_cache_dir()?.join(MANIFEST);
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .wrap_err_with(|| format!("opening {} for writing", path.to_string_lossy()))?;
        f.set_len(0)?;
        let writer = BufWriter::new(&f);
        serde_json::to_writer(writer, self)?;
        Ok(path)
    }
}

impl Hash {
    pub fn new(input: &Input, config_file: &[u8]) -> Result<Self> {
        let context = Blake2bSum::new(16);
        let mut all: Vec<u8> = Vec::new();
        let selectors = input
            .files
            .to_owned()
            .unwrap_or_else(|| vec![Selector::default()]);
        for selector in selectors {
            let mut builder = GlobSetBuilder::new();
            for filter in &selector.filters {
                builder.add(Glob::new(filter)?);
            }
            let filters = builder.build()?;

            for file in Walk::new(&selector.root)
                .flatten()
                .map(|f| f.into_path())
                .filter(|f| f.is_file() && filters.is_match(f))
            {
                let hex = context.read(&file);
                debug!("input file: {:?}", file);
                all.extend(Blake2bSum::as_bytes(&hex));
            }

            if let Some(commands) = &input.invariants {
                for command in commands {
                    let out = Command::new("sh").args(["-c", command]).output()?;
                    let out = out.stdout.as_slice();
                    debug!("input command: {}\n{}", command, std::str::from_utf8(out)?);
                    all.extend_from_slice(out);
                }
            }

            if let Some(env) = &input.env_vars {
                for var in env {
                    if let Ok(val) = env::var(var) {
                        debug!("env var: {} {}", var, val);
                        all.extend_from_slice(val.as_bytes());
                    }
                }
            }
        }

        debug!("config {:?}", std::str::from_utf8(config_file));
        all.extend_from_slice(config_file);

        Ok(Self(context.read_bytes(&all)))
    }

    pub fn create_cache_dir(&self) -> Result<PathBuf> {
        let path = PathBuf::from_str(CACHE_DIR)?.join(self);
        fs::create_dir_all(&path)?;
        Ok(path)
    }
}
