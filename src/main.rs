mod command;
mod configuration;
mod dashboard;

use std::{path::PathBuf, sync::RwLock};

use clap::{Parser, command};
use command::Registry;
use configuration::Config;
use rocket::serde::json::Json;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub configuration: Option<PathBuf>,
}

#[macro_use]
extern crate rocket;

#[get("/")]
fn list_commands(config: &rocket::State<Config>) -> Json<Vec<String>> {
    let commands = config.commands.keys().cloned().collect();

    Json(commands)
}

fn get_status_inner(
    command_name: &str,
    cmd_reg: &rocket::State<RwLock<command::Registry>>,
) -> Result<command::Status, String> {
    let name = command_name.to_owned();
    let mut registry = cmd_reg.write().unwrap();

    registry.get_latest_status(name)
}

#[get("/status/<command_name>")]
fn get_status(
    command_name: &str,
    cmd_reg: &rocket::State<RwLock<command::Registry>>,
) -> Json<Result<command::Status, String>> {
    Json(get_status_inner(command_name, cmd_reg))
}

fn execute_inner(
    command_name: &str,
    config: &rocket::State<Config>,
    cmd_reg: &rocket::State<RwLock<command::Registry>>,
) -> Result<(), String> {
    let command = config
        .commands
        .get(command_name)
        .ok_or("no such command".to_owned())?;

    let process = command.execute().map_err(|err| err.to_string())?;

    let mut registry = cmd_reg.write().unwrap();
    registry.register_new_process(command_name.to_owned(), process);

    Ok(())
}

#[post("/run/<command_name>")]
fn execute(
    command_name: &str,
    config: &rocket::State<Config>,
    cmd_reg: &rocket::State<RwLock<command::Registry>>,
) -> Json<Result<(), String>> {
    Json(execute_inner(command_name, config, cmd_reg))
}

#[post("/kill/<command_name>")]
fn kill(
    command_name: &str,
    cmd_reg: &rocket::State<RwLock<command::Registry>>,
) -> Json<Result<(), String>> {
    Json(cmd_reg.write().unwrap().kill(command_name.to_owned()))
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

    rocket::build()
        .manage(RwLock::new(Registry::new(&config)))
        .manage(config)
        .mount("/", routes![list_commands, get_status, execute, kill])
        .mount("/dashboard", routes![dashboard::render])
}
