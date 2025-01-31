#[derive(Debug, Clone)]
pub struct MarketData {
    pub market_cap: f64,
    pub volatility: f64,
    pub volume_24h: f64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct MarketContext {
    pub market_trend: String,
    pub sector_performance: f64,
    pub sentiment_score: f64,
} 