mod command;
mod configuration;
mod dashboard;

use std::{path::PathBuf, sync::RwLock};

use clap::{Parser, command};
use command::Registry;
use configuration::Config;
use rocket::{http::Method, serde::json::Json};
use rocket_cors::{AllowedOrigins, CorsOptions};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub configuration: Option<PathBuf>,
}

#[macro_use]
extern crate rocket;

#[get("/list")]
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

    let mut cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::some_regex(&config.allowed_origins))
        .allowed_methods(
            vec![Method::Get, Method::Post]
                .into_iter()
                .map(From::from)
                .collect(),
        );

    if cfg!(debug_assertions) {
        log::warn!("Launching in debug, opening CORS to all");
        cors = cors.allowed_origins(AllowedOrigins::all());
    }

    let rocket_config = rocket::Config {
        address: config.address.unwrap_or([0, 0, 0, 0].into()),
        port: config.port.unwrap_or(8000),
        ..Default::default()
    };

    rocket::build()
        .configure(rocket_config)
        .attach(cors.to_cors().unwrap())
        .manage(RwLock::new(Registry::new(&config)))
        .manage(config)
        .mount("/api", routes![list_commands, get_status, execute, kill])
        .mount("/", routes![dashboard::render, dashboard::favicon])
}
