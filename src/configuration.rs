use std::{collections::HashMap, net::IpAddr, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

use crate::command::Command;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub address: Option<IpAddr>,
    pub port: Option<u16>,

    #[serde(default)]
    pub allowed_origins: Vec<String>,

    #[serde(default)]
    pub commands: HashMap<String, Command>,
}

pub fn default_path() -> PathBuf {
    let mut path = match dirs::config_dir() {
        Some(mut config_folder) => {
            config_folder.push("weblauncher");
            config_folder
        }
        None => PathBuf::new(),
    };
    path.push("config.toml");

    path
}

pub fn parse(contents: &str) -> Result<Config> {
    let config: Config = toml::from_str(contents)?;

    log::info!("successfully parsed config file");
    log::debug!("available commands are: {:?}", config.commands);

    Ok(config)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        parse(include_str!("../tests/empty.toml")).unwrap();
    }

    #[test]
    fn basic() {
        parse(include_str!("../tests/basic.toml")).unwrap();
    }
}
