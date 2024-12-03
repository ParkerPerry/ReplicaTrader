use crate::binance_api::BinanceApi;
use sqlx::{Pool, Sqlite};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// TradeProcessor is responsible for processing trades and ensuring risk checks.
pub struct TradeProcessor {
    binance_client: Arc<Mutex<BinanceApi>>,
    db_pool: Pool<Sqlite>, // Database connection pool
}

impl TradeProcessor {
    /// Creates a new instance of the TradeProcessor
    pub fn new(binance_client: Arc<Mutex<BinanceApi>>, db_pool: Pool<Sqlite>) -> Self {
        Self {
            binance_client,
            db_pool,
        }
    }

    /// Executes a trade with proper validations and error handling
    pub async fn execute_trade(
        &self,
        user: &str,
        symbol: &str,
        amount: f64,
        price: f64,
        slippage: f64,
        stop_loss: f64,
        take_profit: f64,
    ) -> Result<(), String> {
        // Fetch the current market price from Binance
        let current_price = self
            .binance_client
            .lock()
            .await
            .get_price(symbol)
            .await
            .map_err(|e| format!("Failed to fetch price: {}", e))?;

        // Validate slippage
        let allowed_slippage = slippage * price;
        if (current_price - price).abs() > allowed_slippage {
            return Err(format!(
                "Slippage too high: current price {}, target price {}, allowed slippage {}",
                current_price, price, allowed_slippage
            ));
        }

        // Validate stop-loss and take-profit
        if current_price < stop_loss {
            return Err(format!(
                "Trade rejected: Current price {} is below stop-loss {}",
                current_price, stop_loss
            ));
        }

        if current_price > take_profit {
            return Err(format!(
                "Trade rejected: Current price {} is above take-profit {}",
                current_price, take_profit
            ));
        }

        // Execute the trade on Binance
        let response = self
            .binance_client
            .lock()
            .await
            .place_order(symbol, "BUY", "LIMIT", amount, price)
            .await;

        match response {
            Ok(order) => {
                println!(
                    "Trade executed successfully for user {}: Order details: {:?}",
                    user, order
                );
                self.log_successful_trade(user, symbol, amount, price).await;
                Ok(())
            }
            Err(e) => {
                eprintln!(
                    "Trade execution failed for user {}: {}. Logging for reconciliation.",
                    user, e
                );
                self.log_failed_trade(user, symbol, amount, price, e.to_string())
                    .await;
                Err(format!("Failed to execute trade: {}", e))
            }
        }
    }

    /// Logs successful trade attempts
    async fn log_successful_trade(
        &self,
        user: &str,
        symbol: &str,
        amount: f64,
        price: f64,
    ) {
        match sqlx::query!(
            r#"
            INSERT INTO trades (user_address, symbol, amount, price, status, timestamp)
            VALUES (?1, ?2, ?3, ?4, 'success', strftime('%Y-%m-%d %H:%M:%f', 'now'))
            "#,
            user,
            symbol,
            amount,
            price
        )
        .execute(&self.db_pool)
        .await
        {
            Ok(_) => println!("Trade logged successfully for user {}", user),
            Err(e) => eprintln!("Failed to log successful trade for user {}: {}", user, e),
        }
    }

    /// Logs failed trade attempts for reconciliation
    async fn log_failed_trade(
        &self,
        user: &str,
        symbol: &str,
        amount: f64,
        price: f64,
        reason: String,
    ) {
        match sqlx::query!(
            r#"
            INSERT INTO failed_trades (user_address, symbol, amount, price, reason, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%d %H:%M:%f', 'now'))
            "#,
            user,
            symbol,
            amount,
            price,
            reason
        )
        .execute(&self.db_pool)
        .await
        {
            Ok(_) => println!("Failed trade logged successfully for user {}", user),
            Err(e) => eprintln!("Failed to log trade for user {}: {}", user, e),
        }
    }
}