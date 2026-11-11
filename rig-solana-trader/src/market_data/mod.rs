pub mod birdeye;
pub mod streaming;
pub mod storage;
pub mod sentiment;
pub mod macro_indicators;
pub mod feature_engineering;
pub mod vector_store;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};
use vector_store::{TokenVectorStore, TokenAnalysis};
use crate::database::DatabaseClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTokenMetadata {
    // Base token data
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    
    // Price metrics
    pub price_usd: f64,
    pub price_sol: f64,
    pub price_change_1h: f64,
    pub price_change_24h: f64,
    pub price_change_7d: f64,
    
    // Volume metrics
    pub volume_24h: f64,
    pub volume_change_24h: f64,
    pub volume_by_price_24h: f64, // Volume weighted by price
    
    // Market metrics
    pub market_cap: f64,
    pub fully_diluted_market_cap: f64,
    pub circulating_supply: f64,
    pub total_supply: f64,
    
    // Liquidity metrics
    pub liquidity_usd: f64,
    pub liquidity_sol: f64,
    pub liquidity_change_24h: f64,
    
    // Technical indicators
    pub rsi_14: Option<f64>,
    pub macd: Option<f64>,
    pub macd_signal: Option<f64>,
    pub bollinger_upper: Option<f64>,
    pub bollinger_lower: Option<f64>,
    
    // On-chain metrics
    pub unique_holders: u32,
    pub active_wallets_24h: u32,
    pub whale_transactions_24h: u32,
    pub average_transaction_size: f64,
    
    // Sentiment metrics
    pub social_score: Option<f64>,
    pub social_volume: Option<u32>,
    pub social_sentiment: Option<f64>,
    pub dev_activity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroIndicator {
    pub timestamp: DateTime<Utc>,
    pub sol_dominance: f64,
    pub total_market_cap: f64,
    pub total_volume_24h: f64,
    pub market_trend: String,
    pub fear_greed_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub token_address: String,
    pub timestamp: DateTime<Utc>,
    pub features: Vec<f64>,
    pub feature_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTrend {
    pub token_address: String,
    pub metadata: TokenMetadata,
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
    pub social_volume_24h: u32,
    pub dev_activity_24h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainMetrics {
    pub unique_holders: u32,
    pub active_wallets_24h: u32,
    pub transactions_24h: u32,
    pub average_transaction_size: f64,
    pub whale_transactions_24h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMetrics {
    pub twitter_followers: u32,
    pub twitter_engagement_rate: f64,
    pub discord_members: u32,
    pub github_stars: u32,
    pub telegram_members: u32,
}

#[async_trait]
pub trait DataProvider: Send + Sync + std::fmt::Debug {
    async fn get_token_metadata(&self, token_address: &str) -> Result<EnhancedTokenMetadata>;
    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<MarketTrend>>;
    async fn get_historical_prices(&self, address: &str, timeframe: &str) -> Result<Vec<PricePoint>>;
    async fn get_macro_indicators(&self) -> Result<MacroIndicator>;
    async fn get_social_metrics(&self, address: &str) -> Result<SocialMetrics>;
    async fn get_feature_vector(&self, token_address: &str) -> Result<FeatureVector>;
}

#[derive(Debug)]
pub struct AggregatedDataProvider {
    providers: Vec<Arc<dyn DataProvider>>,
    cache: Arc<RwLock<DataCache>>,
}

impl AggregatedDataProvider {
    pub fn new(providers: Vec<Arc<dyn DataProvider>>) -> Self {
        Self {
            providers,
            cache: Arc::new(RwLock::new(DataCache::default())),
        }
    }
}

#[async_trait]
impl DataProvider for AggregatedDataProvider {
    async fn get_token_metadata(&self, token_address: &str) -> Result<EnhancedTokenMetadata> {
        // Try each provider in sequence until one succeeds
        for provider in &self.providers {
            if let Ok(metadata) = provider.get_token_metadata(token_address).await {
                return Ok(metadata);
            }
        }
        Err(anyhow::anyhow!("No provider could fetch token metadata"))
    }

    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let mut all_trends = Vec::new();
        
        // Collect trends from all providers
        for provider in &self.providers {
            if let Ok(mut trends) = provider.get_trending_tokens(limit).await {
                all_trends.append(&mut trends);
            }
        }

        // Deduplicate by token address
        let mut unique_trends = HashMap::new();
        for trend in all_trends {
            unique_trends.entry(trend.token_address.clone())
                .or_insert(trend);
        }

        Ok(unique_trends.into_values().take(limit).collect())
    }

    async fn get_historical_prices(&self, address: &str, timeframe: &str) -> Result<Vec<PricePoint>> {
        // Try each provider in sequence until one succeeds
        for provider in &self.providers {
            if let Ok(prices) = provider.get_historical_prices(address, timeframe).await {
                return Ok(prices);
            }
        }
        Err(anyhow::anyhow!("No provider could fetch historical prices"))
    }

    async fn get_macro_indicators(&self) -> Result<MacroIndicator> {
        // Try each provider in sequence until one succeeds
        for provider in &self.providers {
            if let Ok(indicators) = provider.get_macro_indicators().await {
                return Ok(indicators);
            }
        }
        Err(anyhow::anyhow!("No provider could fetch macro indicators"))
    }

    async fn get_social_metrics(&self, address: &str) -> Result<SocialMetrics> {
        // Try each provider in sequence until one succeeds
        for provider in &self.providers {
            if let Ok(metrics) = provider.get_social_metrics(address).await {
                return Ok(metrics);
            }
        }
        Err(anyhow::anyhow!("No provider could fetch social metrics"))
    }

    async fn get_feature_vector(&self, token_address: &str) -> Result<FeatureVector> {
        // Try each provider in sequence until one succeeds
        for provider in &self.providers {
            if let Ok(vector) = provider.get_feature_vector(token_address).await {
                return Ok(vector);
            }
        }
        Err(anyhow::anyhow!("No provider could fetch feature vector"))
    }
}

#[derive(Debug, Default)]
struct DataCache {
    metadata_cache: HashMap<String, (EnhancedTokenMetadata, DateTime<Utc>)>,
    trends_cache: HashMap<String, (Vec<MarketTrend>, DateTime<Utc>)>,
}

pub struct MarketDataProvider {
    vector_store: TokenVectorStore,
    db_client: DatabaseClient,
    // ... existing fields ...
}

impl MarketDataProvider {
    pub async fn new(openai_api_key: &str, db_client: DatabaseClient) -> Result<Self> {
        let vector_store = TokenVectorStore::new(openai_api_key, db_client.clone()).await?;
        
        Ok(Self {
            vector_store,
            db_client,
            // ... initialize other fields ...
        })
    }

    pub async fn analyze_token(&mut self, token_address: &str) -> Result<()> {
        debug!("Analyzing token {}", token_address);
        
        // Get token metadata and market data
        let metadata = self.get_token_metadata(token_address).await?;
        let market_data = self.get_market_data(token_address).await?;
        
        // Create token analysis
        let analysis = TokenAnalysis {
            token_address: token_address.to_string(),
            symbol: metadata.symbol.clone(),
            description: metadata.description.unwrap_or_default(),
            recent_events: market_data.recent_events,
            market_sentiment: self.analyze_market_sentiment(&market_data).await?,
        };
        
        // Add to vector store (which will also persist to database)
        self.vector_store.add_token_analysis(analysis).await?;
        Ok(())
    }

    pub async fn find_similar_tokens(&self, query: &str, limit: usize) -> Result<Vec<TokenAnalysis>> {
        debug!("Finding tokens similar to query: {}", query);
        let results = self.vector_store.find_similar_tokens(query, limit).await?;
        Ok(results.into_iter().map(|(_, _, analysis)| analysis).collect())
    }

    pub async fn get_token_sentiment(&self, token_address: &str) -> Result<Option<String>> {
        self.vector_store.get_token_sentiment(token_address).await
    }

    async fn analyze_market_sentiment(&self, market_data: &MarketData) -> Result<String> {
        // TODO: Implement sentiment analysis using LLM
        // For now return a placeholder
        Ok("neutral".to_string())
    }

    // ... existing methods ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MockProvider {
        name: String,
    }

    #[async_trait]
    impl DataProvider for MockProvider {
        async fn get_token_metadata(&self, address: &str) -> Result<EnhancedTokenMetadata> {
            Ok(EnhancedTokenMetadata {
                address: address.to_string(),
                symbol: "TEST".to_string(),
                name: format!("Test Token {}", self.name),
                decimals: 9,
                price_usd: 1.0,
                price_sol: 0.01,
                price_change_1h: 0.0,
                price_change_24h: 0.0,
                price_change_7d: 0.0,
                volume_24h: 1000000.0,
                volume_change_24h: 0.0,
                volume_by_price_24h: 0.0,
                market_cap: 10000000.0,
                fully_diluted_market_cap: 20000000.0,
                circulating_supply: 1000000.0,
                total_supply: 2000000.0,
                liquidity_usd: 0.0,
                liquidity_sol: 0.0,
                liquidity_change_24h: 0.0,
                rsi_14: None,
                macd: None,
                macd_signal: None,
                bollinger_upper: None,
                bollinger_lower: None,
                unique_holders: 0,
                active_wallets_24h: 0,
                whale_transactions_24h: 0,
                average_transaction_size: 0.0,
                social_score: None,
                social_volume: None,
                social_sentiment: None,
                dev_activity: None,
            })
        }

        async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<MarketTrend>> {
            let mut trends = Vec::new();
            for i in 0..limit {
                trends.push(MarketTrend {
                    token_address: format!("addr{}", i),
                    price_change_24h: 10.0,
                    volume_change_24h: 1000000.0,
                    social_volume_24h: 1000,
                    dev_activity_24h: 50,
                    metadata: TokenMetadata {
                        address: format!("addr{}", i),
                        symbol: "TEST".to_string(),
                        name: format!("Test Token {} {}", self.name, i),
                        decimals: 9,
                        price_usd: 1.0,
                        price_sol: 0.01,
                        volume_24h: 1000000.0,
                        market_cap: 10000000.0,
                        fully_diluted_market_cap: 20000000.0,
                        circulating_supply: 1000000.0,
                        total_supply: 2000000.0,
                    },
                });
            }
            Ok(trends)
        }

        async fn get_historical_prices(&self, _address: &str, _timeframe: &str) -> Result<Vec<PricePoint>> {
            Ok(vec![
                PricePoint {
                    timestamp: Utc::now(),
                    price: 1.0,
                    volume: 1000000.0,
                }
            ])
        }

        async fn get_macro_indicators(&self) -> Result<MacroIndicator> {
            Ok(MacroIndicator {
                timestamp: Utc::now(),
                sol_dominance: 0.5,
                total_market_cap: 1000000000.0,
                total_volume_24h: 10000000.0,
                market_trend: "Bullish".to_string(),
                fear_greed_index: 70,
            })
        }

        async fn get_social_metrics(&self, _address: &str) -> Result<SocialMetrics> {
            Ok(SocialMetrics {
                twitter_followers: 10000,
                twitter_engagement_rate: 1000,
                discord_members: 5000,
                telegram_members: 3000,
                github_stars: 100,
            })
        }

        async fn get_feature_vector(&self, _token_address: &str) -> Result<FeatureVector> {
            Ok(FeatureVector {
                token_address: "test_addr".to_string(),
                timestamp: Utc::now(),
                features: vec![0.5, 0.3, 0.8],
                feature_names: vec!["Social Score".to_string(), "Dev Activity".to_string(), "Liquidity Change".to_string()],
            })
        }
    }

    #[tokio::test]
    async fn test_aggregated_provider() {
        let mut provider = AggregatedDataProvider::new();
        provider.add_provider(Box::new(MockProvider { name: "A".to_string() }));
        provider.add_provider(Box::new(MockProvider { name: "B".to_string() }));

        let trends = provider.get_aggregated_trends(5).await.unwrap();
        assert_eq!(trends.len(), 5);

        let metadata = provider.get_token_metadata("test_addr").await.unwrap();
        assert_eq!(metadata.symbol, "TEST");

        let (onchain, social) = provider.get_comprehensive_metrics("test_addr").await.unwrap();
        assert_eq!(onchain.unique_holders, 1000);
        assert_eq!(social.twitter_followers, 10000);
    }
} 