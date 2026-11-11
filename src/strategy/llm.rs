use crate::market_data::{birdeye::BirdEyeProvider, DataProvider};
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, instrument};

pub struct LLMStrategy {
    birdeye: Arc<BirdEyeProvider>,
}

impl LLMStrategy {
    pub fn new(birdeye: Arc<BirdEyeProvider>) -> Self {
        Self { birdeye }
    }

    #[instrument(skip(self))]
    pub async fn analyze_trading_opportunity(&self, prompt: &str, sol_balance: f64) -> Result<String> {
        debug!("Analyzing trading opportunity with prompt: {}", prompt);
        
        // Format the analysis with the available SOL balance
        let analysis = format!(
            "Available SOL: {}\n{}",
            sol_balance,
            prompt
        );
        
        Ok(analysis)
    }
} 