mod command;
mod configuration;

use std::{
    collections::HashMap,
    path::PathBuf,
    process::Child,
    sync::{LazyLock, RwLock},
};

use clap::{Parser, command};
use configuration::Config;
use rocket::serde::json::Json;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub configuration: Option<PathBuf>,
}

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| RwLock::new(Default::default()));
static RUNNING_JOBS: LazyLock<RwLock<HashMap<u32, Child>>> =
    LazyLock::new(|| RwLock::new(Default::default()));

#[macro_use]
extern crate rocket;

#[get("/")]
fn list_commands() -> Json<Vec<String>> {
    let commands = CONFIG
        .read()
        .unwrap()
        .commands
        .clone()
        .into_keys()
        .collect();

    Json(commands)
}

fn get_status_inner(command_id: u32) -> Result<command::Status, String> {
    let mut jobs = RUNNING_JOBS.write().unwrap();
    let handle = jobs
        .get_mut(&command_id)
        .ok_or("no such command".to_owned())?;

    let status = match handle.try_wait().map_err(|err| err.to_string())? {
        Some(exit_status) => command::Status::Exited(exit_status.to_string()),
        None => command::Status::Running,
    };

    Ok(status)
}

#[get("/status/<command_id>")]
fn get_status(command_id: u32) -> Json<Result<command::Status, String>> {
    Json(get_status_inner(command_id))
}

fn execute_inner(command_name: &str) -> Result<u32, String> {
    let cfg = CONFIG.read().unwrap();
    let command = cfg
        .commands
        .get(command_name)
        .ok_or("no such command".to_owned())?;

    let child = command.execute().map_err(|err| err.to_string())?;
    let id = child.id();
    RUNNING_JOBS.write().unwrap().insert(id, child);

    Ok(id)
}

#[post("/run/<command_name>")]
fn execute(command_name: &str) -> Json<Result<u32, String>> {
    Json(execute_inner(command_name))
}

#[launch]
fn rocket() -> _ {
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .env()
        .init()
        .unwrap();

    let args = Args::parse();
    let config_file_path = args
        .configuration
        .unwrap_or_else(configuration::default_path);

    log::info!(
        "resolved config location is {}",
        config_file_path.as_os_str().to_string_lossy()
    );

    let config_contents =
        std::fs::read_to_string(config_file_path).expect("configuration file should be readable");

    let config = configuration::parse(&config_contents)
        .expect("configuration file contents should be valid");
    *CONFIG.write().unwrap() = config;

    rocket::build().mount("/", routes![list_commands, get_status, execute])
}
