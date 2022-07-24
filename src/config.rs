use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub run: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Input {
    pub root: String,
    pub filters: Vec<String>,
    pub commands: Option<Vec<String>>,
    pub env_vars: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Output {
    pub root: String,
    pub filters: Vec<String>,
}
