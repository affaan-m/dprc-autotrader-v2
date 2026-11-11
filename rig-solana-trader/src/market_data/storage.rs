use super::{TokenMetadata, MarketTrend, PricePoint, OnChainMetrics, SocialMetrics};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub metadata: TokenMetadata,
    pub price_history: Vec<PricePoint>,
    pub onchain_metrics: Option<OnChainMetrics>,
    pub social_metrics: Option<SocialMetrics>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub timestamp: DateTime<Utc>,
    pub trends: Vec<MarketTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_address: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub entry_time: DateTime<Utc>,
    pub partial_sells: Vec<PartialSell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialSell {
    pub price: f64,
    pub quantity: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct MarketDataStorage {
    token_data: Arc<RwLock<HashMap<String, TokenData>>>,
    market_snapshots: Arc<RwLock<Vec<MarketSnapshot>>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    max_snapshots: usize,
    data_dir: PathBuf,
}

impl MarketDataStorage {
    pub fn new(max_snapshots: usize) -> Self {
        let data_dir = PathBuf::from("data");
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        }

        let mut storage = Self {
            token_data: Arc::new(RwLock::new(HashMap::new())),
            market_snapshots: Arc::new(RwLock::new(Vec::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            max_snapshots,
            data_dir,
        };

        storage.load_from_disk();
        storage
    }

    fn load_from_disk(&mut self) {
        self.load_token_data();
        self.load_market_snapshots();
        self.load_positions();
    }

    fn load_token_data(&self) {
        let path = self.data_dir.join("token_data.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<HashMap<String, TokenData>>(&content) {
                    let mut token_data = self.token_data.blocking_write();
                    *token_data = data;
                }
            }
        }
    }

    fn load_market_snapshots(&self) {
        let path = self.data_dir.join("market_snapshots.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<Vec<MarketSnapshot>>(&content) {
                    let mut snapshots = self.market_snapshots.blocking_write();
                    *snapshots = data;
                }
            }
        }
    }

    fn load_positions(&self) {
        let path = self.data_dir.join("positions.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<HashMap<String, Position>>(&content) {
                    let mut positions = self.positions.blocking_write();
                    *positions = data;
                }
            }
        }
    }

    async fn save_to_disk(&self) -> Result<()> {
        self.save_token_data().await?;
        self.save_market_snapshots().await?;
        self.save_positions().await?;
        Ok(())
    }

    async fn save_token_data(&self) -> Result<()> {
        let path = self.data_dir.join("token_data.json");
        let token_data = self.token_data.read().await;
        let content = serde_json::to_string_pretty(&*token_data)?;
        fs::write(&path, content)?;
        Ok(())
    }

    async fn save_market_snapshots(&self) -> Result<()> {
        let path = self.data_dir.join("market_snapshots.json");
        let snapshots = self.market_snapshots.read().await;
        let content = serde_json::to_string_pretty(&*snapshots)?;
        fs::write(&path, content)?;
        Ok(())
    }

    async fn save_positions(&self) -> Result<()> {
        let path = self.data_dir.join("positions.json");
        let positions = self.positions.read().await;
        let content = serde_json::to_string_pretty(&*positions)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub async fn add_position(&self, position: Position) -> Result<()> {
        let mut positions = self.positions.write().await;
        positions.insert(position.token_address.clone(), position);
        drop(positions);
        self.save_positions().await?;
        Ok(())
    }

    pub async fn update_position(&self, token_address: &str, partial_sell: PartialSell) -> Result<()> {
        let mut positions = self.positions.write().await;
        if let Some(position) = positions.get_mut(token_address) {
            position.partial_sells.push(partial_sell);
            drop(positions);
            self.save_positions().await?;
        }
        Ok(())
    }

    pub async fn get_position(&self, token_address: &str) -> Option<Position> {
        self.positions.read().await.get(token_address).cloned()
    }

    pub async fn get_all_positions(&self) -> HashMap<String, Position> {
        self.positions.read().await.clone()
    }

    pub async fn update_token_data(
        &self,
        address: &str,
        metadata: Option<TokenMetadata>,
        price_point: Option<PricePoint>,
        onchain: Option<OnChainMetrics>,
        social: Option<SocialMetrics>,
    ) -> Result<()> {
        let mut data = self.token_data.write().await;
        
        let token_data = data.entry(address.to_string())
            .or_insert_with(|| TokenData {
                metadata: metadata.clone().unwrap_or_else(|| TokenMetadata {
                    address: address.to_string(),
                    symbol: String::new(),
                    name: String::new(),
                    decimals: 0,
                    price_usd: 0.0,
                    price_sol: 0.0,
                    volume_24h: 0.0,
                    market_cap: 0.0,
                    fully_diluted_market_cap: 0.0,
                    circulating_supply: 0.0,
                    total_supply: 0.0,
                }),
                price_history: Vec::new(),
                onchain_metrics: None,
                social_metrics: None,
                last_updated: Utc::now(),
            });

        if let Some(meta) = metadata {
            token_data.metadata = meta;
        }

        if let Some(price) = price_point {
            token_data.price_history.push(price);
            // Keep only last 24 hours of price points (assuming 1-minute intervals)
            if token_data.price_history.len() > 1440 {
                token_data.price_history.remove(0);
            }
        }

        if let Some(metrics) = onchain {
            token_data.onchain_metrics = Some(metrics);
        }

        if let Some(metrics) = social {
            token_data.social_metrics = Some(metrics);
        }

        token_data.last_updated = Utc::now();
        drop(data);

        self.save_token_data().await?;
        Ok(())
    }

    pub async fn add_market_snapshot(&self, trends: Vec<MarketTrend>) -> Result<()> {
        let mut snapshots = self.market_snapshots.write().await;
        
        snapshots.push(MarketSnapshot {
            timestamp: Utc::now(),
            trends,
        });

        while snapshots.len() > self.max_snapshots {
            snapshots.remove(0);
        }

        drop(snapshots);
        self.save_market_snapshots().await?;
        Ok(())
    }

    pub async fn get_token_data(&self, address: &str) -> Option<TokenData> {
        self.token_data.read().await.get(address).cloned()
    }

    pub async fn get_token_price_history(&self, address: &str) -> Option<Vec<PricePoint>> {
        self.token_data.read().await
            .get(address)
            .map(|data| data.price_history.clone())
    }

    pub async fn get_market_snapshots(&self, limit: Option<usize>) -> Vec<MarketSnapshot> {
        let snapshots = self.market_snapshots.read().await;
        match limit {
            Some(n) => snapshots.iter().rev().take(n).cloned().collect(),
            None => snapshots.clone(),
        }
    }

    pub async fn get_trending_tokens_history(&self) -> Vec<(DateTime<Utc>, Vec<String>)> {
        let snapshots = self.market_snapshots.read().await;
        snapshots.iter()
            .map(|snapshot| (
                snapshot.timestamp,
                snapshot.trends.iter()
                    .map(|trend| trend.token_address.clone())
                    .collect()
            ))
            .collect()
    }

    pub async fn analyze_token_momentum(&self, address: &str) -> Option<f64> {
        if let Some(data) = self.get_token_data(address).await {
            if data.price_history.len() < 2 {
                return None;
            }

            // Calculate price momentum over available history
            let price_changes: Vec<f64> = data.price_history.windows(2)
                .map(|window| {
                    let [prev, curr] = window else { unreachable!() };
                    (curr.price - prev.price) / prev.price
                })
                .collect();

            // Weight recent changes more heavily
            let weighted_sum: f64 = price_changes.iter()
                .enumerate()
                .map(|(i, change)| change * (i + 1) as f64)
                .sum();

            let weights_sum: f64 = (1..=price_changes.len()).sum::<usize>() as f64;
            
            Some(weighted_sum / weights_sum)
        } else {
            None
        }
    }

    pub async fn get_token_correlation(&self, token1: &str, token2: &str) -> Option<f64> {
        let (hist1, hist2) = match (
            self.get_token_price_history(token1).await,
            self.get_token_price_history(token2).await,
        ) {
            (Some(h1), Some(h2)) => (h1, h2),
            _ => return None,
        };

        if hist1.is_empty() || hist2.is_empty() {
            return None;
        }

        // Get overlapping time periods
        let start_time = hist1[0].timestamp.max(hist2[0].timestamp);
        let end_time = hist1.last().unwrap().timestamp.min(hist2.last().unwrap().timestamp);

        let prices1: Vec<f64> = hist1.iter()
            .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
            .map(|p| p.price)
            .collect();

        let prices2: Vec<f64> = hist2.iter()
            .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
            .map(|p| p.price)
            .collect();

        if prices1.len() < 2 || prices2.len() < 2 {
            return None;
        }

        // Calculate correlation coefficient
        let mean1 = prices1.iter().sum::<f64>() / prices1.len() as f64;
        let mean2 = prices2.iter().sum::<f64>() / prices2.len() as f64;

        let mut covariance = 0.0;
        let mut var1 = 0.0;
        let mut var2 = 0.0;

        for i in 0..prices1.len() {
            let diff1 = prices1[i] - mean1;
            let diff2 = prices2[i] - mean2;
            covariance += diff1 * diff2;
            var1 += diff1 * diff1;
            var2 += diff2 * diff2;
        }

        let correlation = covariance / (var1.sqrt() * var2.sqrt());
        Some(correlation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_data_storage() {
        let storage = MarketDataStorage::new(100);

        // Test token data storage
        let address = "test_token";
        let metadata = TokenMetadata {
            address: address.to_string(),
            symbol: "TEST".to_string(),
            name: "Test Token".to_string(),
            decimals: 9,
            price_usd: 1.0,
            price_sol: 0.01,
            volume_24h: 1000000.0,
            market_cap: 10000000.0,
            fully_diluted_market_cap: 20000000.0,
            circulating_supply: 1000000.0,
            total_supply: 2000000.0,
        };

        storage.update_token_data(
            address,
            Some(metadata.clone()),
            Some(PricePoint {
                timestamp: Utc::now(),
                price: 1.0,
                volume: 1000000.0,
            }),
            None,
            None,
        ).await.unwrap();

        let data = storage.get_token_data(address).await.unwrap();
        assert_eq!(data.metadata.symbol, "TEST");
        assert_eq!(data.price_history.len(), 1);
    }
} 