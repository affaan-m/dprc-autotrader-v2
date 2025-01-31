use rig_core::agent::Agent;
use rig::completion::{CompletionModel, Prompt};
use anyhow::Result;
use std::collections::HashSet;
use std::time::Duration;
use std::collections::HashMap;
use thiserror::Error;
use crate::market_data::{MarketData, MarketContext};
use solana_sdk::nonce::State;

pub struct StoicPersonality {
    allowed_interactions: HashSet<String>,
    base_prompt: String,
    max_position_size: f64,
    risk_tolerance: f64,
    trade_cooldown: Duration,
    technical_indicators: Vec<String>,
    market_context: HashMap<String, f64>,
    agent: Agent<dyn CompletionModel>,
}

#[derive(Error, Debug)]
pub enum StoicPersonalityError {
    #[error("Risk tolerance exceeded maximum allowed value")]
    RiskToleranceExceeded,
    #[error("Invalid position size: {0}")]
    InvalidPositionSize(f64),
    #[error("Market data incomplete: {0}")]
    IncompleteMarketData(String),
    #[error("LLM response validation failed: {0}")]
    ResponseValidation(String),
}

impl StoicPersonality {
    pub fn new() -> Self {
        Self {
            allowed_interactions: HashSet::new(),
            base_prompt: r#"You are a stoic trading bot. Your responses should reflect stoic principles:
1. Emotional detachment from market movements
2. Focus on rational decision making based on data
3. Acceptance of market conditions
4. Long-term value perspective
5. Risk management emphasis

When tweeting about trades:
1. Always include exact amounts (e.g. "Bought 0.5 SOL worth of $TICKER")
2. Include market cap ("MC: $xxxM")
3. Always include contract address ("CA: address")
4. Always include Solscan transaction link
5. End with a stoic analysis based on actual market indicators:
   - Volume trends
   - Price action
   - Market depth
   - Social sentiment
   - Development activity"#.to_string(),
            max_position_size: 1.0,
            risk_tolerance: 0.2,
            trade_cooldown: Duration::from_secs(300),
            technical_indicators: vec![
                "RSI".into(),
                "MACD".into(),
                "Volume".into()
            ],
            market_context: HashMap::new(),
            agent: Agent::new(rig_core::providers::openai::Client::from_env()),
        }
    }

    pub fn add_allowed_interaction(&mut self, twitter_handle: String) {
        self.allowed_interactions.insert(twitter_handle);
    }

    pub fn is_interaction_allowed(&self, twitter_handle: &str) -> bool {
        self.allowed_interactions.contains(twitter_handle)
    }

    pub async fn generate_trade_tweet<M: CompletionModel>(
        &self,
        agent: &Agent<M>,
        trade_details: &str,
        market_data: &MarketData,
    ) -> Result<String> {
        let prompt = format!(
            r#"{}

Generate a tweet about this trade using the following template:
[BUY/SELL] {:.2} SOL worth of {}
MC: ${:.2}M | Risk: {:.1}% | Vol: {:.2}%
CA: <contract_address>
🔍 https://solscan.io/tx/<tx_id>

Technical Indicators:
{}

Market Context:
{}

[Stoic Analysis]
{}

Trade details:
{}

Requirements:
1. Use exact numbers from the trade details
2. Include all template fields
3. Keep stoic analysis focused on actual market data
4. Stay under 280 characters
5. Use cashtags for token symbols"#,
            self.base_prompt,
            self.max_position_size,
            market_data.market_cap / 1_000_000.0,
            self.risk_tolerance * 100.0,
            market_data.volatility * 100.0,
            self.technical_indicators.join("\n"),
            self.format_market_context(),
            trade_details
        );

        let response = agent.prompt(&prompt).await?;
        Ok(self.postprocess_tweet(response))
    }

    fn postprocess_tweet(&self, tweet: String) -> String {
        let mut processed = tweet.trim().to_string();
        
        if !processed.contains("#StoicTrading") {
            processed.push_str("\n\n#StoicTrading #Solana #AlgoTrading");
        }
        
        processed.chars().take(280).collect()
    }

