use super::{DataProvider, TokenMetadata, MarketTrend, PricePoint, OnChainMetrics, SocialMetrics};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct CoinGeckoProvider {
    client: reqwest::Client,
    solana_tokens: HashMap<String, String>, // Maps Solana address to CoinGecko ID
}

#[derive(Debug, Deserialize)]
struct CoinGeckoTokenData {
    id: String,
    symbol: String,
    name: String,
    platforms: HashMap<String, String>,
    market_data: MarketData,
    community_data: Option<CommunityData>,
    developer_data: Option<DeveloperData>,
}

#[derive(Debug, Deserialize)]
struct MarketData {
    current_price: HashMap<String, f64>,
    price_change_percentage_24h: Option<f64>,
    total_volume: HashMap<String, f64>,
    market_cap: HashMap<String, f64>,
    fully_diluted_valuation: HashMap<String, f64>,
    circulating_supply: Option<f64>,
    total_supply: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct CommunityData {
    twitter_followers: Option<u32>,
    telegram_channel_user_count: Option<u32>,
    reddit_subscribers: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct DeveloperData {
    forks: Option<u32>,
    stars: Option<u32>,
    subscribers: Option<u32>,
    total_issues: Option<u32>,
    closed_issues: Option<u32>,
    pull_requests_merged: Option<u32>,
    pull_request_contributors: Option<u32>,
    commit_count_4_weeks: Option<u32>,
}

impl CoinGeckoProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            solana_tokens: HashMap::new(),
        }
    }

    async fn update_token_list(&mut self) -> Result<()> {
        let url = "https://api.coingecko.com/api/v3/coins/list?include_platform=true";
        
        let tokens: Vec<CoinGeckoTokenData> = self.client
            .get(url)
            .send()
            .await?
            .json()
            .await?;

        self.solana_tokens.clear();
        for token in tokens {
            if let Some(solana_addr) = token.platforms.get("solana") {
                self.solana_tokens.insert(solana_addr.clone(), token.id);
            }
        }

        Ok(())
    }

    async fn get_coingecko_id(&mut self, solana_address: &str) -> Result<String> {
        if self.solana_tokens.is_empty() {
            self.update_token_list().await?;
        }

        self.solana_tokens
            .get(solana_address)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Token not found in CoinGecko"))
    }

    async fn get_token_data(&mut self, solana_address: &str) -> Result<CoinGeckoTokenData> {
        let coingecko_id = self.get_coingecko_id(solana_address).await?;
        
        let url = format!(
            "https://api.coingecko.com/api/v3/coins/{}?localization=false&tickers=false&market_data=true&community_data=true&developer_data=true",
            coingecko_id
        );

        let data = self.client
            .get(&url)
            .send()
            .await?
            .json::<CoinGeckoTokenData>()
            .await?;

        Ok(data)
    }
}

#[async_trait]
impl DataProvider for CoinGeckoProvider {
    async fn get_token_metadata(&self, address: &str) -> Result<TokenMetadata> {
        let mut this = self.clone();
        let data = this.get_token_data(address).await?;
        
        Ok(TokenMetadata {
            address: address.to_string(),
            symbol: data.symbol,
            name: data.name,
            decimals: 9, // CoinGecko doesn't provide decimals
            price_usd: data.market_data.current_price.get("usd").copied().unwrap_or(0.0),
            price_sol: data.market_data.current_price.get("sol").copied().unwrap_or(0.0),
            volume_24h: data.market_data.total_volume.get("usd").copied().unwrap_or(0.0),
            market_cap: data.market_data.market_cap.get("usd").copied().unwrap_or(0.0),
            fully_diluted_market_cap: data.market_data.fully_diluted_valuation.get("usd").copied().unwrap_or(0.0),
            circulating_supply: data.market_data.circulating_supply.unwrap_or(0.0),
            total_supply: data.market_data.total_supply.unwrap_or(0.0),
        })
    }

    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let url = "https://api.coingecko.com/api/v3/search/trending";
        
