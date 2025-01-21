use crate::personality::StoicPersonality;
use crate::market_data::{DataProvider, MarketTrend};
use crate::twitter::TwitterClient;
use crate::strategy::{TradeAction, TradeRecommendation, TradingStrategy};
use crate::dex::jupiter::JupiterDex;
use anyhow::Result;
use chrono::{DateTime, Utc};
use mongodb::{Database, bson::doc};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, warn};
use rig::completion::CompletionModel;
use solana_sdk::signature::Keypair;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenState {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct DataSyncService<M: CompletionModel> {
    db: Arc<Database>,
    data_provider: Box<dyn DataProvider>,
    twitter: TwitterClient,
    trading_strategy: TradingStrategy<M>,
    dex: JupiterDex,
    personality: StoicPersonality,
    wallet: Keypair,
    sync_interval: u64,
}

impl<M: CompletionModel> DataSyncService<M> {
    pub fn new(
        db: Database,
        data_provider: Box<dyn DataProvider>,
        twitter: TwitterClient,
        trading_strategy: TradingStrategy<M>,
        dex: JupiterDex,
        wallet: Keypair,
        sync_interval: u64,
    ) -> Self {
        Self {
            db: Arc::new(db),
            data_provider,
            twitter,
            trading_strategy,
            dex,
            personality: StoicPersonality::new(),
            wallet,
            sync_interval,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting data sync service");
        loop {
            if let Err(e) = self.sync_market_data().await {
                tracing::error!("Error syncing market data: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(self.sync_interval)).await;
        }
    }

    pub async fn sync_market_data(&self) -> Result<()> {
        info!("Starting market data sync cycle");
        
        // Fetch trending tokens
        info!("Fetching trending tokens from BirdEye");
        let trends = self.data_provider.get_trending_tokens(20).await?;
        info!("Found {} trending tokens", trends.len());

        // Insert token states and analyze trading opportunities
        for trend in trends {
            info!(
                "Processing token {} ({}) - Price: ${:.4}, 24h Change: {:.2}%, Volume: ${:.2}M",
                trend.metadata.name,
                trend.metadata.symbol,
                trend.metadata.price_usd,
                trend.price_change_24h,
                trend.metadata.volume_24h / 1_000_000.0
            );

            let state = self.market_trend_to_token_state(trend.clone());
            info!("Inserting token state into MongoDB");
            self.db.collection("token_states").insert_one(state, None).await?;

            // Format market data for LLM analysis
            let prompt = format!(
                "Analyze trading opportunity for {} ({}). Price: ${:.4}, 24h Change: {:.2}%, Volume: ${:.2}M",
                trend.metadata.name,
                trend.metadata.symbol,
                trend.metadata.price_usd,
                trend.price_change_24h,
                trend.metadata.volume_24h / 1_000_000.0
            );

            // Analyze trading opportunity
            info!("Analyzing trading opportunity with LLM");
            if let Ok(analysis) = self.trading_strategy.analyze_trading_opportunity(&prompt, 1.0).await {
                // Parse the analysis into a trade recommendation
                if let Ok(trade) = serde_json::from_str::<TradeRecommendation>(&analysis) {
                    info!(
                        "Received trade recommendation: Action={:?}, Amount={} SOL, Confidence={:.2}, Risk={}",
                        trade.action, trade.amount_in_sol, trade.confidence, trade.risk_assessment
                    );
                    
                    // Execute trade if confidence is high enough
                    if trade.confidence >= 0.8 {
                        match trade.action {
                            TradeAction::Buy => {
                                info!("Executing BUY order for {} SOL worth of {}", 
                                    trade.amount_in_sol, trend.metadata.symbol);
                                
                                if let Ok(signature) = self.dex.execute_swap(
                                    "So11111111111111111111111111111111111111112", // SOL
                                    &trade.token_address,
                                    trade.amount_in_sol as u64,
                                    &self.wallet,
                                ).await {
                                    info!("Trade executed successfully. Signature: {}", signature);

                                    // Generate and post tweet about the trade
                                    info!("Generating tweet for successful buy");
                                    let tweet = self.personality.generate_trade_tweet(
                                        &self.trading_strategy.agent,
                                        &format!(
                                            "Action: Buy
Amount: {} SOL
Token: {}
Price: ${:.4}
Market Cap: ${:.2}M
24h Volume: ${:.2}M
24h Change: {:.2}%
Contract: {}
Transaction: {}
Analysis: {}
Risk Assessment: {}
Market Analysis:
- Volume: {}
- Price Trend: {}
- Liquidity: {}
- Momentum: {}",
                                            trade.amount_in_sol,
                                            trend.metadata.symbol,
                                            trend.metadata.price_usd,
                                            trend.metadata.market_cap / 1_000_000.0,
                                            trend.metadata.volume_24h / 1_000_000.0,
                                            trend.price_change_24h,
                                            trend.token_address,
                                            signature,
                                            trade.reasoning,
                                            trade.risk_assessment,
                                            trade.market_analysis.volume_analysis,
                                            trade.market_analysis.price_trend,
                                            trade.market_analysis.liquidity_assessment,
                                            trade.market_analysis.momentum_indicators
                                        ),
                                    ).await?;
                                    
                                    info!("Posting tweet: {}", tweet);
                                    if let Err(e) = self.twitter.post_tweet(&tweet).await {
                                        warn!("Failed to post trade tweet: {}", e);
                                    }
                                } else {
                                    warn!("Failed to execute buy order");
                                }
                            },
                            TradeAction::Sell => {
                                info!("Executing SELL order for {} SOL worth of {}", 
                                    trade.amount_in_sol, trend.metadata.symbol);
                                
                                if let Ok(signature) = self.dex.execute_swap(
                                    &trade.token_address,
                                    "So11111111111111111111111111111111111111112", // SOL
                                    trade.amount_in_sol as u64,
                                    &self.wallet,
                                ).await {
                                    info!("Trade executed successfully. Signature: {}", signature);

                                    // Generate and post tweet about the trade
                                    info!("Generating tweet for successful sell");
                                    let tweet = self.personality.generate_trade_tweet(
                                        &self.trading_strategy.agent,
                                        &format!(
                                            "Action: Sell
Amount: {} SOL
Token: {}
Price: ${:.4}
Market Cap: ${:.2}M
24h Volume: ${:.2}M
24h Change: {:.2}%
Contract: {}
Transaction: {}
Analysis: {}
Risk Assessment: {}
Market Analysis:
- Volume: {}
- Price Trend: {}
- Liquidity: {}
- Momentum: {}",
                                            trade.amount_in_sol,
                                            trend.metadata.symbol,
                                            trend.metadata.price_usd,
                                            trend.metadata.market_cap / 1_000_000.0,
                                            trend.metadata.volume_24h / 1_000_000.0,
                                            trend.price_change_24h,
                                            trend.token_address,
                                            signature,
                                            trade.reasoning,
                                            trade.risk_assessment,
                                            trade.market_analysis.volume_analysis,
                                            trade.market_analysis.price_trend,
                                            trade.market_analysis.liquidity_assessment,
                                            trade.market_analysis.momentum_indicators
                                        ),
                                    ).await?;
                                    
                                    info!("Posting tweet: {}", tweet);
                                    if let Err(e) = self.twitter.post_tweet(&tweet).await {
                                        warn!("Failed to post trade tweet: {}", e);
                                    }
                                } else {
                                    warn!("Failed to execute sell order");
                                }
                            },
                            TradeAction::Hold => {
                                info!("Decision: HOLD {} - {}", 
                                    trend.metadata.symbol, trade.reasoning);
                            }
                        }
                    } else {
                        info!("Skipping trade due to low confidence: {:.2}", trade.confidence);
                    }
                } else {
                    warn!("Failed to parse trade recommendation");
                }
            } else {
                warn!("Failed to get trading analysis from LLM");
            }
        }

        info!("Market data sync cycle complete");
        Ok(())
    }

    fn market_trend_to_token_state(&self, trend: MarketTrend) -> TokenState {
        TokenState {
            address: trend.token_address,
            symbol: trend.metadata.symbol,
            name: trend.metadata.name,
            price_usd: trend.metadata.price_usd,
            price_sol: trend.metadata.price_sol,
            volume_24h: trend.metadata.volume_24h,
            market_cap: trend.metadata.market_cap,
            price_change_24h: trend.price_change_24h,
            volume_change_24h: trend.volume_change_24h,
            timestamp: Utc::now(),
        }
    }
} 