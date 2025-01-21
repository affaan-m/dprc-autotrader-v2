//! Database Module for Rig Solana Trader
//!
//! This module handles all MongoDB interactions for the trading bot. It manages:
//!
//! # Collections
//! - `token_states`: Historical market data for tokens
//!   - Indexed by: token address, timestamp
//!   - Contains: price, volume, market cap, changes
//!
//! - `positions`: Active trading positions
//!   - Indexed by: token address
//!   - Contains: entry price, quantity, partial sells
//!
//! # Configuration
//! Database connection is configured through:
//! - `MONGODB_URI`: Connection string with auth
//! - `MONGODB_DB_NAME`: Database name (default: solana_trader)
//!
//! # Indexes
//! The following indexes are recommended:
//! ```js
//! db.token_states.createIndex({ "address": 1, "timestamp": -1 })
//! db.positions.createIndex({ "address": 1 }, { unique: true })
//! ```

pub mod positions;
pub mod sync;

use mongodb::{Client, Database};
use anyhow::Result;

#[derive(Debug)]
pub struct DatabaseClient {
    db: Database,
}

impl DatabaseClient {
    pub async fn new(connection_string: &str, database_name: &str) -> Result<Self> {
        let client = Client::with_uri_str(connection_string).await?;
        let db = client.database(database_name);
        
        Ok(Self { db })
    }

    pub fn positions(&self) -> positions::PositionsCollection {
        positions::PositionsCollection::new(&self.db)
    }

    pub async fn insert_one<T>(&self, collection_name: &str, document: &T) -> Result<()> 
    where 
        T: serde::Serialize 
    {
        self.db.collection(collection_name)
            .insert_one(mongodb::bson::to_document(document)?, None)
            .await?;
        Ok(())
    }
} 