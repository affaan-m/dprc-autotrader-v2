use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use reqwest::Client;
use super::{DataProvider, MarketTrend, TokenMetadata, PricePoint, OnChainMetrics, SocialMetrics};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
struct BirdEyeTokenResponse {
    data: BirdEyeTokenData,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTokenData {
    address: String,
    symbol: String,
    name: String,
    price: f64,
    volume_24h: f64,
    decimals: u8,
    price_sol: f64,
    market_cap: f64,
    fully_diluted_market_cap: Option<f64>,
    circulating_supply: Option<f64>,
    total_supply: Option<f64>,
    price_change_24h: Option<f64>,
    volume_change_24h: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTrendingResponse {
    data: Vec<BirdEyeTrendingData>,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTrendingData {
    address: String,
    symbol: String,
    name: String,
    price: f64,
    volume_24h: f64,
    price_change_24h: Option<f64>,
    volume_change_24h: Option<f64>,
}

pub struct BirdEyeProvider {
    api_key: String,
    client: Client,
}

impl BirdEyeProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    async fn get_trending_tokens(&self, url: &str) -> Result<Vec<MarketTrend>> {
        let response = self.client
            .get(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeTrendingResponse>()
            .await?;

        Ok(response.data.into_iter().map(|data| MarketTrend {
            token_address: data.address.clone(),
            metadata: TokenMetadata {
                address: data.address,
                symbol: data.symbol,
                name: data.name,
                decimals: 9, // Default for most Solana tokens
                price_usd: data.price,
                price_sol: data.price,
                volume_24h: data.volume_24h,
                market_cap: 0.0, // Not available in trending data
                fully_diluted_market_cap: 0.0,
                circulating_supply: 0.0,
                total_supply: 0.0,
            },
            price_change_24h: data.price_change_24h.unwrap_or(0.0),
            volume_change_24h: data.volume_change_24h.unwrap_or(0.0),
            social_volume_24h: 0,
            dev_activity_24h: 0,
        }).collect())
    }
}

#[async_trait]
impl DataProvider for BirdEyeProvider {
    async fn get_token_metadata(&self, token_address: &str) -> Result<TokenMetadata> {
        let url = format!(
            "https://public-api.birdeye.so/public/token_metadata?address={}",
            token_address
        );

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeTokenResponse>()
            .await?;

        let data = response.data;
        Ok(TokenMetadata {
            address: data.address,
            symbol: data.symbol,
            name: data.name,
            decimals: data.decimals,
            price_usd: data.price,
            price_sol: data.price_sol,
            volume_24h: data.volume_24h,
            market_cap: data.market_cap,
            fully_diluted_market_cap: data.fully_diluted_market_cap.unwrap_or(0.0),
            circulating_supply: data.circulating_supply.unwrap_or(0.0),
            total_supply: data.total_supply.unwrap_or(0.0),
        })
    }

    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/public/trending_tokens?limit={}",
            limit
        );
        self.get_trending_tokens(&url).await
    }

    async fn get_historical_prices(&self, address: &str) -> Result<Vec<PricePoint>> {
        let url = format!(
            "https://public-api.birdeye.so/public/price_history?address={}&type=hour&limit=168",
            address
        );

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let data = response["data"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        let prices: Vec<PricePoint> = data.iter()
            .filter_map(|point| {
                let timestamp = point["timestamp"].as_i64()?;
                let price = point["value"].as_f64()?;
                let volume = point["volume"].as_f64().unwrap_or(0.0);

                Some(PricePoint {
                    timestamp: DateTime::from_timestamp(timestamp, 0)?,
                    price,
                    volume,
                })
            })
            .collect();

        Ok(prices)
    }

    async fn get_onchain_metrics(&self, address: &str) -> Result<OnChainMetrics> {
        let url = format!(
            "https://public-api.birdeye.so/public/token_holders?address={}",
            address
        );

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let data = response["data"].as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        Ok(OnChainMetrics {
            unique_holders: data["unique_holders"].as_u64().unwrap_or(0) as u32,
            active_wallets_24h: data["active_wallets_24h"].as_u64().unwrap_or(0) as u32,
            transactions_24h: data["transactions_24h"].as_u64().unwrap_or(0) as u32,
            average_transaction_size: data["avg_transaction_size"].as_f64().unwrap_or(0.0),
            whale_transactions_24h: data["whale_transactions_24h"].as_u64().unwrap_or(0) as u32,
        })
    }

    async fn get_social_metrics(&self, _address: &str) -> Result<SocialMetrics> {
        // BirdEye doesn't provide social metrics
        Err(anyhow::anyhow!("Social metrics not available from BirdEye"))
    }
} 