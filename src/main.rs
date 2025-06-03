mod command;
mod configuration;

use std::path::PathBuf;

use clap::{Parser, command};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    pub configuration: Option<PathBuf>,
}

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello world !"
}

#[launch]
fn rocket() -> _ {
    let args = Args::parse();
    let config_file_path = args.configuration.unwrap_or(configuration::default_path());
    let config_contents =
        std::fs::read_to_string(config_file_path).expect("configuration file should be readable");

    let config = configuration::parse(&config_contents)
        .expect("configuration file contents should be valid");

    rocket::build().mount("/", routes![index])
}
