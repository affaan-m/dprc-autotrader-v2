use crate::market_data::{birdeye::BirdEyeProvider, DataProvider};
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, instrument};

pub struct LLMStrategy {
    birdeye: Arc<BirdEyeProvider>,
}

#[derive(Debug)]
pub struct TradeData {
    pub price: f64,
    pub volume: f64,
    pub market_cap: f64,
    pub price_change: f64,
}

impl LLMStrategy {
    pub fn new(birdeye: Arc<BirdEyeProvider>) -> Self {
        Self { birdeye }
    }

    #[instrument(skip(self))]
    pub async fn analyze_token(&self, token_address: &str) -> Result<String> {
        debug!("Analyzing token {}", token_address);
        
        // Get token history and market data
        let token_history = self.birdeye.as_ref().get_historical_prices(token_address).await?;
        let market_data = self.birdeye.as_ref().get_token_metadata(token_address).await?;
        
        let prompt = format!(
            "Analyze trading opportunity for token {}:\n\nMarket Data:\n{:#?}\n\nHistory:\n{:#?}",
            token_address,
            market_data,
            token_history,
        );

        Ok(prompt)
    }
} 