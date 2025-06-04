use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub target: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Command {
    pub fn execute(&self) -> Result<std::process::Child> {
        let child = std::process::Command::new(&self.target)
            .args(&self.args)
            .spawn()?;

        Ok(child)
    }
}
