use serde::{Deserialize, Serialize};

pub const OUTPUT_COLORS_TXT_FILE: &str = "output-colors.txt";
pub const OUTPUT_PLAIN_TXT_FILE: &str = "output.txt";
pub const OUTPUT_TAR_FILE: &str = "output.tar";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
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
