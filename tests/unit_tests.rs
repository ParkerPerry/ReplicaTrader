#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[tokio::test]
    async fn test_execute_trade_success() {
        let binance_client = BinanceApi::new("mock_api_key".to_string(), "mock_secret_key".to_string());
        let result = binance_client
            .place_order("BTCUSDT", "BUY", "LIMIT", 1.0, 20000.0)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_listener() {
        let result = block_on(listen_to_events());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trade_failure_logging() {
        let binance_client = BinanceApi::new("mock_api_key".to_string(), "mock_secret_key".to_string());
        let reason = "Insufficient liquidity";
        let log_result = binance_client
            .log_trade_failure("0xMockUser", reason)
            .await;
        assert!(log_result.is_ok());
    }
}