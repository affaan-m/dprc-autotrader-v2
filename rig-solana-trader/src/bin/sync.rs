//! Market Data Synchronization Service
//! 
//! This binary runs a service that continuously synchronizes market data from various sources
//! (primarily BirdEye) into MongoDB for analysis and trading decisions. It handles:
//! 
//! - Fetching trending tokens at configurable intervals
//! - Storing token states with price, volume, and market data
//! - Detailed logging of all operations for monitoring
//! - Graceful shutdown on Ctrl+C
//!
//! # Configuration
//! The service is configured through environment variables:
//! - `MONGODB_URI`: MongoDB connection string (default: mongodb://localhost:27017)
//! - `BIRDEYE_API_KEY`: API key for BirdEye data
//! - `DATA_SYNC_INTERVAL_SECONDS`: Interval between syncs (default: 60)
//! - `RUST_LOG`: Logging level configuration
//!
//! # Usage
//! ```bash
//! cargo run --bin sync
//! ```

use std::sync::Arc;
use anyhow::Result;
use dotenv::dotenv;
use tracing::{info, error, debug, warn, instrument};
use tracing_subscriber::{fmt, EnvFilter};
use chrono::Utc;
use rig_solana_trader::{
    database::DatabaseClient,
    market_data::{
        AggregatedDataProvider,
        birdeye::BirdEyeProvider,
        MarketTrend,
        DataProvider,
        TokenMetadata,
    },
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenState {
    address: String,
    symbol: String,
    name: String,
    price_usd: f64,
    price_sol: f64,
    volume_24h: f64,
    market_cap: f64,
    price_change_24h: f64,
    volume_change_24h: f64,
    timestamp: chrono::DateTime<Utc>,
}

struct DataSyncService {
    data_provider: Arc<AggregatedDataProvider>,
    db: Arc<DatabaseClient>,
}

impl DataSyncService {
    #[instrument]
    fn new(
        data_provider: Arc<AggregatedDataProvider>,
        db: Arc<DatabaseClient>,
    ) -> Self {
        info!("Creating new DataSyncService instance");
        let service = Self {
            data_provider,
            db,
        };
        
        service.start_sync_tasks();
        info!("DataSyncService initialized successfully");
        service
    }

    #[instrument(skip(self))]
    fn start_sync_tasks(&self) {
        let data_provider = Arc::clone(&self.data_provider);
        let db = Arc::clone(&self.db);

        info!("Starting market data sync task");
        tokio::spawn(async move {
            loop {
                info!("Beginning new market data sync cycle");
                debug!("Fetching trending tokens from data provider");
                
                match data_provider.as_ref().get_trending_tokens(100).await {
                    Ok(trends) => {
                        info!(
                            token_count = trends.len(),
                            "Successfully fetched trending tokens"
                        );
                        
                        for trend in trends {
                            debug!(
                                token.address = %trend.token_address,
                                token.symbol = %trend.metadata.symbol,
                                token.name = %trend.metadata.name,
                                token.price_usd = trend.metadata.price_usd,
                                token.volume_24h = trend.metadata.volume_24h,
                                token.price_change_24h = trend.price_change_24h,
                                "Processing token data"
                            );

                            let token_state = TokenState {
                                address: trend.token_address.clone(),
                                symbol: trend.metadata.symbol.clone(),
                                name: trend.metadata.name.clone(),
                                price_usd: trend.metadata.price_usd,
                                price_sol: trend.metadata.price_sol,
                                volume_24h: trend.metadata.volume_24h,
                                market_cap: trend.metadata.market_cap,
                                price_change_24h: trend.price_change_24h,
                                volume_change_24h: trend.volume_change_24h,
                                timestamp: Utc::now(),
                            };

                            debug!(
                                token.symbol = %token_state.symbol,
                                token.price_usd = token_state.price_usd,
                                token.volume_24h = token_state.volume_24h,
                                "Inserting token state into MongoDB"
                            );

                            match db.insert_one("token_states", &token_state).await {
                                Ok(_) => info!(
                                    token.symbol = %token_state.symbol,
                                    token.price_usd = token_state.price_usd,
                                    token.volume_24h = token_state.volume_24h,
                                    token.price_change_24h = token_state.price_change_24h,
                                    "Successfully stored token state"
                                ),
                                Err(e) => error!(
                                    token.symbol = %token_state.symbol,
                                    error = %e,
                                    "Failed to insert token state"
                                ),
                            }
                        }
                    },
                    Err(e) => {
                        error!(
                            error = %e,
                            "Failed to fetch trending tokens"
                        );
                    }
                }

                info!("Market data sync cycle complete");
                debug!("Sleeping for 60 seconds before next sync cycle");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize detailed logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("rig_solana_trader=debug".parse().unwrap()))
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_target(true)
        .init();

    info!("Starting Solana trading bot...");
    dotenv().ok();
    
    // Initialize MongoDB client
    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    info!("Connecting to MongoDB at {}", mongodb_uri);
    let db = Arc::new(DatabaseClient::new(&mongodb_uri, "solana_trades").await?);
    info!("Successfully connected to MongoDB");

    // Initialize data providers
    debug!("Initializing BirdEye API client");
    let birdeye_api_key = std::env::var("BIRDEYE_API_KEY")
        .expect("BIRDEYE_API_KEY must be set");
    let birdeye = Arc::new(BirdEyeProvider::new(birdeye_api_key));
    info!("BirdEye API client initialized successfully");
    
    let data_provider = Arc::new(AggregatedDataProvider::new(vec![birdeye]));
    info!("Data provider aggregation complete");

    // Start data sync service
    info!("Starting data sync service...");
    let _sync_service = DataSyncService::new(
        data_provider.clone(),
        db.clone(),
    );
    info!("Data sync service started successfully");

    info!("Bot is now running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");
    info!("Shutting down gracefully...");
    
    Ok(())
} 