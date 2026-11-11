use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use rig_core::message_bus::{MessageBus, Message};

pub struct SolanaIngestor {
    rpc_client: Arc<RpcClient>,
    message_bus: MessageBus,
}

impl SolanaIngestor {
    pub fn new(message_bus: MessageBus) -> Self {
        Self {
            rpc_client: Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com")),
            message_bus
        }
    }

    pub async fn run(self) {
        loop {
            let block = self.rpc_client.get_latest_blockhash().await.unwrap();
            let transactions = self.rpc_client.get_block(&block).await.unwrap();
            
            self.message_bus.publish(Message::BlockData {
                block_hash: block,
                transactions,
                timestamp: Utc::now()
            }).await;
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

pub struct SentimentAnalyzer {
    llm: Arc<dyn CompletionModel>,
    message_bus: MessageBus,
}

impl SentimentAnalyzer {
    pub fn new(message_bus: MessageBus) -> Self {
        Self {
            llm: Arc::new(DeepSeek::new()),
            message_bus
        }
    }

    pub async fn analyze(&self, text: &str) -> f32 {
        let prompt = format!("Analyze sentiment of this crypto-related text. Return only a number between -1 (negative) and 1 (positive): {}", text);
        self.llm.complete(&prompt).await
            .parse()
            .unwrap_or(0.0)
    }
} 