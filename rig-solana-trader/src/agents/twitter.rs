use rig_core::{
    agent::Agent,
    message_bus::{Message, MessageBus},
};
use rig_solana_trader::{personality::StoicPersonality, twitter::TwitterClient};
use std::sync::Arc;

pub struct TwitterAgent {
    bus: MessageBus,
    client: TwitterClient,
    personality: Arc<StoicPersonality>,
}

impl TwitterAgent {
    pub fn new(bus: MessageBus, personality: Arc<StoicPersonality>) -> Self {
        Self {
            bus,
            client: TwitterClient::new(),
            personality,
        }
    }
}

#[async_trait]
impl Agent for TwitterAgent {
    async fn run(&self) -> anyhow::Result<()> {
        let mut receiver = self.bus.subscribe("trade_executed");
        
        while let Ok(msg) = receiver.recv().await {
            if let Message::TradeExecuted(execution) = msg {
                let tweet = self.personality
                    .generate_trade_tweet(&execution)
                    .await?;

                self.client.post_tweet(&tweet).await?;
            }
        }
        Ok(())
    }
} 