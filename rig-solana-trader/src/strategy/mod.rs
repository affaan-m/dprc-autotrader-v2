use crate::market_data::TokenMetadata;
use anyhow::Result;
use rig::agent::Agent;
use rig::completion::{CompletionModel, Prompt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeRecommendation {
    pub token_address: String,
    pub action: TradeAction,
    pub amount_in_sol: f64,
    pub reasoning: String,
    pub confidence: f64,
    pub risk_assessment: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
    Hold,
}

pub struct TradingStrategy<M: CompletionModel> {
    pub agent: Agent<M>,
    portfolio: HashMap<String, PortfolioPosition>,
    max_position_sol: f64,
    min_position_sol: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortfolioPosition {
    token: TokenMetadata,
    quantity: f64,
    cost_basis_sol: f64,
    entry_timestamp: i64,
    partial_sells: Vec<PartialSell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartialSell {
    quantity: f64,
    price_sol: f64,
    timestamp: i64,
}

impl<M: CompletionModel> TradingStrategy<M> {
    pub fn new(agent: Agent<M>, max_position_sol: f64, min_position_sol: f64) -> Self {
        Self {
            agent,
            portfolio: HashMap::new(),
            max_position_sol,
            min_position_sol,
        }
    }

    pub async fn analyze_trading_opportunity(&self, prompt: &str) -> Result<String> {
        let response = self.agent.prompt(prompt).await?;
        Ok(response.to_string())
    }

    pub async fn evaluate_sell_opportunity(
        &self,
        position: &PortfolioPosition,
        current_price_sol: f64,
    ) -> Result<Option<TradeRecommendation>> {
        let position_context = serde_json::to_string_pretty(&position)?;
        let profit_loss = (current_price_sol - position.cost_basis_sol) / position.cost_basis_sol * 100.0;

        let prompt = format!(
            r#"Evaluate this position for potential exit or partial sell:

Position Details:
{}

Current Status:
- Current Price: {} SOL
- Cost Basis: {} SOL
- Profit/Loss: {}%
- Partial Sells: {}

Consider:
1. Profit taking levels (50%, 100%, 200%, etc.)
2. Stop loss conditions
3. Market conditions and momentum
4. Risk management principles
5. Original investment thesis

Provide recommendation in JSON format:
{{
    "action": "Sell|Hold",
    "amount_in_sol": 0.0,
    "reasoning": "Clear explanation",
    "confidence": 0.0-1.0,
    "risk_assessment": "Current risk level and explanation"
}}

Remember stoic principles:
- Emotional detachment
- Acceptance of market conditions
- Focus on risk management
- Long-term perspective"#,
            position_context,
            current_price_sol,
            position.cost_basis_sol,
            profit_loss,
            position.partial_sells.len()
        );

        let response = self.agent.prompt(&prompt).await?;
        let recommendation: TradeRecommendation = serde_json::from_str(&response)?;
        
        Ok(Some(recommendation))
    }

    pub fn update_portfolio(&mut self, token: TokenMetadata, quantity: f64, cost_basis_sol: f64) {
        let now = chrono::Utc::now().timestamp();
        self.portfolio.insert(
            token.address.clone(),
            PortfolioPosition {
                token,
                quantity,
                cost_basis_sol,
                entry_timestamp: now,
                partial_sells: Vec::new(),
            },
        );
    }

    pub fn record_partial_sell(
        &mut self,
        token_address: &str,
        quantity: f64,
        price_sol: f64,
    ) -> Result<()> {
        let position = self.portfolio.get_mut(token_address)
            .ok_or_else(|| anyhow::anyhow!("Position not found"))?;

        let now = chrono::Utc::now().timestamp();
        position.partial_sells.push(PartialSell {
            quantity,
            price_sol,
            timestamp: now,
        });
        position.quantity -= quantity;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rig::providers::openai;

    #[tokio::test]
    async fn test_trading_strategy() {
        // Add tests with mock agent responses
    }
} 