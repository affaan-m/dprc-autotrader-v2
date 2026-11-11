use crate::market_data::EnhancedTokenMetadata;
use crate::strategy::{TechnicalSignals, MarketContext, StrategyConfig};
use anyhow::Result;
use rig_solana_trader::personality::StoicPersonality;

#[derive(Debug)]
pub struct RiskManager {
    config: StrategyConfig,
    max_position_per_token: f64,
    max_drawdown: f64,
    min_liquidity_ratio: f64,
    personality: StoicPersonality,
}

impl RiskManager {
    pub fn new(config: StrategyConfig, personality: StoicPersonality) -> Self {
        Self {
            config,
            max_position_per_token: 0.2, // 20% of portfolio per token
            max_drawdown: 0.2,
            min_liquidity_ratio: 0.1, // Minimum liquidity to market cap ratio
            personality,
        }
    }

    pub async fn assess_risk(
        &self,
        token: &EnhancedTokenMetadata,
        technical: &TechnicalSignals,
        market: &MarketContext,
    ) -> Result<f64> {
        let mut risk_score = 0.0;
        let mut weight_sum = 0.0;

        // 1. Liquidity Risk (0.0 = high risk, 1.0 = low risk)
        let liquidity_risk = self.assess_liquidity_risk(token);
        risk_score += liquidity_risk * 0.3;
        weight_sum += 0.3;

        // 2. Volatility Risk
        let volatility_risk = 1.0 - technical.volatility_score;
        risk_score += volatility_risk * 0.2;
        weight_sum += 0.2;

        // 3. Market Risk
        let market_risk = self.assess_market_risk(market);
        risk_score += market_risk * 0.15;
        weight_sum += 0.15;

        // 4. Technical Risk
        let technical_risk = self.assess_technical_risk(technical);
        risk_score += technical_risk * 0.2;
        weight_sum += 0.2;

        // 5. Social/Sentiment Risk
        let sentiment_risk = self.assess_sentiment_risk(token, market);
        risk_score += sentiment_risk * 0.15;
        weight_sum += 0.15;

        // Normalize risk score to 0-1 range (0 = highest risk, 1 = lowest risk)
        Ok(risk_score / weight_sum)
    }

    fn assess_liquidity_risk(&self, token: &EnhancedTokenMetadata) -> f64 {
        let mut risk_score = 0.0;

        // Liquidity to market cap ratio
        let liquidity_ratio = token.liquidity_usd / token.market_cap;
        if liquidity_ratio >= self.min_liquidity_ratio {
            risk_score += 0.4;
        }

        // Volume analysis
        let volume_to_mcap = token.volume_24h / token.market_cap;
        risk_score += (volume_to_mcap * 5.0).min(0.3); // Cap at 0.3

        // Liquidity change trend
        if token.liquidity_change_24h > 0.0 {
            risk_score += 0.2;
        }

        // Minimum thresholds
        if token.liquidity_usd < self.config.min_liquidity_usd {
            return 0.0; // Immediate rejection if below minimum liquidity
        }

        risk_score.min(1.0)
    }

    fn assess_market_risk(&self, market: &MarketContext) -> f64 {
        let mut risk_score = 0.5; // Start neutral

        // Market trend analysis
        match market.market_trend.as_str() {
            "Bullish" => risk_score += 0.2,
            "Bearish" => risk_score -= 0.2,
            _ => {} // Neutral
        }

        // Sector performance
        if market.sector_performance > 0.0 {
            risk_score += 0.1;
        } else {
            risk_score -= 0.1;
        }

        // Volume profile
        if market.volume_profile == "High" {
            risk_score += 0.1;
        }

        risk_score.max(0.0).min(1.0)
    }

    fn assess_technical_risk(&self, technical: &TechnicalSignals) -> f64 {
        let mut risk_score = 0.0;

        // Trend strength
        risk_score += technical.trend_strength * 0.4;

        // Momentum
        risk_score += technical.momentum_score * 0.3;

        // Signal type analysis
        match technical.signal_type.as_str() {
            "Strong Uptrend" => risk_score += 0.2,
            "Strong Downtrend" => risk_score -= 0.1,
            "High Volatility" => risk_score -= 0.2,
            "Ranging" => risk_score += 0.1,
            _ => {}
        }

        risk_score.max(0.0).min(1.0)
    }

    fn assess_sentiment_risk(&self, token: &EnhancedTokenMetadata, market: &MarketContext) -> f64 {
        let mut risk_score = 0.5; // Start neutral

        // Social sentiment
        if let Some(sentiment) = token.social_sentiment {
            risk_score += (sentiment - 0.5) * 0.3;
        }

        // Social volume
        if let Some(volume) = token.social_volume {
            if volume > 1000 {
                risk_score += 0.1;
            }
        }

        // Development activity
        if let Some(dev_activity) = token.dev_activity {
            if dev_activity > 0 {
                risk_score += 0.1;
            }
        }

        // Market sentiment correlation
        risk_score += (market.sentiment_score - 0.5) * 0.2;

        risk_score.max(0.0).min(1.0)
    }

    pub fn validate_position_size(&self, size_in_sol: f64, current_portfolio_value: f64) -> bool {
        // Check if position size is within limits
        if size_in_sol < self.config.min_position_sol || size_in_sol > self.config.max_position_sol {
            return false;
        }

        // Check position size relative to portfolio
        let position_ratio = size_in_sol / current_portfolio_value;
        if position_ratio > self.max_position_per_token {
            return false;
        }

        true
    }

    pub fn validate_trade(&self, action: &TradeAction) -> Result<()> {
        let risk_score = self.calculate_risk_score(action);
        
        if risk_score > self.personality.risk_tolerance {
            return Err(anyhow::anyhow!(
                "Risk score {} exceeds tolerance {}",
                risk_score,
                self.personality.risk_tolerance
            ));
        }

        Ok(())
    }

    fn calculate_risk_score(&self, action: &TradeAction) -> f64 {
        let market_risk = action.analysis.as_ref().map(|a| a.risk_assessment).unwrap_or(1.0);
        let position_risk = action.params.amount / self.personality.max_position_size;
        
        market_risk * position_risk
    }
} 