mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use polymarket_client_sdk::gamma;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;

#[derive(Parser)]
#[command(name = "polyterm", version, about = "The Bloomberg terminal for Polymarket")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Plain CLI listing of top markets (for scripting)
    Markets {
        #[arg(short, long, default_value_t = 5)]
        limit: i32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => tui::run().await,
        Some(Commands::Markets { limit }) => list_markets(limit).await,
    }
}

async fn list_markets(limit: i32) -> Result<()> {
    let client = gamma::Client::default();
    let req = MarketsRequest::builder().limit(limit).build();
    let markets = client.markets(&req).await?;

    println!("Fetched {} market(s)\n", markets.len());
    for m in &markets {
        println!("{:#?}\n", m);
    }
    Ok(())
}
