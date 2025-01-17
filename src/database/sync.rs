use anyhow::Result;
use chrono::{DateTime, Utc};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

use crate::market_data::{DataProvider, MarketTrend};
use crate::twitter::TwitterClient;
use crate::strategy::{TradingStrategy, TradeAction, TradeRecommendation};
use crate::dex::jupiter::JupiterDex;
use crate::StoicPersonality;
use rig::completion::CompletionModel;
use solana_sdk::signature::Keypair;
use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenState {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
    pub timestamp: DateTime<Utc>,
}

// ... rest of the existing code ... 