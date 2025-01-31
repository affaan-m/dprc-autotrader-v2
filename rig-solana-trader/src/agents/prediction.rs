use rig_core::{
    agent::Agent,
    message_bus::{Message, MessageBus},
    storage::VectorStorage,
};
use rig_solana_trader::{personality::StoicPersonality, storage::MarketData};
use std::sync::Arc;

pub struct PredictionAgent {
    bus: MessageBus,
    storage: Arc<dyn VectorStorage>,
    personality: Arc<StoicPersonality>,
}

impl PredictionAgent {
    pub fn new(
        bus: MessageBus,
        storage: Arc<dyn VectorStorage>,
        personality: Arc<StoicPersonality>,
    ) -> Self {
        Self { bus, storage, personality }
    }
}

#[async_trait]
impl Agent for PredictionAgent {
    async fn run(&self) -> anyhow::Result<()> {
        let mut receiver = self.bus.subscribe("processed_market");
        
        while let Ok(msg) = receiver.recv().await {
            if let Message::ProcessedMarketData(data) = msg {
                // Find similar historical patterns
                let similar = self.storage
                    .nearest("market_data", data.to_embedding(), 5)
                    .await?;

                // Generate prediction with risk constraints
                let prediction = self.personality
                    .generate_prediction(data, similar)
                    .await?;

                self.bus.publish(Message::Prediction(prediction)).await;
            }
        }
        Ok(())
    }
} 