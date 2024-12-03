use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::contract::Contract;
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::binance_api::BinanceApi;

pub async fn listen_to_events() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("https://your-eth-node")?;
    let contract_address = "0xYourContractAddress".parse()?;
    let abi = include_str!("../abi/ReplicaTrader.json");

    let client = Arc::new(provider);
    let contract = Contract::from_json(client.clone(), contract_address, abi.as_bytes())?;
    let binance_api = Arc::new(Mutex::new(BinanceApi::new(
        "your_binance_api_key".to_string(),
        "your_binance_secret_key".to_string(),
    )));

    let mut stream = contract.event::<(Address, U256, U256, U256)>("TradeTriggered").stream().await?;
    while let Some(event) = stream.next().await {
        match event {
            Ok((user, amount, price, _timestamp)) => {
                println!("Trade triggered: User: {}, Amount: {}, Price: {}", user, amount, price);
                let binance_api = binance_api.clone();
                let trade_user = user.clone();

                tokio::spawn(async move {
                    // Validate conditions before executing
                    let slippage_threshold = 0.01; // Example: 1% slippage allowed
                    let stop_loss = 30_000.0;      // Example stop-loss price
                    let take_profit = 40_000.0;    // Example take-profit price

                    // Fetch current price
                    let current_price = binance_api.lock().await.get_price("BTCUSDT").await.unwrap_or(0.0);

                    // Check slippage
                    let price_difference = (current_price - price.as_u64() as f64).abs();
                    if price_difference / price.as_u64() as f64 > slippage_threshold {
                        println!("Trade failed due to excessive slippage for user: {}", trade_user);
                        return;
                    }

                    // Check stop-loss and take-profit
                    if current_price < stop_loss || current_price > take_profit {
                        println!(
                            "Trade failed for user: {} due to stop-loss or take-profit violation.",
                            trade_user
                        );
                        return;
                    }

                    // Execute trade via Binance API
                    let result = binance_api.lock().await.place_order(
                        "BTCUSDT",
                        "BUY",
                        "LIMIT",
                        amount.as_u64() as f64,
                        price.as_u64() as f64,
                    ).await;

                    match result {
                        Ok(order_id) => println!("Trade successful for user: {} with Order ID: {}", trade_user, order_id),
                        Err(e) => {
                            println!("Trade execution failed for user: {}: {:?}", trade_user, e);
                            // Log failure or attempt reconciliation
                        }
                    }
                });
            }
            Err(e) => eprintln!("Failed to parse event: {:?}", e),
        }
    }

    Ok(())
}