#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[tokio::test]
    async fn test_execute_trade_success() {
        // Setup: Mock Binance API and database
        let binance_client = BinanceApi::new("mock_api_key".to_string(), "mock_secret_key".to_string());
        let result = binance_client
            .place_order("BTCUSDT", "BUY", "LIMIT", 1.0, 20000.0)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_listener() {
        // Test the event listener
        let result = block_on(listen_to_events());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_failed_trade_logging() {
        // Simulate logging a failed trade
        log_trade_failure("0xMockAddress", "Insufficient funds");
        // Check if the log was correctly added to the database (mock or real)
    }
}