    pub async fn generate_reply<M: CompletionModel>(
        &self,
        agent: &Agent<M>,
        tweet_text: &str,
        author: &str,
        market_context: &MarketContext,
    ) -> Result<Option<String>> {
        if !self.is_interaction_allowed(author) {
            return Ok(None);
        }

        let prompt = format!(
            r#"{}

Respond to this tweet considering current market conditions:
Market Trend: {}
Sector Performance: {:.2}%
Sentiment Score: {:.2}

Tweet to respond to:
{}

Requirements:
1. Only respond if the tweet warrants a response
2. Be helpful but maintain stoic detachment
3. Focus on data-driven insights from these indicators: {}
4. Never give financial advice
5. Stay under 280 characters"#,
            self.base_prompt,
            market_context.market_trend,
            market_context.sector_performance,
            market_context.sentiment_score,
            tweet_text,
            self.technical_indicators.join(", ")
        );

        let response = agent.prompt(&prompt).await?;
        if response.trim().is_empty() || response.to_lowercase().contains("no response") {
            Ok(None)
        } else {
            Ok(Some(response.to_string()))
        }
    }

    pub fn with_max_position_size(mut self, size: f64) -> Self {
        assert!(size > 0.0, "Position size must be positive");
        self.max_position_size = size;
        self
    }

    pub fn with_risk_tolerance(mut self, tolerance: f64) -> Self {
        self.risk_tolerance = tolerance.clamp(0.0, 1.0);
        self
    }

    pub fn with_technical_indicators(mut self, indicators: Vec<String>) -> Self {
        self.technical_indicators = indicators;
        self
    }

    fn format_market_context(&self) -> String {
        self.market_context
            .iter()
            .map(|(k, v)| format!("{}: {:.2}", k, v))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub async fn analyze_state(&self, state: &solana_sdk::nonce::State) -> Analysis {
        let prompt = format!("{} Analyze market state:\n{}", 
            self.base_prompt,
            state.to_markdown()
        );
        
        self.agent.prompt(&prompt)
            .await
            .parse()
            .unwrap_or(Analysis::Hold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[test]
    fn test_personality_defaults() {
        let personality = StoicPersonality::default();
        assert!(personality.base_prompt.contains("stoic"));
        assert!(personality.allowed_interactions.is_empty());
    }

    #[test]
    fn test_allowed_interactions() {
        let mut personality = StoicPersonality::new();
        personality.add_allowed_interaction("vitalik".to_string());
        assert!(personality.is_interaction_allowed("vitalik"));
        assert!(!personality.is_interaction_allowed("random_user"));
    }

    #[test]
    fn test_configuration() {
        let personality = StoicPersonality::new()
            .with_max_position_size(2.5)
            .with_risk_tolerance(0.3)
            .with_technical_indicators(vec!["EMA".into(), "OBV".into()]);
        
        assert_eq!(personality.max_position_size, 2.5);
        assert_eq!(personality.risk_tolerance, 0.3);
        assert_eq!(personality.technical_indicators, vec!["EMA", "OBV"]);
    }

    #[tokio::test]
    async fn test_tweet_generation() {
        let personality = StoicPersonality::new();
        let mock_agent = Agent::new(MockCompletionModel::default());
        let market_data = MarketData {
            market_cap: 50_000_000.0,
            volatility: 0.15,
            // ... other fields ...
        };
        
        let tweet = personality
            .generate_trade_tweet(&mock_agent, "Test trade", &market_data)
            .await
            .unwrap();
        
        assert!(tweet.contains("#StoicTrading"));
        assert!(tweet.len() <= 280);
    }

    #[test]
    fn test_market_context_formatting() {
        let mut personality = StoicPersonality::new();
        personality.market_context.insert("Liquidity".into(), 1.5);
        personality.market_context.insert("Funding Rate".into(), -0.02);
        
        let formatted = personality.format_market_context();
        assert!(formatted.contains("Liquidity: 1.50"));
        assert!(formatted.contains("Funding Rate: -0.02"));
    }
} 