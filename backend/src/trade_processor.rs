use crate::binance_api::BinanceApi;
use sqlx::{Pool, Sqlite};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// TradeProcessor handles liquidity checks, risk validation, and trade execution
pub struct TradeProcessor {
    binance_client: Arc<Mutex<BinanceApi>>,
    db_pool: Pool<Sqlite>,
}

impl TradeProcessor {
    /// Creates a new TradeProcessor instance
    pub fn new(binance_client: Arc<Mutex<BinanceApi>>, db_pool: Pool<Sqlite>) -> Self {
        Self {
            binance_client,
            db_pool,
        }
    }

    /// Executes a trade with validations, including liquidity, slippage, stop-loss, and take-profit checks
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
        let current_price = self
            .binance_client
            .lock()
            .await
            .get_price(symbol)
            .await
            .map_err(|e| format!("Failed to fetch price: {}", e))?;

        let allowed_slippage = slippage * price;
        if (current_price - price).abs() > allowed_slippage {
            return Err(format!(
                "Slippage too high: current price {}, target price {}, allowed slippage {}",
                current_price, price, allowed_slippage
            ));
        }

        let order_book = self
            .binance_client
            .lock()
            .await
            .get_order_book(symbol, 10)
            .await
            .map_err(|e| format!("Failed to fetch order book: {}", e))?;

        let bids = order_book["bids"]
            .as_array()
            .ok_or("Failed to parse order book bids")?;

        let mut available_liquidity = 0.0;
        for bid in bids {
            let bid_price = bid[0].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let bid_volume = bid[1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            if bid_price >= price {
                available_liquidity += bid_volume;
                if available_liquidity >= amount {
                    break;
                }
            }
        }

        if available_liquidity < amount {
            return Err("Insufficient liquidity".to_string());
        }

        if current_price < stop_loss {
            return Err(format!(
                "Price {} is below stop-loss {}. Trade rejected.",
                current_price, stop_loss
            ));
        }

        if current_price > take_profit {
            return Err(format!(
                "Price {} exceeds take-profit {}. Trade rejected.",
                current_price, take_profit
            ));
        }

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

    /// Logs successful trades to the database
    async fn log_successful_trade(
        &self,
        user: &str,
        symbol: &str,
        amount: f64,
        price: f64,
    ) {
        if let Err(e) = sqlx::query!(
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
            eprintln!(
                "Failed to log successful trade for user {} with error: {}",
                user, e
            );
        }
    }

    /// Logs failed trades for future reconciliation
    async fn log_failed_trade(
        &self,
        user: &str,
        symbol: &str,
        amount: f64,
        price: f64,
        reason: String,
    ) {
        if let Err(e) = sqlx::query!(
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
            eprintln!(
                "Failed to log trade failure for user {} with error: {}",
                user, e
            );
        }
    }
}