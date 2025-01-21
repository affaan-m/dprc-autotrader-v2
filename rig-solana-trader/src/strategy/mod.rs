//! Trading Strategy Implementation
//!
//! This module implements the core trading logic using LLM-powered analysis.
//! The strategy combines multiple factors:
//!
//! # Analysis Factors
//! - Market momentum and trends
//! - Volume and liquidity analysis
//! - Price action patterns
//! - Social sentiment and metrics
//! - On-chain activity
//!
//! # Risk Management
//! Configurable parameters (via .env):
//! - `MAX_POSITION_SIZE_SOL`: Maximum position size (default: 1.0 SOL)
//! - `MIN_POSITION_SIZE_SOL`: Minimum position size (default: 0.1 SOL)
//! - `MAX_TOKENS_PER_WALLET`: Maximum concurrent positions
//! - `STOP_LOSS_PERCENTAGE`: Auto stop-loss trigger
//! - `TAKE_PROFIT_PERCENTAGE`: Auto take-profit levels
//! - `MIN_LIQUIDITY_USD`: Minimum liquidity requirement
//! - `MIN_CONFIDENCE_THRESHOLD`: Required confidence for trades
//!
//! # Position Management
//! - Automatic position tracking
//! - Partial profit taking
//! - Dynamic position sizing
//! - Trading cooldown periods

pub mod llm;

