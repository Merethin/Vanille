use std::{error::Error, fs, process::exit};
use vanille_graph::delivery_time::generate_delivery_time_plot;
use vanille_models::report::ReportEntry;
use dateparser::parse;
use clap::Parser;
use serenity::all::ChannelId;

#[derive(Parser)]
struct Cli {
    region: String,
    queue: u64,
    start: String,
    end: String,
    out: String,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    dotenv::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|err| {
        eprintln!("Missing DATABASE_URL environment variable: {err}");
        exit(1);
    });

    let pool = sqlx::PgPool::connect(&db_url).await.unwrap_or_else(|err| {
        eprintln!("Error connecting to Postgres: {}", err);
        exit(1);
    });

    let start = parse(&cli.start).map(|v| v.timestamp() as u64).ok();
    let end = parse(&cli.end).map(|v| v.timestamp() as u64).ok();

    let mut range = None;
    if let Some(start) = start && let Some(end) = end {
        range = Some((start, end));
    }

    let entries = ReportEntry::query(&pool, ChannelId::new(cli.queue), range).await?;

    let bytes = generate_delivery_time_plot(&cli.region, &entries).await?;
    fs::write(cli.out, bytes)?;

    Ok(())
}