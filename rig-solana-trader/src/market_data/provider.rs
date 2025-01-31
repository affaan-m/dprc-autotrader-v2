use rig_core::{
    providers::{
        DataProvider,
        openai::Client as OpenAIClient,
        twitter::TwitterClient,
        solana::SolanaClient,
    },
    Result,
};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use crate::vector_store::TokenVectorStore;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenMetadata {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub market_cap: Option<f64>,
    pub volume_24h: Option<f64>,
    pub price_usd: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketData {
    pub token: TokenMetadata,
    pub price_history: Vec<PricePoint>,
    pub social_sentiment: Option<f64>,
    pub technical_indicators: TechnicalIndicators,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PricePoint {
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TechnicalIndicators {
    pub rsi_14: Option<f64>,
    pub macd: Option<f64>,
    pub ma_50: Option<f64>,
    pub ma_200: Option<f64>,
}

pub struct MarketDataProvider {
    openai_client: OpenAIClient,
    twitter_client: TwitterClient,
    solana_client: SolanaClient,
    vector_store: TokenVectorStore,
}

impl MarketDataProvider {
    pub async fn new(
        openai_api_key: &str,
        twitter_bearer_token: &str,
        solana_rpc_url: &str,
        vector_store: TokenVectorStore,
    ) -> Result<Self> {
        Ok(Self {
            openai_client: OpenAIClient::new(openai_api_key),
            twitter_client: TwitterClient::new(twitter_bearer_token),
            solana_client: SolanaClient::new(solana_rpc_url),
            vector_store,
        })
    }

    pub async fn get_token_metadata(&self, token_address: &str) -> Result<TokenMetadata> {
        debug!("Fetching metadata for token {}", token_address);
        
        // Get on-chain data
        let mint = self.solana_client.get_mint(token_address).await?;
        
        // Get market data from external sources
        let market_data = self.solana_client.get_token_market_data(token_address).await?;
        
        Ok(TokenMetadata {
            address: token_address.to_string(),
            symbol: mint.symbol,
            name: mint.name,
            decimals: mint.decimals,
            total_supply: mint.supply,
            market_cap: market_data.market_cap,
            volume_24h: market_data.volume_24h,
            price_usd: market_data.price_usd,
        })
    }

    pub async fn get_market_data(&self, token_address: &str) -> Result<MarketData> {
        debug!("Fetching market data for token {}", token_address);
        
        // Get token metadata
        let token = self.get_token_metadata(token_address).await?;
        
        // Get price history
        let price_history = self.solana_client
            .get_token_price_history(token_address)
            .await?;
            
        // Get social sentiment
        let social_sentiment = self.analyze_social_sentiment(&token.symbol).await?;
        
        // Calculate technical indicators
        let technical_indicators = self.calculate_technical_indicators(&price_history)?;
        
        Ok(MarketData {
            token,
            price_history,
            social_sentiment,
            technical_indicators,
        })
    }

    async fn analyze_social_sentiment(&self, symbol: &str) -> Result<Option<f64>> {
        debug!("Analyzing social sentiment for {}", symbol);
        
        // Get recent tweets
        let tweets = self.twitter_client
            .search_tweets(&format!("${}", symbol))
            .await?;
            
        if tweets.is_empty() {
            return Ok(None);
        }
        
        // Analyze sentiment using OpenAI
        let sentiment = self.openai_client
            .analyze_sentiment(&tweets.join("\n"))
            .await?;
            
        Ok(Some(sentiment))
    }

    fn calculate_technical_indicators(&self, price_history: &[PricePoint]) -> Result<TechnicalIndicators> {
        if price_history.is_empty() {
            return Ok(TechnicalIndicators::default());
        }
        
        // Calculate indicators
        let prices: Vec<f64> = price_history.iter().map(|p| p.price).collect();
        
        Ok(TechnicalIndicators {
            rsi_14: Some(self.calculate_rsi(&prices, 14)?),
            macd: Some(self.calculate_macd(&prices)?),
            ma_50: Some(self.calculate_moving_average(&prices, 50)?),
            ma_200: Some(self.calculate_moving_average(&prices, 200)?),
        })
    }

    fn calculate_rsi(&self, prices: &[f64], period: usize) -> Result<f64> {
        // TODO: Implement RSI calculation
        Ok(50.0)
    }

    fn calculate_macd(&self, prices: &[f64]) -> Result<f64> {
        // TODO: Implement MACD calculation
        Ok(0.0)
    }

    fn calculate_moving_average(&self, prices: &[f64], period: usize) -> Result<f64> {
        if prices.len() < period {
            return Ok(prices.last().copied().unwrap_or_default());
        }
        
        let sum: f64 = prices.iter().rev().take(period).sum();
        Ok(sum / period as f64)
    }
} 