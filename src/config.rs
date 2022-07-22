use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub run: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Input {
    pub root: String,
    pub filters: Vec<String>,
    pub commands: Option<Vec<String>>,
    pub env_vars: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Output {
    pub root: String,
    pub filters: Vec<String>,
}
