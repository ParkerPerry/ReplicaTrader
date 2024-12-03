use reqwest::{Client, Error};
use serde_json::Value;
use std::collections::HashMap;

/// Structure for the Binance API client
pub struct BinanceApi {
    client: Client,
    api_key: String,
    secret_key: String,
    base_url: String,
}

impl BinanceApi {
    /// Constructor for BinanceApi
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            secret_key,
            base_url: "https://api.binance.com".to_string(),
        }
    }

    // ------------------------
    // Order Management
    // ------------------------

    /// Place a LIMIT order on Binance
    pub async fn place_order(
        &self,
        symbol: &str,
        side: &str,
        order_type: &str,
        quantity: f64,
        price: f64,
    ) -> Result<Value, Error> {
        let endpoint = format!("{}/api/v3/order", self.base_url);
        let mut params = HashMap::new();

        params.insert("symbol", symbol);
        params.insert("side", side);
        params.insert("type", order_type);
        params.insert("timeInForce", "GTC");
        params.insert("quantity", &quantity.to_string());
        params.insert("price", &price.to_string());
        params.insert("timestamp", &self.get_timestamp().to_string());

        let signature = self.sign_request(&params);
        params.insert("signature", &signature);

        let response = self
            .client
            .post(&endpoint)
            .header("X-MBX-APIKEY", &self.api_key)
            .form(&params)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        Ok(response_body)
    }

    /// Cancel an existing order on Binance
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<Value, Error> {
        let endpoint = format!("{}/api/v3/order", self.base_url);
        let mut params = HashMap::new();

        params.insert("symbol", symbol);
        params.insert("orderId", order_id);
        params.insert("timestamp", &self.get_timestamp().to_string());

        let signature = self.sign_request(&params);
        params.insert("signature", &signature);

        let response = self
            .client
            .delete(&endpoint)
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&params)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        Ok(response_body)
    }

    // ------------------------
    // Market Data
    // ------------------------

    /// Get the current price for a symbol
    pub async fn get_price(&self, symbol: &str) -> Result<f64, Error> {
        let endpoint = format!("{}/api/v3/ticker/price", self.base_url);
        let response = self
            .client
            .get(&endpoint)
            .query(&[("symbol", symbol)])
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        let price = response_body["price"]
            .as_str()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        Ok(price)
    }

    /// Get order book depth for a symbol
    pub async fn get_order_book(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Value, Error> {
        let endpoint = format!("{}/api/v3/depth", self.base_url);
        let response = self
            .client
            .get(&endpoint)
            .query(&[("symbol", symbol), ("limit", &limit.to_string())])
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        Ok(response_body)
    }

    /// Get historical trades for a symbol
    pub async fn get_historical_trades(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Value, Error> {
        let endpoint = format!("{}/api/v3/historicalTrades", self.base_url);
        let response = self
            .client
            .get(&endpoint)
            .query(&[("symbol", symbol), ("limit", &limit.to_string())])
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        Ok(response_body)
    }

    // ------------------------
    // Account Management
    // ------------------------

    /// Get account balances
    pub async fn get_account_balances(&self) -> Result<Value, Error> {
        let endpoint = format!("{}/api/v3/account", self.base_url);
        let mut params = HashMap::new();
        params.insert("timestamp", &self.get_timestamp().to_string());

        let signature = self.sign_request(&params);
        params.insert("signature", &signature);

        let response = self
            .client
            .get(&endpoint)
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&params)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        Ok(response_body)
    }

    /// Get open orders for a specific symbol
    pub async fn get_open_orders(&self, symbol: &str) -> Result<Value, Error> {
        let endpoint = format!("{}/api/v3/openOrders", self.base_url);
        let mut params = HashMap::new();

        params.insert("symbol", symbol);
        params.insert("timestamp", &self.get_timestamp().to_string());

        let signature = self.sign_request(&params);
        params.insert("signature", &signature);

        let response = self
            .client
            .get(&endpoint)
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&params)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        Ok(response_body)
    }

    // ------------------------
    // Error Handling & Reconciliation
    // ------------------------

    /// Log trade failures for reconciliation
    pub fn log_trade_failure(&self, user: &str, reason: &str) {
        eprintln!("Trade failed for user {}: {}", user, reason);
        // Store to a database or file for reconciliation if needed
    }

    /// Retry failed trade
    pub async fn retry_failed_trade(&self, user: &str, symbol: &str, amount: f64, price: f64) -> Result<Value, String> {
        match self.place_order(symbol, "BUY", "LIMIT", amount, price).await {
            Ok(order) => Ok(order),
            Err(e) => Err(format!("Failed to retry trade: {}", e)),
        }
    }

    // ------------------------
    // Helper Functions
    // ------------------------

    /// Helper function to generate a timestamp
    fn get_timestamp(&self) -> u64 {
        let start = std::time::SystemTime::now();
        start
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }

    /// Helper function to sign the request
    fn sign_request(&self, params: &HashMap<&str, &str>) -> String {
        let query_string: String = params
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>()
            .join("&");

        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, self.secret_key.as_bytes());
        let signature = ring::hmac::sign(&key, query_string.as_bytes());
        hex::encode(signature.as_ref())
    }
}