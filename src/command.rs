use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Command {
    pub target: String,
    #[serde(default)]
    pub args: Vec<String>,
}
