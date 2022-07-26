use serde::{Deserialize, Serialize};

pub const OUTPUT_COLORS_TXT_FILE: &str = "output-colors.txt";
pub const OUTPUT_PLAIN_TXT_FILE: &str = "output.txt";
pub const OUTPUT_TAR_FILE: &str = "output.tar";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub inputs: Vec<Input>,
    pub outputs: Option<Vec<Output>>,
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
