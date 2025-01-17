use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast::{self, Sender};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub token_address: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: i64,
}

pub struct MarketDataStream {
    price_updates: Sender<PriceUpdate>,
    watched_tokens: HashMap<String, String>, // token_address -> symbol
}

impl MarketDataStream {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            price_updates: tx,
            watched_tokens: HashMap::new(),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PriceUpdate> {
        self.price_updates.subscribe()
    }

    pub fn watch_token(&mut self, token_address: String, symbol: String) {
        self.watched_tokens.insert(token_address, symbol);
    }

    pub async fn stream_token_data(&self) -> Result<()> {
        let ws_url = Url::parse("wss://public-api.birdeye.so/socket")?;
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to price updates for watched tokens
        for token_address in self.watched_tokens.keys() {
            let subscribe_msg = serde_json::json!({
                "event": "subscribe",
                "channel": format!("price:{}", token_address),
            });
            write.send(Message::Text(subscribe_msg.to_string())).await?;
        }

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(update) = serde_json::from_str::<PriceUpdate>(&text) {
                        if let Err(e) = self.price_updates.send(update.clone()) {
                            error!("Failed to broadcast price update: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
} 