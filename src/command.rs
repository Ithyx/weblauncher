use std::{
    collections::HashMap,
    process::{Child, ExitStatus},
};

use anyhow::Result;
use serde::{Deserialize, Serialize, ser::SerializeStructVariant};

use crate::configuration::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    NotStarted,
    Running,
    Exited(ExitStatus),
}

impl Serialize for Status {
    fn serialize<S>(&self, s: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Status::NotStarted => {
                let s = s.serialize_struct_variant("Status", 0, "NotStarted", 0)?;
                s.end()
            }
            Status::Running => {
                let s = s.serialize_struct_variant("Status", 1, "Running", 0)?;
                s.end()
            }
            Status::Exited(exit_status) => {
                let mut s = s.serialize_struct_variant("Status", 2, "Exited", 1)?;
                s.serialize_field("exit status", &exit_status.to_string())?;
                s.end()
            }
        }
    }
}

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

#[derive(Debug)]
pub struct Registry {
    statuses: HashMap<String, Status>,
    process_handles: HashMap<String, Option<Child>>,
}

impl Registry {
    pub fn new(config: &Config) -> Self {
        let statuses = config
            .commands
            .keys()
            .map(|name| (name.clone(), Status::NotStarted))
            .collect();
        let process_handles = config
            .commands
            .keys()
            .map(|name| (name.clone(), None))
            .collect();

        Self {
            statuses,
            process_handles,
        }
    }

    pub fn get_latest_status(&mut self, command_name: String) -> Result<Status, String> {
        let last_status = self
            .statuses
            .get_mut(&command_name)
            .ok_or("no such command")?;

        match last_status {
            Status::Running => {
                let process = self
                    .process_handles
                    .get_mut(&command_name)
                    .ok_or("no such running command")?
                    .as_mut()
                    .ok_or("command not running")?;

                match process.try_wait().map_err(|err| err.to_string())? {
                    Some(exit_status) => {
                        let new_status = Status::Exited(exit_status);
                        *last_status = new_status.clone();

                        self.process_handles.remove(&command_name);

                        Ok(new_status)
                    }
                    None => Ok(last_status.clone()),
                }
            }
            _ => Ok(last_status.clone()),
        }
    }

    pub fn register_new_process(&mut self, command_name: String, process: Child) {
        self.statuses.insert(command_name.clone(), Status::Running);
        self.process_handles.insert(command_name, Some(process));
    }

    pub fn kill(&mut self, command_name: String) -> Result<(), String> {
        let latest_status = self.get_latest_status(command_name.clone())?;
        if latest_status != Status::Running {
            return Err("command not running".to_owned());
        }

        let process = self
            .process_handles
            .get_mut(&command_name)
            .ok_or("no such running command")?
            .as_mut()
            .ok_or("command not running")?;

        process.kill().map_err(|err| err.to_string())?;

        Ok(())
    }
}
