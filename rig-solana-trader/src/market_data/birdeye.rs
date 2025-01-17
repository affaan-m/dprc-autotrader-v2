use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use reqwest::Client;
use std::collections::HashMap;
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

    async fn get_trending_by_rank(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/defi/token_trending?sort_by=rank&sort_type=asc&offset=0&limit={}",
            limit
        );
        self.get_trending_tokens_internal(&url).await
    }

    async fn get_trending_by_volume(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/defi/token_trending?sort_by=volume24hUSD&sort_type=asc&offset=0&limit={}",
            limit
        );
        self.get_trending_tokens_internal(&url).await
    }

    async fn get_trending_by_liquidity(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/defi/token_trending?sort_by=liquidity&sort_type=asc&offset=0&limit={}",
            limit
        );
        self.get_trending_tokens_internal(&url).await
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

    async fn get_token_list_by_volume(&self, limit: usize, min_liquidity: f64) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/defi/tokenlist?sort_by=v24hUSD&sort_type=desc&offset=0&limit={}&min_liquidity={}",
            limit, min_liquidity
        );
        self.get_token_list_internal(&url).await
    }

    async fn get_token_list_by_market_cap(&self, limit: usize, min_liquidity: f64) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/defi/tokenlist?sort_by=mc&sort_type=desc&offset=0&limit={}&min_liquidity={}",
            limit, min_liquidity
        );
        self.get_token_list_internal(&url).await
    }

    async fn get_token_list_by_price_change(&self, limit: usize, min_liquidity: f64) -> Result<Vec<MarketTrend>> {
        let url = format!(
            "https://public-api.birdeye.so/defi/tokenlist?sort_by=v24hChangePercent&sort_type=desc&offset=0&limit={}&min_liquidity={}",
            limit, min_liquidity
        );
        self.get_token_list_internal(&url).await
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

    async fn get_trending_tokens_internal(&self, url: &str) -> Result<Vec<MarketTrend>> {
        let response = self.client
            .get(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<BirdEyeTrendingResponse>()
            .await?;

        Ok(response.data.tokens.into_iter().map(|token| MarketTrend {
            token_address: token.address.clone(),
            metadata: TokenMetadata {
                address: token.address,
                symbol: token.symbol,
                name: token.name,
                decimals: token.decimals,
                price_usd: token.price,
                price_sol: token.price,
                volume_24h: token.volume_24h_usd.unwrap_or(0.0),
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
            .json::<serde_json::Value>()
            .await?;

        let data = &response["data"][token_address];
        
        Ok(TokenMetadata {
            address: data["address"].as_str().unwrap_or("").to_string(),
            symbol: data["symbol"].as_str().unwrap_or("").to_string(),
            name: data["name"].as_str().unwrap_or("").to_string(),
            decimals: data["decimals"].as_u64().unwrap_or(0) as u8,
            price_usd: 0.0, // Need to fetch from market data endpoint
            price_sol: 0.0,
            volume_24h: 0.0,
            market_cap: 0.0,
            fully_diluted_market_cap: 0.0,
            circulating_supply: 0.0,
            total_supply: 0.0,
        })
    }

    async fn get_trending_tokens(&self, limit: usize) -> Result<Vec<MarketTrend>> {
        let mut all_trends = Vec::new();

        // Collect trends from all sorting methods
        if let Ok(mut trends) = self.get_trending_by_rank(limit).await {
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_trending_by_volume(limit).await {
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_trending_by_liquidity(limit).await {
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_new_listings(limit).await {
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_token_list_by_volume(limit, 100.0).await {
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_token_list_by_market_cap(limit, 100.0).await {
            all_trends.append(&mut trends);
        }
        if let Ok(mut trends) = self.get_token_list_by_price_change(limit, 100.0).await {
            all_trends.append(&mut trends);
        }

        // Deduplicate by token address and take the first 'limit' entries
        let mut unique_trends = HashMap::new();
        for trend in all_trends {
            unique_trends.entry(trend.token_address.clone())
                .or_insert(trend);
        }

        Ok(unique_trends.into_values().take(limit).collect())
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