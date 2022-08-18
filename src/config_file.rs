use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct ConfigFile {
    pub id: String,
    pub parent: Option<String>,
    pub config: Config,
    pub bytes: Vec<u8>,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub description: Option<String>,
    pub run: String,
    pub depends_on: Option<Vec<DependsOn>>,
    pub input: Input,
    pub output: Option<Output>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DependsOn {
    pub name: Option<String>,
}

impl Display for DependsOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.clone().unwrap_or_default())
    }
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

/// recursively find all configuration files
pub fn build_tree<T>(task_names: &[T]) -> Result<Vec<ConfigFile>>
where
    T: AsRef<str> + Display,
{
    let mut tree = vec![];
    for task in task_names {
        tree_rec(&mut tree, task, None)?;
    }
    Ok(tree)
}

fn tree_rec<T>(tree: &mut Vec<ConfigFile>, task: &T, parent: Option<&T>) -> Result<()>
where
    T: AsRef<str> + Display,
{
    if !tree.iter().any(|c: &ConfigFile| c.id == task.as_ref()) {
        let mut config_file = find_one(task)?;
        config_file.parent = parent.map(|p| p.to_string());
        if let Some(deps) = &config_file.config.depends_on {
            let task_names = deps
                .iter()
                .flat_map(|d| d.name.clone())
                .collect::<Vec<String>>();
            for task in task_names {
                tree_rec(tree, &task, Some(&config_file.id))?;
            }
        }
        tree.push(config_file);
    }

    Ok(())
}

/// find all the parsable configuration files in the current directory
pub fn find_all() -> Result<Vec<ConfigFile>> {
    let found = fs::read_dir(".")?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().unwrap_or_default() == "toml" {
                try_read_config_file(&path).ok()
            } else {
                None
            }
        })
        .collect();
    Ok(found)
}

/// return configuration file for the specified command
pub fn find_one<T>(task_name: &T) -> Result<ConfigFile>
where
    T: AsRef<str> + Display,
{
    let path = task_name.as_ref().to_string() + ".toml";
    let path = Path::new(&path);
    try_read_config_file(path)
}

fn try_read_config_file(path: &Path) -> Result<ConfigFile> {
    match fs::read(path).wrap_err_with(|| format!("opening {}", path.to_string_lossy())) {
        Ok(bytes) => match toml::from_slice::<Config>(&bytes).wrap_err("parsing TOML") {
            Ok(config) => Ok(ConfigFile {
                id: path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
                parent: None,
                config,
                bytes,
                path: path.into(),
            }),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
