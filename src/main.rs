mod commands;
mod tasks;
mod interactions;
mod models;
mod bot;
mod config;
mod embeds;
mod api;

use caramel::ns::api::Client;
use log::{warn, error};
use std::{process::exit, error::Error};
use config_file::FromConfigFile;

use caramel::{log::setup_log, ns::UserAgent};

use crate::config::Config;
use crate::bot::start_client;

const PROGRAM: &str = "Vanille";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Merethin";
const CONFIG_PATH: &'static str = "config/vanille.toml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_log(vec!["serenity", "poise", "lapin"]);

    dotenv::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        error!("Missing discord token, please provide it in the environment or .env file (as DISCORD_TOKEN)");
        exit(1);
    });

    let user_agent = UserAgent::read_from_env(PROGRAM, VERSION, AUTHOR);

    let config = Config::from_config_file(CONFIG_PATH).unwrap_or_else(|e| {
        warn!("Failed to load config file: {} - loading default values", e);
        config::Config::default()
    });

    let conn = lapin::Connection::connect(
        &config.input.url,
        lapin::ConnectionProperties::default(),
    ).await?;

    let channel = conn.create_channel().await?;
    let pool = sqlx::PgPool::connect(&config.database.url).await.unwrap_or_else(|err| {
        error!("Error connecting to Postgres: {}", err);
        exit(1);
    });

    let api_client = Client::new(user_agent.clone())?;

    match start_client(token, pool, user_agent, channel, config, api_client).await {
        Ok(_) => {},
        Err(err) => {
            error!("Error in discord client: {}", err);
            exit(1);
        }
    }

    Ok(())
}
