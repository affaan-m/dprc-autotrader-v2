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
//! - `DATABASE_URL`: MongoDB connection string
//! - Database name: solana_trader

pub mod positions;
pub mod sync;

use mongodb::{Client, Database, IndexModel};
use mongodb::options::{IndexOptions, ClientOptions};
use mongodb::bson::{doc, Document};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use chrono::Utc;
use serde::{Serialize, Deserialize, DeserializeOwned};

const POSITIONS_COLLECTION: &str = "positions";
const TOKEN_STATES_COLLECTION: &str = "token_states";
const TRADE_HISTORY_COLLECTION: &str = "trade_history";
const SOCIAL_INTERACTIONS_COLLECTION: &str = "social_interactions";
const TOKEN_ANALYSIS_COLLECTION: &str = "token_analysis";

#[derive(Debug)]
pub struct DatabaseClient<T> where T: Serialize + DeserializeOwned {
    db: Database,
}

impl<T> DatabaseClient<T> where T: Serialize + DeserializeOwned {
    pub async fn new(connection_string: &str, database_name: &str) -> Result<Self> {
        debug!("Initializing MongoDB client with database: {}", database_name);
        
        let mut client_options = ClientOptions::parse(connection_string).await?;
        client_options.app_name = Some("rig-solana-trader".to_string());
        
        let client = Client::with_options(client_options)?;
        let db = client.database(database_name);
        
        info!("MongoDB client initialized successfully");
        Ok(Self { db })
    }

    pub async fn initialize_collections(&self) -> Result<()> {
        info!("Initializing MongoDB collections and indexes...");

        // Positions collection indexes
        self.create_positions_indexes().await?;
        info!("Positions collection indexes created");

        // Token states collection indexes
        self.create_token_states_indexes().await?;
        info!("Token states collection indexes created");

        // Trade history collection indexes
        self.create_trade_history_indexes().await?;
        info!("Trade history collection indexes created");

        // Social interactions collection indexes
        self.create_social_indexes().await?;
        info!("Social interactions collection indexes created");

        // Token analysis collection indexes
        self.create_token_analysis_indexes().await?;
        info!("Token analysis collection indexes created");

        Ok(())
    }

    async fn create_positions_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(POSITIONS_COLLECTION);
        
        // Unique index on token address
        let address_index = IndexModel::builder()
            .keys(doc! { "token.address": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Index on entry timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "entry_timestamp": -1 })
            .build();

        collection.create_index(address_index, None).await?;
        collection.create_index(timestamp_index, None).await?;

        debug!("Created indexes for positions collection");
        Ok(())
    }

    async fn create_token_states_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(TOKEN_STATES_COLLECTION);
        
        // Compound index on address and timestamp
        let address_time_index = IndexModel::builder()
            .keys(doc! { "address": 1, "timestamp": -1 })
            .build();

        // Index on market cap for quick sorting
        let market_cap_index = IndexModel::builder()
            .keys(doc! { "market_cap": -1 })
            .build();

        collection.create_index(address_time_index, None).await?;
        collection.create_index(market_cap_index, None).await?;

        debug!("Created indexes for token states collection");
        Ok(())
    }

    async fn create_trade_history_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(TRADE_HISTORY_COLLECTION);
        
        // Index on timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "timestamp": -1 })
            .build();

        // Compound index on token address and timestamp
        let token_time_index = IndexModel::builder()
            .keys(doc! { "token_address": 1, "timestamp": -1 })
            .build();

        collection.create_index(timestamp_index, None).await?;
        collection.create_index(token_time_index, None).await?;

        debug!("Created indexes for trade history collection");
        Ok(())
    }

    async fn create_social_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(SOCIAL_INTERACTIONS_COLLECTION);
        
        // Index on timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "timestamp": -1 })
            .build();

        // Index on interaction type
        let type_index = IndexModel::builder()
            .keys(doc! { "interaction_type": 1 })
            .build();

        // Compound index on user and timestamp
        let user_time_index = IndexModel::builder()
            .keys(doc! { "user_id": 1, "timestamp": -1 })
            .build();

        collection.create_index(timestamp_index, None).await?;
        collection.create_index(type_index, None).await?;
        collection.create_index(user_time_index, None).await?;

        debug!("Created indexes for social interactions collection");
        Ok(())
    }

    async fn create_token_analysis_indexes(&self) -> Result<()> {
        let collection = self.db.collection::<Document>(TOKEN_ANALYSIS_COLLECTION);
        
        // Index on token address
        let address_index = IndexModel::builder()
            .keys(doc! { "token_address": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();

        // Index on timestamp
        let timestamp_index = IndexModel::builder()
            .keys(doc! { "timestamp": -1 })
            .build();

        // Index on embedding vector
        let vector_index = IndexModel::builder()
            .keys(doc! { "embedding": 1 })
            .build();

        collection.create_index(address_index, None).await?;
        collection.create_index(timestamp_index, None).await?;
        collection.create_index(vector_index, None).await?;

        debug!("Created indexes for token analysis collection");
        Ok(())
    }

    pub fn positions(&self) -> positions::PositionsCollection {
        positions::PositionsCollection::new(&self.db)
    }

    pub async fn insert_one<T>(&self, collection_name: &str, document: &T) -> Result<()> 
    where 
        T: serde::Serialize 
    {
        debug!("Inserting document into collection: {}", collection_name);
        
        self.db.collection(collection_name)
            .insert_one(mongodb::bson::to_document(document)?, None)
            .await?;
            
        debug!("Document inserted successfully");
        Ok(())
    }

    pub async fn find_one<T>(&self, collection_name: &str, filter: Document) -> Result<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>
    {
        debug!("Finding document in collection: {} with filter: {:?}", collection_name, filter);
        
        let result = self.db.collection(collection_name)
            .find_one(filter, None)
            .await?;
            
        if result.is_some() {
            debug!("Document found");
        } else {
            debug!("No document found");
        }
        
        Ok(result)
    }

    pub async fn save_token_analysis(&self, analysis: &TokenAnalysis, embedding: Vec<f32>) -> Result<()> {
        let collection = self.db.collection::<Document>(TOKEN_ANALYSIS_COLLECTION);
        
        let doc = doc! {
            "token_address": &analysis.token_address,
            "symbol": &analysis.symbol,
            "description": &analysis.description,
            "recent_events": &analysis.recent_events,
            "market_sentiment": &analysis.market_sentiment,
            "embedding": embedding,
            "timestamp": Utc::now(),
        };

        collection.insert_one(doc, None).await?;
        debug!("Saved token analysis for {}", analysis.symbol);
        Ok(())
    }

    pub async fn get_token_analysis(&self, token_address: &str) -> Result<Option<(TokenAnalysis, Vec<f32>)>> {
        let collection = self.db.collection::<Document>(TOKEN_ANALYSIS_COLLECTION);
        
        let filter = doc! { "token_address": token_address };
        let result = collection.find_one(filter, None).await?;
        
        if let Some(doc) = result {
            let analysis = TokenAnalysis {
                token_address: doc.get_str("token_address")?.to_string(),
                symbol: doc.get_str("symbol")?.to_string(),
                description: doc.get_str("description")?.to_string(),
                recent_events: doc.get_array("recent_events")?
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect(),
                market_sentiment: doc.get_str("market_sentiment")?.to_string(),
            };
            
            let embedding = doc.get_array("embedding")?
                .iter()
                .map(|v| v.as_f64().unwrap_or_default() as f32)
                .collect();
                
            Ok(Some((analysis, embedding)))
        } else {
            Ok(None)
        }
    }
} 