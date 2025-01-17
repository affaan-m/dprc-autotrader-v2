pub mod birdeye;
pub mod streaming;
pub mod storage;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub fully_diluted_market_cap: f64,
    pub circulating_supply: f64,
    pub total_supply: f64,
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
pub trait DataProvider: Send + Sync {
    async fn get_token_metadata(&self, token_address: &str) -> Result<TokenMetadata>;
    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<MarketTrend>>;
    async fn get_historical_prices(&self, address: &str) -> Result<Vec<PricePoint>>;
    async fn get_onchain_metrics(&self, address: &str) -> Result<OnChainMetrics>;
    async fn get_social_metrics(&self, address: &str) -> Result<SocialMetrics>;
}

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
    async fn get_token_metadata(&self, token_address: &str) -> Result<TokenMetadata> {
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

    async fn get_historical_prices(&self, address: &str) -> Result<Vec<PricePoint>> {
        // Try each provider in sequence until one succeeds
        for provider in &self.providers {
            if let Ok(prices) = provider.get_historical_prices(address).await {
                return Ok(prices);
            }
        }
        Err(anyhow::anyhow!("No provider could fetch historical prices"))
    }

    async fn get_onchain_metrics(&self, address: &str) -> Result<OnChainMetrics> {
        // Try each provider in sequence until one succeeds
        for provider in &self.providers {
            if let Ok(metrics) = provider.get_onchain_metrics(address).await {
                return Ok(metrics);
            }
        }
        Err(anyhow::anyhow!("No provider could fetch on-chain metrics"))
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
}

#[derive(Debug, Default)]
struct DataCache {
    metadata_cache: HashMap<String, (TokenMetadata, DateTime<Utc>)>,
    trends_cache: HashMap<String, (Vec<MarketTrend>, DateTime<Utc>)>,
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
        async fn get_token_metadata(&self, address: &str) -> Result<TokenMetadata> {
            Ok(TokenMetadata {
                address: address.to_string(),
                symbol: "TEST".to_string(),
                name: format!("Test Token {}", self.name),
                decimals: 9,
                price_usd: 1.0,
                price_sol: 0.01,
                volume_24h: 1000000.0,
                market_cap: 10000000.0,
                fully_diluted_market_cap: 20000000.0,
                circulating_supply: 1000000.0,
                total_supply: 2000000.0,
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

        async fn get_historical_prices(&self, _address: &str) -> Result<Vec<PricePoint>> {
            Ok(vec![
                PricePoint {
                    timestamp: Utc::now(),
                    price: 1.0,
                    volume: 1000000.0,
                }
            ])
        }

        async fn get_onchain_metrics(&self, _address: &str) -> Result<OnChainMetrics> {
            Ok(OnChainMetrics {
                unique_holders: 1000,
                active_wallets_24h: 500,
                transactions_24h: 5000,
                average_transaction_size: 100.0,
                whale_transactions_24h: 10,
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