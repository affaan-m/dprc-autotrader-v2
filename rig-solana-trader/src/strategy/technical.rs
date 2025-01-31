use crate::market_data::EnhancedTokenMetadata;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct TechnicalAnalyzer {
    rsi_period: u32,
    macd_fast: u32,
    macd_slow: u32,
    macd_signal: u32,
    bb_period: u32,
    bb_std_dev: f64,
}

impl TechnicalAnalyzer {
    pub fn new() -> Self {
        Self {
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            bb_period: 20,
            bb_std_dev: 2.0,
        }
    }

    pub async fn analyze(&self, token: &EnhancedTokenMetadata) -> Result<super::TechnicalSignals> {
        let trend_strength = self.calculate_trend_strength(token);
        let momentum_score = self.calculate_momentum_score(token);
        let volatility_score = self.calculate_volatility_score(token);
        let support_resistance = self.identify_support_resistance(token);
        let signal_type = self.determine_signal_type(
            trend_strength,
            momentum_score,
            volatility_score,
            token,
        );

        Ok(super::TechnicalSignals {
            trend_strength,
            momentum_score,
            volatility_score,
            support_resistance,
            signal_type,
            timeframe: "4h".to_string(), // Default timeframe
        })
    }

    fn calculate_trend_strength(&self, token: &EnhancedTokenMetadata) -> f64 {
        let price_weight = if token.price_change_24h > 0.0 { 0.6 } else { 0.4 };
        let volume_weight = if token.volume_change_24h > 0.0 { 0.7 } else { 0.3 };
        
        let price_score = (token.price_change_24h / 100.0).min(1.0).max(-1.0);
        let volume_score = (token.volume_change_24h / 200.0).min(1.0).max(-1.0);
        
        let trend_score = (price_score * price_weight + volume_score * volume_weight).abs();
        
        if let Some(rsi) = token.rsi_14 {
            let rsi_score = if rsi > 70.0 {
                (100.0 - rsi) / 30.0
            } else if rsi < 30.0 {
                rsi / 30.0
            } else {
                0.5 + (rsi - 50.0) / 40.0
            };
            (trend_score + rsi_score) / 2.0
        } else {
            trend_score
        }
    }

    fn calculate_momentum_score(&self, token: &EnhancedTokenMetadata) -> f64 {
        let mut score = 0.0;
        let mut signals = 0;

        // RSI Signal
        if let Some(rsi) = token.rsi_14 {
            score += if rsi > 70.0 {
                1.0
            } else if rsi < 30.0 {
                -1.0
            } else {
                0.0
            };
            signals += 1;
        }

        // MACD Signal
        if let (Some(macd), Some(signal)) = (token.macd, token.macd_signal) {
            score += if macd > signal {
                1.0
            } else {
                -1.0
            };
            signals += 1;
        }

        // Price momentum
        let price_momentum = token.price_change_24h / 100.0;
        score += price_momentum.signum();
        signals += 1;

        // Volume momentum
        let volume_momentum = token.volume_change_24h / 100.0;
        score += volume_momentum.signum();
        signals += 1;

        if signals > 0 {
            (score / signals as f64 + 1.0) / 2.0 // Normalize to 0-1
        } else {
            0.5 // Neutral if no signals
        }
    }

    fn calculate_volatility_score(&self, token: &EnhancedTokenMetadata) -> f64 {
        let mut volatility = 0.0;

        // Bollinger Bands volatility
        if let (Some(upper), Some(lower)) = (token.bollinger_upper, token.bollinger_lower) {
            let current_price = token.price_usd;
            let band_width = (upper - lower) / current_price;
            volatility += band_width;
        }

        // Price change volatility
        let price_volatility = token.price_change_24h.abs() / 100.0;
        volatility += price_volatility;

        // Volume volatility
        let volume_volatility = token.volume_change_24h.abs() / 100.0;
        volatility += volume_volatility;

        // Normalize to 0-1 range
        (volatility / 3.0).min(1.0)
    }

    fn identify_support_resistance(&self, token: &EnhancedTokenMetadata) -> Vec<f64> {
        // This is a simplified implementation
        // In a real system, this would analyze historical price data
        vec![
            token.price_usd * 0.9,  // Support level
            token.price_usd * 1.1   // Resistance level
        ]
    }

    fn determine_signal_type(
        &self,
        trend_strength: f64,
        momentum_score: f64,
        volatility_score: f64,
        token: &EnhancedTokenMetadata,
    ) -> String {
        if trend_strength > 0.7 && momentum_score > 0.7 {
            if token.price_change_24h > 0.0 {
                "Strong Uptrend".to_string()
            } else {
                "Strong Downtrend".to_string()
            }
        } else if volatility_score > 0.8 {
            "High Volatility".to_string()
        } else if trend_strength < 0.3 {
            "Ranging".to_string()
        } else {
            "Mixed Signals".to_string()
        }
    }
} 