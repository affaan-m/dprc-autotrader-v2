use rig::agent::Agent;
use rig::completion::{CompletionModel, Prompt};
use anyhow::Result;
use std::collections::HashSet;

pub struct StoicPersonality {
    allowed_interactions: HashSet<String>,
    base_prompt: String,
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
    ) -> Result<String> {
        let prompt = format!(
            r#"{}

Generate a tweet about this trade using the following template:
[BUY/SELL] x.xx SOL worth of $TICKER
MC: $xxxM
CA: <contract_address>
🔍 https://solscan.io/tx/<tx_id>

[Stoic analysis based on actual market indicators]

Trade details:
{}

Requirements:
1. Use exact numbers from the trade details
2. Include all template fields
3. Keep stoic analysis focused on actual market data
4. Stay under 280 characters
5. Use cashtags for token symbols"#,
            self.base_prompt,
            trade_details
        );

        let response = agent.prompt(&prompt).await?;
        Ok(response.to_string())
    }

    pub async fn generate_reply<M: CompletionModel>(
        &self,
        agent: &Agent<M>,
        tweet_text: &str,
        author: &str,
    ) -> Result<Option<String>> {
        if !self.is_interaction_allowed(author) {
            return Ok(None);
        }

        let prompt = format!(
            r#"{}

Respond to this tweet:
{}

Requirements:
1. Only respond if the tweet warrants a response
2. Be helpful but maintain stoic detachment
3. Focus on data-driven insights
4. Never give financial advice
5. Stay under 280 characters"#,
            self.base_prompt,
            tweet_text
        );

        let response = agent.prompt(&prompt).await?;
        if response.trim().is_empty() || response.to_lowercase().contains("no response") {
            Ok(None)
        } else {
            Ok(Some(response.to_string()))
        }
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
} 