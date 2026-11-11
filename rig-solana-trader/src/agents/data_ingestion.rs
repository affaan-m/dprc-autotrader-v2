use rig_core::{
    agent::Agent,
    message_bus::{Message, MessageBus},
    storage::VectorStorage,
};
use rig_solana_trader::{personality::StoicPersonality, storage::MarketData};
use std::sync::Arc;

pub struct DataIngestionAgent {
    bus: MessageBus,
    storage: Arc<dyn VectorStorage>,
    personality: Arc<StoicPersonality>,
}

impl DataIngestionAgent {
    pub fn new(
        bus: MessageBus,
        storage: Arc<dyn VectorStorage>,
        personality: Arc<StoicPersonality>,
    ) -> Self {
        Self { bus, storage, personality }
    }
}

#[async_trait]
impl Agent for DataIngestionAgent {
    async fn run(&self) -> anyhow::Result<()> {
        let mut receiver = self.bus.subscribe("market_data");
        
        while let Ok(msg) = receiver.recv().await {
            if let Message::MarketData(data) = msg {
                // Store raw data
                self.storage
                    .insert("market_data", data.to_embedding())
                    .await?;

                // Process with personality constraints
                let processed = self.personality.process_market_data(data).await?;
                
                // Store processed data
                self.storage
                    .insert("processed_market", processed.to_embedding())
                    .await?;

                // Publish to message bus
                self.bus.publish(Message::ProcessedMarketData(processed)).await;
            }
        }
        Ok(())
    }
} 