        #[derive(Deserialize)]
        struct TrendingResponse {
            coins: Vec<TrendingCoin>,
        }

        #[derive(Deserialize)]
        struct TrendingCoin {
            item: CoinItem,
        }

        #[derive(Deserialize)]
        struct CoinItem {
            id: String,
            symbol: String,
            name: String,
            price_btc: f64,
        }

        let trending = self.client
            .get(url)
            .send()
            .await?
            .json::<TrendingResponse>()
            .await?;

        let mut trends = Vec::new();
        for coin in trending.coins.into_iter().take(limit) {
            if let Ok(data) = self.get_token_metadata(&coin.item.id).await {
                trends.push(MarketTrend {
                    token_address: data.address,
                    price_change_24h: 0.0, // CoinGecko trending doesn't provide this
                    volume_change_24h: 0.0,
                    social_volume_24h: 0,
                    dev_activity_24h: 0,
                    metadata: data,
                });
            }
        }

        Ok(trends)
    }

    async fn get_historical_prices(&self, address: &str) -> Result<Vec<PricePoint>> {
        let mut this = self.clone();
        let coingecko_id = this.get_coingecko_id(address).await?;
        
        let url = format!(
            "https://api.coingecko.com/api/v3/coins/{}/market_chart?vs_currency=usd&days=7&interval=hourly",
            coingecko_id
        );

        #[derive(Deserialize)]
        struct MarketChartResponse {
            prices: Vec<(i64, f64)>,
            total_volumes: Vec<(i64, f64)>,
        }

        let data = self.client
            .get(&url)
            .send()
            .await?
            .json::<MarketChartResponse>()
            .await?;

        let mut price_points = Vec::new();
        for (i, (timestamp, price)) in data.prices.iter().enumerate() {
            price_points.push(PricePoint {
                timestamp: DateTime::from_timestamp(*timestamp / 1000, 0)
                    .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?,
                price: *price,
                volume: data.total_volumes.get(i).map(|(_,v)| *v).unwrap_or(0.0),
            });
        }

        Ok(price_points)
    }

    async fn get_onchain_metrics(&self, _address: &str) -> Result<OnChainMetrics> {
        // CoinGecko doesn't provide detailed on-chain metrics
        Err(anyhow::anyhow!("On-chain metrics not available from CoinGecko"))
    }

    async fn get_social_metrics(&self, address: &str) -> Result<SocialMetrics> {
        let mut this = self.clone();
        let data = this.get_token_data(address).await?;
        
        let community = data.community_data.unwrap_or_else(|| CommunityData {
            twitter_followers: None,
            telegram_channel_user_count: None,
            reddit_subscribers: None,
        });

        let dev = data.developer_data.unwrap_or_else(|| DeveloperData {
            forks: None,
            stars: None,
            subscribers: None,
            total_issues: None,
            closed_issues: None,
            pull_requests_merged: None,
            pull_request_contributors: None,
            commit_count_4_weeks: None,
        });

        Ok(SocialMetrics {
            twitter_followers: community.twitter_followers.unwrap_or(0),
            twitter_engagement_24h: 0, // CoinGecko doesn't provide this
            discord_members: 0, // CoinGecko doesn't provide this
            telegram_members: community.telegram_channel_user_count.unwrap_or(0),
            github_stars: dev.stars.unwrap_or(0),
            sentiment_score: 0.0, // CoinGecko doesn't provide this
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coingecko_provider() {
        let provider = CoinGeckoProvider::new();
        
        // Test SOL token
        let sol_address = "So11111111111111111111111111111111111111112";
        
        let metadata = provider.get_token_metadata(sol_address).await.unwrap();
        assert_eq!(metadata.symbol.to_lowercase(), "sol");
        
        let trends = provider.get_trending_tokens(5).await.unwrap();
        assert!(!trends.is_empty());
        
        let prices = provider.get_historical_prices(sol_address).await.unwrap();
        assert!(!prices.is_empty());
        
        let social = provider.get_social_metrics(sol_address).await.unwrap();
        assert!(social.twitter_followers > 0);
    }
} 