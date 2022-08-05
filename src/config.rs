use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct ConfigFile {
    pub config: Config,
    pub name: String,
    pub bytes: Vec<u8>,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub description: Option<String>,
    pub run: String,
    pub input: Input,
    pub output: Option<Output>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub files: Option<Vec<Selector>>,
    pub invariants: Option<Vec<String>>,
    pub env_vars: Option<Vec<String>>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            files: Some(vec![Selector::default()]),
            invariants: Default::default(),
            env_vars: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Output {
    pub files: Option<Vec<Selector>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Selector {
    pub root: String,
    pub filters: Vec<String>,
}

impl Default for Selector {
    fn default() -> Self {
        Self {
            root: ".".to_string(),
            filters: vec!["*".to_string()],
        }
    }
}

/// find all the parsable configuration files in the current directory
pub fn find_all() -> Result<Vec<ConfigFile>> {
    let found = fs::read_dir(".")?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().unwrap_or_default() == "toml" {
                try_read_config(&path).ok()
            } else {
                None
            }
        })
        .collect();
    Ok(found)
}

/// return configuration files for the specified commands
pub fn find<T>(tasks: &[T]) -> Result<Vec<ConfigFile>>
where
    T: AsRef<str> + Display,
{
    tasks
        .iter()
        .map(|task| {
            let path = task.as_ref().to_string() + ".toml";
            let path = Path::new(&path);
            try_read_config(path)
        })
        .collect::<Result<Vec<_>>>()
}

fn try_read_config(path: &Path) -> Result<ConfigFile> {
    match fs::read(path).wrap_err_with(|| format!("opening {}", path.to_string_lossy())) {
        Ok(bytes) => match toml::from_slice::<Config>(&bytes).wrap_err("parsing TOML") {
            Ok(config) => Ok(ConfigFile {
                config,
                name: path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
                bytes,
                path: path.into(),
            }),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