use crate::market_data::TokenMetadata;
use anyhow::Result;
use rig::agent::Agent;
use rig::completion::{CompletionModel, Prompt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub token: TokenMetadata,
    pub quantity: f64,
    pub cost_basis_sol: f64,
    pub entry_timestamp: i64,
    pub partial_sells: Vec<PartialSell>,
    #[serde(default)]
    pub mongodb_id: Option<String>, // For MongoDB document tracking
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialSell {
    pub quantity: f64,
    pub price_sol: f64,
    pub timestamp: i64,
    pub tx_signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeRecommendation {
    pub token_address: String,
    pub action: TradeAction,
    pub amount_in_sol: f64,
    pub reasoning: String,
    pub confidence: f64,
    pub risk_assessment: String,
    pub market_analysis: MarketAnalysis,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketAnalysis {
    pub volume_analysis: String,
    pub price_trend: String,
    pub liquidity_assessment: String,
    pub momentum_indicators: String,
    pub risk_factors: Vec<String>,
    pub opportunity_factors: Vec<String>,
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
    recently_purchased: HashMap<String, i64>, // token_address -> timestamp
}

impl<M: CompletionModel> TradingStrategy<M> {
    pub fn new(agent: Agent<M>, max_position_sol: f64, min_position_sol: f64) -> Self {
        Self {
            agent,
            portfolio: HashMap::new(),
            max_position_sol,
            min_position_sol,
            recently_purchased: HashMap::new(),
        }
    }

    pub async fn analyze_trading_opportunity(&self, prompt: &String, sol_balance: f64) -> Result<String> {
        let formatted_prompt = format!(
            r#"You are a seasoned crypto trading expert focused on Solana ecosystem tokens. Analyze opportunities and provide recommendations in valid JSON format.

Key Trading Rules:
1. Input token MUST be SOL (address: "So11111111111111111111111111111111111111112")
2. Output token MUST NOT be SOL
3. Maximum trade size: {:.4} SOL (20% of current balance {:.4} SOL)
4. Minimum trade size: 0.1 SOL
5. No meme coins or tokens with less than 1 week of trading history
6. Require minimum $50k daily volume and $500k market cap

Market Analysis Requirements:
1. Volume Analysis:
   - 24h volume vs 7-day average
   - Large wallet transaction patterns
   - Buy/sell ratio in last 24h
   - Whale accumulation signals
   - Volume concentration (DEX distribution)

2. Price Action:
   - Key support/resistance levels
   - Moving averages (15m, 1h, 4h, 1d)
   - Bollinger bands breakouts
   - RSI divergences
   - MACD crossovers

3. Liquidity Analysis:
   - DEX liquidity depth ($100k+ preferred)
   - Liquidity provider concentration
   - Recent liquidity changes
   - Slippage impact on $1k, $5k, $10k trades
   - Liquidity lockup periods

4. Project Fundamentals:
   - Development activity (GitHub/commits)
   - Team background and track record
   - Recent partnerships/integrations
   - Token utility and use cases
   - Community growth metrics

5. Risk Assessment:
   - Smart contract audits
   - Team token vesting/unlocks
   - Insider wallet movements
   - Competition analysis
   - Regulatory compliance

Confidence Score Guidelines:
0.1-0.3: New tokens, limited history
0.3-0.5: Established but uncertain trend
0.5-0.7: Good fundamentals, needs more signals
0.7-0.8: Strong buy with multiple confirmations
0.8-0.9: High conviction setup (rare)
0.9+: Reserved for exceptional opportunities

JSON Response Format:
{{
    "token_address": "string",
    "action": "Buy" | "Sell" | "Hold",
    "amount_in_sol": number,
    "reasoning": "string",
    "confidence": number between 0.0 and 1.0,
    "risk_assessment": "string",
    "market_analysis": {{
        "volume_analysis": "string",
        "price_trend": "string",
        "liquidity_assessment": "string",
        "momentum_indicators": "string",
        "risk_factors": ["string"],
        "opportunity_factors": ["string"]
    }}
}}

Buy Signal Requirements:
1. Confidence > 0.7 AND
2. 24h volume > $50k AND
3. Liquidity depth > $100k AND
4. At least 3 bullish technical indicators AND
5. No major risk factors

Trading opportunity details:
{}"#,
            sol_balance * 0.20,
            sol_balance,
            prompt
        );

        let response = self.agent.prompt(&formatted_prompt).await?;
        Ok(response.to_string())
    }

    pub async fn evaluate_sell_opportunity(
        &self,
        position: &PortfolioPosition,
        current_price_sol: f64,
    ) -> Result<Option<TradeRecommendation>> {
        let profit_loss = (current_price_sol - position.cost_basis_sol) / position.cost_basis_sol * 100.0;
        let partial_sells = position.partial_sells.len();

        // Implement tiered take-profit and stop-loss logic
        let (action, amount) = match (profit_loss, partial_sells) {
            (loss, _) if loss <= -50.0 => (TradeAction::Sell, position.quantity), // Stop loss
            (profit, 0) if profit >= 100.0 => (TradeAction::Sell, position.quantity * 0.5), // First take profit
            (profit, 1) if profit >= 200.0 => (TradeAction::Sell, position.quantity * 0.25), // Second take profit
            (profit, 2) if profit >= 400.0 => (TradeAction::Sell, position.quantity * 0.15), // Third take profit
            (profit, 3) if profit >= 800.0 => (TradeAction::Sell, position.quantity * 0.1), // Fourth take profit
            _ => (TradeAction::Hold, 0.0),
        };

        if matches!(action, TradeAction::Hold) {
            return Ok(None);
        }

        Ok(Some(TradeRecommendation {
            token_address: position.token.address.clone(),
            action,
            amount_in_sol: amount * current_price_sol,
            reasoning: format!("Taking profit at {:.1}% gain", profit_loss),
            confidence: 0.9,
            risk_assessment: "Profit target reached".to_string(),
            market_analysis: MarketAnalysis {
                volume_analysis: "".to_string(),
                price_trend: "".to_string(),
                liquidity_assessment: "".to_string(),
                momentum_indicators: "".to_string(),
                risk_factors: Vec::new(),
                opportunity_factors: Vec::new(),
            },
        }))
    }

    pub fn update_portfolio(&mut self, token: TokenMetadata, quantity: f64, cost_basis_sol: f64) {
        let now = Utc::now().timestamp();
        let token_address = token.address.clone();
        self.portfolio.insert(
            token.address.clone(),
            PortfolioPosition {
                token,
                quantity,
                cost_basis_sol,
                entry_timestamp: now,
                partial_sells: Vec::new(),
                mongodb_id: None,
            },
        );
        self.recently_purchased.insert(token_address, now);
    }

    pub fn record_partial_sell(
        &mut self,
        token_address: &str,
        quantity: f64,
        price_sol: f64,
    ) -> Result<()> {
        let position = self.portfolio.get_mut(token_address)
            .ok_or_else(|| anyhow::anyhow!("Position not found"))?;

        let now = Utc::now().timestamp();
        position.partial_sells.push(PartialSell {
            quantity,
            price_sol,
            timestamp: now,
            tx_signature: None,
        });
        position.quantity -= quantity;

        Ok(())
    }

    pub fn was_recently_purchased(&self, token_address: &str) -> bool {
        if let Some(timestamp) = self.recently_purchased.get(token_address) {
            let now = Utc::now().timestamp();
            now - timestamp < 3600 * 24 // 24 hour cooldown
        } else {
            false
        }
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