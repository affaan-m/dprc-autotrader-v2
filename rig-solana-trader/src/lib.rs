//! Rig Solana Trader is an autonomous trading bot for the Solana blockchain.
//! 
//! # Overview
//! This bot uses LLM-powered analysis to make trading decisions on Solana tokens.
//! It combines market data from multiple sources, technical analysis, and stoic
//! principles to execute trades through Jupiter DEX.
//!
//! # Key Features
//! - Real-time market data aggregation from BirdEye API
//! - LLM-powered trading analysis with stoic personality
//! - Automated trade execution via Jupiter DEX
//! - Position tracking and portfolio management
//! - Twitter integration for trade announcements
//! - MongoDB persistence for market data and positions
//!
//! # Architecture
//! The bot consists of several key components:
//! - `market_data`: Interfaces with BirdEye API for token data
//! - `strategy`: Implements trading logic and LLM analysis
//! - `personality`: Handles stoic tweet generation
//! - `dex`: Manages trade execution via Jupiter
//! - `database`: Handles MongoDB persistence
//! - `twitter`: Manages Twitter interactions
//! - `agents`: Multi-agent system for trading decisions
//! - `pipeline`: Trading operation orchestration

use rig_core::{
    providers::{
        openai::Client as OpenAIClient,
        twitter::TwitterClient,
        solana::SolanaClient,
    },
    Result,
};
use rig_mongodb::MongoDBClient;
use tracing::{info, debug, Level};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use std::time::Duration;

pub mod agents;
pub mod market_data;
pub mod strategy;
pub mod database;
pub mod dex;
pub mod personality;
pub mod twitter;

// Constants for rate limiting
const TRADE_COOLDOWN: Duration = Duration::from_secs(60); // 1 minute between trades
const TWEET_COOLDOWN: Duration = Duration::from_secs(300); // 5 minutes between tweets
const RESPONSE_COOLDOWN: Duration = Duration::from_secs(60); // 1 minute between responses

pub async fn initialize(
    openai_api_key: &str,
    twitter_bearer_token: &str,
    solana_rpc_url: &str,
    mongodb_uri: &str,
) -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into())
            .add_directive("rig_solana_trader=debug".parse()?))
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_level(true)
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
        .pretty()
        .build();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Initializing Solana trading bot...");

    // Initialize MongoDB connection
    let db_client = MongoDBClient::new(mongodb_uri)
        .await
        .map_err(|e| {
            debug!("MongoDB connection error: {:?}", e);
            e
        })?;

    info!("MongoDB connection established");

    // Initialize OpenAI client
    let openai_client = OpenAIClient::new(openai_api_key);
    debug!("OpenAI client initialized");

    // Initialize Twitter client
    let twitter_client = TwitterClient::new(twitter_bearer_token);
    debug!("Twitter client initialized");

    // Initialize Solana client
    let solana_client = SolanaClient::new(solana_rpc_url);
    debug!("Solana client initialized");

    info!("Initialization complete");
    Ok(())
}

pub fn get_rate_limits() -> (Duration, Duration, Duration) {
    (TRADE_COOLDOWN, TWEET_COOLDOWN, RESPONSE_COOLDOWN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn test_initialization() {
        dotenv().ok();

        let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let twitter_bearer_token = env::var("TWITTER_BEARER_TOKEN").expect("TWITTER_BEARER_TOKEN not set");
        let solana_rpc_url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL not set");
        let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI not set");

        let result = initialize(
            &openai_api_key,
            &twitter_bearer_token,
            &solana_rpc_url,
            &mongodb_uri,
        ).await;

        assert!(result.is_ok(), "Initialization failed: {:?}", result.err());
    }
}

// ... rest of the file ... 