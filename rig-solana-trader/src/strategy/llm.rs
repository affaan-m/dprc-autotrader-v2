use crate::database::{Database, TokenState, TradeRecord};
use crate::market_data::birdeye::BirdEyeProvider;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDecision {
    pub action: String,  // "buy", "sell", or "hold"
    pub quantity: Option<f64>,
    pub reasoning: String,
    pub confidence: f64,
    pub risk_assessment: String,
}

pub struct LLMStrategy {
    db: Arc<Database>,
    birdeye: Arc<BirdEyeProvider>,
    openai_client: reqwest::Client,
    api_key: String,
}

impl LLMStrategy {
    pub fn new(db: Arc<Database>, birdeye: Arc<BirdEyeProvider>, openai_api_key: String) -> Self {
        Self {
            db,
            birdeye,
            openai_client: reqwest::Client::new(),
            api_key: openai_api_key,
        }
    }

    pub async fn analyze_trading_opportunity(&self, token_address: &str) -> Result<TradingDecision> {
        // Gather comprehensive data for analysis
        let token_history = self.db.get_token_history(token_address, 24).await?;
        let recent_trades = self.db.get_token_trades(token_address, 10).await?;
        let market_data = self.birdeye.get_multi_token_data(&[token_address.to_string()]).await?;
        let momentum = self.birdeye.get_token_momentum(token_address).await;
        
        // Format data for LLM consumption
        let prompt = self.format_analysis_prompt(
            token_address,
            &token_history,
            &recent_trades,
            &market_data,
            momentum,
        ).await?;

        // Get LLM decision
        let decision = self.get_llm_decision(&prompt).await?;
        
        Ok(decision)
    }

    async fn format_analysis_prompt(
        &self,
        token_address: &str,
        history: &[TokenState],
        trades: &[TradeRecord],
        market_data: &HashMap<String, TradeData>,
        momentum: Option<f64>,
    ) -> Result<String> {
        let mut prompt = String::new();
        
        // Add token metadata
        if let Some(latest) = history.last() {
            prompt.push_str(&format!(
                "Token Analysis for {} ({}):\n\n",
                latest.name, latest.symbol
            ));
        }

        // Add current market data
        if let Some(data) = market_data.get(token_address) {
            prompt.push_str(&format!(
                "Current Market Data:\n\
                - Price: ${:.6}\n\
                - 24h Volume: ${:.2}\n\
                - Market Cap: ${:.2}\n\
                - 24h Price Change: {:.2}%\n\
                - Liquidity: ${:.2}\n\n",
                data.price,
                data.volume24h,
                data.mc.unwrap_or(0.0),
                data.priceChange24h.unwrap_or(0.0),
                data.liquidity,
            ));
        }

        // Add price history summary
        if !history.is_empty() {
            let price_changes: Vec<f64> = history.windows(2)
                .map(|w| (w[1].price_usd - w[0].price_usd) / w[0].price_usd * 100.0)
                .collect();

            let avg_price_change = price_changes.iter().sum::<f64>() / price_changes.len() as f64;
            let max_price = history.iter().map(|h| h.price_usd).fold(0.0, f64::max);
            let min_price = history.iter().map(|h| h.price_usd).fold(f64::INFINITY, f64::min);

            prompt.push_str(&format!(
                "Price History (24h):\n\
                - Average Price Change: {:.2}%\n\
                - Highest Price: ${:.6}\n\
                - Lowest Price: ${:.6}\n\
                - Price Volatility: {:.2}%\n\n",
                avg_price_change,
                max_price,
                min_price,
                (max_price - min_price) / min_price * 100.0,
            ));
        }

        // Add momentum analysis
        if let Some(m) = momentum {
            prompt.push_str(&format!(
                "Momentum Analysis:\n\
                - Current Momentum Score: {:.4}\n\
                - Interpretation: {}\n\n",
                m,
                if m > 0.1 {
                    "Strong Upward"
                } else if m > 0.05 {
                    "Moderate Upward"
                } else if m > -0.05 {
                    "Neutral"
                } else if m > -0.1 {
                    "Moderate Downward"
                } else {
                    "Strong Downward"
                }
            ));
        }

        // Add recent trades
        if !trades.is_empty() {
            prompt.push_str("Recent Trading History:\n");
            for trade in trades.iter().take(5) {
                prompt.push_str(&format!(
                    "- {} at ${:.6} ({:.2} units) - Confidence: {:.2}\n",
                    trade.action,
                    trade.price,
                    trade.quantity,
                    trade.confidence,
                ));
            }
            prompt.push_str("\n");
        }

        // Add decision request
        prompt.push_str(
            "Based on the above data, please analyze the trading opportunity and provide:\n\
            1. Recommended action (buy, sell, or hold)\n\
            2. Suggested quantity (if buy/sell)\n\
            3. Detailed reasoning\n\
            4. Confidence level (0.0-1.0)\n\
            5. Risk assessment\n\n\
            Please format your response as a JSON object with the following structure:\n\
            {\n\
                \"action\": \"buy|sell|hold\",\n\
                \"quantity\": null or number,\n\
                \"reasoning\": \"detailed explanation\",\n\
                \"confidence\": number,\n\
                \"risk_assessment\": \"risk level and explanation\"\n\
            }\n"
        );

        Ok(prompt)
    }

    async fn get_llm_decision(&self, prompt: &str) -> Result<TradingDecision> {
        let response = self.openai_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "model": "gpt-4-turbo-preview",
                "messages": [
                    {
                        "role": "system",
                        "content": "You are an expert cryptocurrency trader focused on analyzing Solana tokens. \
                                  You make careful, data-driven decisions while managing risk. Your responses \
                                  should be in valid JSON format as specified in the prompt."
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": 0.3,
                "response_format": { "type": "json_object" }
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        let decision: TradingDecision = serde_json::from_str(content)?;
        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_llm_strategy() {
        dotenv().ok();
        
        let db = Arc::new(
            Database::new(&env::var("MONGODB_URI").unwrap())
                .await
                .unwrap()
        );

        let birdeye = Arc::new(
            BirdEyeProvider::new(env::var("BIRDEYE_API_KEY").unwrap())
        );

        let strategy = LLMStrategy::new(
            db,
            birdeye,
            env::var("OPENAI_API_KEY").unwrap(),
        );

        // Test with USDC
        let decision = strategy
            .analyze_trading_opportunity("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
            .await
            .unwrap();

        assert!(["buy", "sell", "hold"].contains(&decision.action.as_str()));
        assert!(decision.confidence >= 0.0 && decision.confidence <= 1.0);
        assert!(!decision.reasoning.is_empty());
        assert!(!decision.risk_assessment.is_empty());
    }
} 