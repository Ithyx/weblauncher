mod command;
mod configuration;

use std::{
    path::PathBuf,
    sync::{LazyLock, RwLock},
};

use clap::{Parser, command};
use configuration::Config;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub configuration: Option<PathBuf>,
}

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| RwLock::new(Default::default()));

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> String {
    let commands = &CONFIG.read().unwrap().commands;

    format!("available commands: {commands:?}")
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

    rocket::build().mount("/", routes![index])
}
