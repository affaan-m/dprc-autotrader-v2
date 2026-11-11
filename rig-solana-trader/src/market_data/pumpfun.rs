use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PumpFunMarketData {
    pub current_market_cap: f64,
    pub bonding_market_cap: f64,
    pub buy_volume_4h: f64,
    pub sell_volume_4h: f64,
}

pub struct MarketDataClient {
    client: Client,
    api_key: String,
}

impl MarketDataClient {
    const BASE_URL: &'static str = "https://api.pumpfunapi.org";

    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_token_data(&self, mint: &str) -> Result<PumpFunMarketData> {
        let response = self.client
            .get(&format!("{}/pumpfun/new/tokens", Self::BASE_URL))
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json::<PumpFunMarketData>()
            .await?;

        Ok(response)
    }

    pub fn analyze_market(&self, data: &PumpFunMarketData) -> f64 {
        let liquidity_ratio = data.bonding_market_cap / data.current_market_cap.max(1.0);
        let volume_ratio = data.buy_volume_4h / data.sell_volume_4h.max(1.0);
        
        liquidity_ratio * volume_ratio
    }
} 