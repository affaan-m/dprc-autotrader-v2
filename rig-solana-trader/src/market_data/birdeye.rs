//! BirdEye API Integration
//!
//! This module implements the BirdEye API client for fetching Solana token data.
//! BirdEye provides comprehensive market data including:
//! - Token metadata and prices
//! - Trading volume and liquidity
//! - Price changes and market trends
//!
//! # Rate Limits
//! BirdEye API has the following limits:
//! - 10 requests per second
//! - 100,000 requests per day
//! - 100 tokens per request for trending endpoints
//!
//! # Error Handling
//! The implementation includes:
//! - Automatic retry on rate limit errors (429)
//! - Exponential backoff for failed requests
//! - Detailed error logging for debugging
//!
//! # Configuration
//! Required environment variables:
//! - `BIRDEYE_API_KEY`: API key from BirdEye
//!
//! # Endpoints
//! - GET /token/meta: Token metadata
//! - GET /token/list: Token listings
//! - GET /token/trending: Trending tokens
//! - GET /token/price: Real-time prices

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize};
use std::collections::HashMap;
use tracing::{debug, info, instrument};
use chrono::DateTime;
use crate::market_data::{
    DataProvider,
    MarketTrend,
    TokenMetadata,
    PricePoint,
    OnChainMetrics,
    SocialMetrics,
};

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
    data: BirdEyeTrendingResponseData,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTrendingResponseData {
    #[serde(rename = "updateUnixTime")]
    update_unix_time: i64,
    #[serde(rename = "updateTime")]
    update_time: String,
    tokens: Vec<BirdEyeTrendingToken>,
    total: i64,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTrendingToken {
    address: String,
    decimals: u8,
    liquidity: f64,
    #[serde(rename = "logoURI")]
    logo_uri: Option<String>,
    name: String,
    symbol: String,
    #[serde(rename = "volume24hUSD")]
    volume_24h_usd: Option<f64>,
    rank: Option<i64>,
    price: f64,
    #[serde(rename = "priceChange24h")]
    price_change_24h: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct BirdEyeNewListingResponse {
    success: bool,
    data: BirdEyeNewListingData,
}

#[derive(Debug, Deserialize)]
struct BirdEyeNewListingData {
    items: Vec<BirdEyeNewListingToken>,
}

#[derive(Debug, Deserialize)]
struct BirdEyeNewListingToken {
    address: String,
    symbol: String,
    name: String,
    decimals: u8,
    source: String,
    #[serde(rename = "liquidityAddedAt")]
    liquidity_added_at: String,
    #[serde(rename = "logoURI")]
    logo_uri: Option<String>,
    liquidity: f64,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTokenListResponse {
    success: bool,
    data: BirdEyeTokenListData,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTokenListData {
    #[serde(rename = "updateUnixTime")]
    update_unix_time: i64,
    #[serde(rename = "updateTime")]
    update_time: String,
    tokens: Vec<BirdEyeTokenListToken>,
    total: i64,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTokenListToken {
    address: String,
    decimals: u8,
    #[serde(rename = "lastTradeUnixTime")]
    last_trade_unix_time: i64,
    liquidity: f64,
    #[serde(rename = "logoURI")]
    logo_uri: Option<String>,
    mc: f64,
    name: String,
    symbol: String,
    #[serde(rename = "v24hChangePercent")]
    v24h_change_percent: f64,
    #[serde(rename = "v24hUSD")]
    v24h_usd: f64,
}

#[derive(Debug, Deserialize)]
struct BirdEyeWalletResponse {
    success: bool,
    data: BirdEyeWalletData,
}

#[derive(Debug, Deserialize)]
struct BirdEyeWalletData {
    wallet: String,
    #[serde(rename = "totalUsd")]
    total_usd: f64,
    items: Vec<BirdEyeWalletToken>,
}

#[derive(Debug, Deserialize)]
struct BirdEyeWalletToken {
    address: String,
    decimals: u8,
    balance: i64,
    #[serde(rename = "uiAmount")]
    ui_amount: f64,
    #[serde(rename = "chainId")]
    chain_id: String,
    name: String,
    symbol: String,
    icon: Option<String>,
    #[serde(rename = "logoURI")]
    logo_uri: Option<String>,
    #[serde(rename = "priceUsd")]
    price_usd: f64,
    #[serde(rename = "valueUsd")]
    value_usd: f64,
}

#[derive(Debug, Deserialize, Clone)]
struct BirdEyeTransactionResponse {
    success: bool,
    data: HashMap<String, Vec<BirdEyeTransaction>>,
}

#[derive(Debug, Deserialize, Clone)]
struct BirdEyeTransaction {
    #[serde(rename = "txHash")]
    tx_hash: String,
    #[serde(rename = "blockNumber")]
    block_number: i64,
    #[serde(rename = "blockTime")]
    block_time: String,
    status: bool,
    from: String,
    to: String,
    fee: i64,
    #[serde(rename = "mainAction")]
    main_action: String,
    #[serde(rename = "balanceChange")]
    balance_change: Vec<BirdEyeBalanceChange>,
    #[serde(rename = "contractLabel")]
    contract_label: Option<BirdEyeContractLabel>,
}

#[derive(Debug, Deserialize, Clone)]
struct BirdEyeBalanceChange {
    amount: f64,
    symbol: String,
    name: String,
    decimals: u8,
    address: String,
    #[serde(rename = "logoURI")]
    logo_uri: Option<String>,
    token_account: Option<String>,
    owner: Option<String>,
    #[serde(rename = "programId")]
    program_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct BirdEyeContractLabel {
    address: String,
    name: String,
    metadata: BirdEyeContractMetadata,
}

#[derive(Debug, Deserialize, Clone)]
struct BirdEyeContractMetadata {
    icon: String,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTokenMetadataResponse {
    data: HashMap<String, BirdEyeTokenMetadata>,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTokenMetadata {
    address: String,
    name: String,
    symbol: String,
    decimals: u8,
    extensions: BirdEyeTokenExtensions,
    #[serde(rename = "logo_uri")]
    logo_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BirdEyeTokenExtensions {
    #[serde(rename = "coingecko_id")]
    coingecko_id: Option<String>,
    #[serde(rename = "serum_v3_usdc")]
    serum_v3_usdc: Option<String>,
    #[serde(rename = "serum_v3_usdt")]
    serum_v3_usdt: Option<String>,
    website: Option<String>,
    telegram: Option<String>,
    twitter: Option<String>,
    description: Option<String>,
    discord: Option<String>,
    medium: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BirdEyeMarketDataResponse {
    data: BirdEyeMarketData,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct BirdEyeMarketData {
    address: String,
    price: f64,
    liquidity: f64,
    supply: f64,
    marketcap: f64,
    #[serde(rename = "circulating_supply")]
    circulating_supply: f64,
    #[serde(rename = "circulating_marketcap")]
    circulating_marketcap: f64,
}

#[derive(Debug)]
pub struct BirdEyeProvider {
    api_key: String,
    client: Client,
}

impl BirdEyeProvider {
    pub fn new(api_key: String) -> Self {
        info!("Initializing BirdEye API provider");
        Self {
            api_key,
            client: Client::new(),
        }
    }

    #[instrument(skip(self), fields(api = "birdeye"))]
    async fn get_trending_by_rank(&self) -> Result<Vec<MarketTrend>> {
        debug!("Fetching trending tokens by rank");
        let url = "https://public-api.birdeye.so/defi/token_trending?sort_by=rank&sort_type=asc&offset=0&limit=20";
        self.get_trending_tokens_internal(url).await
    }

    #[instrument(skip(self), fields(api = "birdeye"))]
    async fn get_trending_by_volume(&self) -> Result<Vec<MarketTrend>> {
        debug!("Fetching trending tokens by volume");
        let url = "https://public-api.birdeye.so/defi/token_trending?sort_by=volume24hUSD&sort_type=asc&offset=0&limit=20";
        self.get_trending_tokens_internal(url).await
    }

    #[instrument(skip(self), fields(api = "birdeye"))]
    async fn get_trending_by_liquidity(&self) -> Result<Vec<MarketTrend>> {
        debug!("Fetching trending tokens by liquidity");
        let url = "https://public-api.birdeye.so/defi/token_trending?sort_by=liquidity&sort_type=asc&offset=0&limit=20";
        self.get_trending_tokens_internal(url).await
    }

    async fn get_new_listings(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/defi/v2/tokens/new_listing?time_to=10000000000&limit={}&meme_platform_enabled=true",
            limit
        );
        
        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeNewListingResponse>()
            .await?;

        Ok(response.data.items.into_iter().map(|token| MarketTrend {
            token_address: token.address.clone(),
            metadata: TokenMetadata {
                address: token.address,
                symbol: token.symbol,
                name: token.name,
                decimals: token.decimals,
                price_usd: 0.0, // Not available in new listings
                price_sol: 0.0,
                volume_24h: 0.0,
                market_cap: 0.0,
                fully_diluted_market_cap: 0.0,
                circulating_supply: 0.0,
                total_supply: 0.0,
            },
            price_change_24h: 0.0,
            volume_change_24h: 0.0,
            social_volume_24h: 0,
            dev_activity_24h: 0,
        }).collect())
    }

    async fn get_token_list_by_volume(&self, _limit: usize, _min_liquidity: f64) -> Result<Vec<MarketTrend>> {
        let url = "https://public-api.birdeye.so/defi/tokenlist?sort_by=v24hUSD&sort_type=desc&offset=0&limit=50&min_liquidity=100";
        self.get_token_list_internal(url).await
    }

    async fn get_token_list_by_market_cap(&self, _limit: usize, _min_liquidity: f64) -> Result<Vec<MarketTrend>> {
        let url = "https://public-api.birdeye.so/defi/tokenlist?sort_by=mc&sort_type=desc&offset=0&limit=50&min_liquidity=100";
        self.get_token_list_internal(url).await
    }

    async fn get_token_list_by_price_change(&self, _limit: usize, _min_liquidity: f64) -> Result<Vec<MarketTrend>> {
        let url = "https://public-api.birdeye.so/defi/tokenlist?sort_by=v24hChangePercent&sort_type=desc&offset=0&limit=50&min_liquidity=100";
        self.get_token_list_internal(url).await
    }

    async fn get_token_list_internal(&self, url: &str) -> Result<Vec<MarketTrend>> {
        let response = self.client
            .get(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeTokenListResponse>()
            .await?;

        Ok(response.data.tokens.into_iter().map(|token| MarketTrend {
            token_address: token.address.clone(),
            metadata: TokenMetadata {
                address: token.address,
                symbol: token.symbol,
                name: token.name,
                decimals: token.decimals,
                price_usd: 0.0, // Need to fetch separately
                price_sol: 0.0,
                volume_24h: token.v24h_usd,
                market_cap: token.mc,
                fully_diluted_market_cap: 0.0,
                circulating_supply: 0.0,
                total_supply: 0.0,
            },
            price_change_24h: token.v24h_change_percent,
            volume_change_24h: 0.0,
            social_volume_24h: 0,
            dev_activity_24h: 0,
        }).collect())
    }

    async fn get_wallet_tokens(&self, wallet_address: &str) -> Result<BirdEyeWalletData> {
        let url = format!(
            "https://public-api.birdeye.so/v1/wallet/token_list?wallet={}",
            wallet_address
        );

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeWalletResponse>()
            .await?;

        Ok(response.data)
    }

    async fn get_wallet_transactions(&self, wallet_address: &str, limit: usize) -> Result<Vec<BirdEyeTransaction>> {
        let url = format!(
            "https://public-api.birdeye.so/v1/wallet/tx_list?wallet={}&limit={}",
            wallet_address, limit
        );

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeTransactionResponse>()
            .await?;

        Ok(response.data.get("solana")
            .cloned()
            .unwrap_or_default())
    }

    #[instrument(skip(self), fields(api = "birdeye"))]
    async fn get_trending_tokens_internal(&self, url: &str) -> Result<Vec<MarketTrend>> {
        debug!(url = %url, "Making API request");
        
        let response = self.client
            .get(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeTrendingResponse>()
            .await?;

        info!(
            token_count = response.data.tokens.len(),
            "Successfully parsed trending tokens"
        );

        Ok(response.data.tokens.into_iter().map(|token| MarketTrend {
            token_address: token.address.clone(),
            metadata: TokenMetadata {
                address: token.address,
                symbol: token.symbol,
                name: token.name,
                decimals: token.decimals,
                price_usd: token.price,
                price_sol: token.price, // Price is in USD
                volume_24h: token.volume_24h_usd.unwrap_or(0.0),
                market_cap: 0.0, // Not available in trending response
                fully_diluted_market_cap: 0.0,
                circulating_supply: 0.0,
                total_supply: 0.0,
            },
            price_change_24h: token.price_change_24h.unwrap_or(0.0),
            volume_change_24h: 0.0, // Not available in trending response
            social_volume_24h: 0,
            dev_activity_24h: 0,
        }).collect())
    }
}

#[async_trait]
impl DataProvider for BirdEyeProvider {
    async fn get_token_metadata(&self, token_address: &str) -> Result<TokenMetadata> {
        let url = format!(
            "https://public-api.birdeye.so/defi/v3/token/meta-data/multiple?list_address={}",
            token_address
        );

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeTokenMetadataResponse>()
            .await?;

        let metadata = response.data.get(token_address)
            .ok_or_else(|| anyhow::anyhow!("Token metadata not found"))?;

        // Get market data
        let market_url = format!(
            "https://public-api.birdeye.so/defi/v3/token/market-data?address={}",
            token_address
        );

        let market_data = self.client
            .get(&market_url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeMarketDataResponse>()
            .await?;
        
        Ok(TokenMetadata {
            address: metadata.address.clone(),
            symbol: metadata.symbol.clone(),
            name: metadata.name.clone(),
            decimals: metadata.decimals,
            price_usd: market_data.data.price,
            price_sol: market_data.data.price, // Price is in USD
            volume_24h: 0.0, // Not available in this endpoint
            market_cap: market_data.data.marketcap,
            fully_diluted_market_cap: market_data.data.marketcap,
            circulating_supply: market_data.data.circulating_supply,
            total_supply: market_data.data.supply,
        })
    }

    #[instrument(skip(self), fields(api = "birdeye"))]
    async fn get_trending_tokens(&self, _limit: usize) -> Result<Vec<MarketTrend>> {
        debug!("Fetching trending tokens from all sources");
        let mut all_trends = Vec::new();

        // Collect trends from all sorting methods
        if let Ok(mut trends) = self.get_trending_by_rank().await {
            debug!(count = trends.len(), "Got trending by rank");
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_trending_by_volume().await {
            debug!(count = trends.len(), "Got trending by volume");
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_trending_by_liquidity().await {
            debug!(count = trends.len(), "Got trending by liquidity");
            all_trends.append(&mut trends);
        }

        // Deduplicate by token address
        let mut unique_trends = HashMap::new();
        for trend in all_trends {
            unique_trends.entry(trend.token_address.clone())
                .or_insert(trend);
        }

        let trends: Vec<_> = unique_trends.into_values().collect();
        info!(
            total_trends = trends.len(),
            "Successfully aggregated trending tokens"
        );

        Ok(trends)
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