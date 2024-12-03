use dotenv::dotenv;
use sqlx::SqlitePool;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

mod binance_api;
mod event_listener;
mod trade_processor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize SQLite database pool
    let db_pool = SqlitePool::connect(env::var("DATABASE_URL").unwrap().as_str()).await?;

    // Read Binance API credentials
    let api_key = env::var("BINANCE_API_KEY").expect("Missing BINANCE_API_KEY");
    let secret_key = env::var("BINANCE_SECRET_KEY").expect("Missing BINANCE_SECRET_KEY");

    // Initialize Binance API client
    let binance_client = Arc::new(Mutex::new(binance_api::BinanceApi::new(api_key, secret_key)));

    // Create TradeProcessor
    let trade_processor = trade_processor::TradeProcessor::new(binance_client.clone(), db_pool.clone());

    // Start listening to events and process trades
    if let Err(e) = event_listener::listen_to_events(trade_processor).await {
        eprintln!("Error in event listener: {}", e);
    }

    Ok(())
}