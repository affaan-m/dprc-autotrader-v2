pub mod database;
pub mod dex;
pub mod market_data;
pub mod personality;
pub mod strategy;
pub mod twitter;

use market_data::AggregatedDataProvider;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

// ... rest of the file ... 