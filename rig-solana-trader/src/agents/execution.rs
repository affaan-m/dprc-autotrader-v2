use rig_core::{
    agent::Agent,
    message_bus::{Message, MessageBus},
    storage::VectorStorage,
};
use rig_solana_trader::{personality::StoicPersonality, trading::TradeExecution};
use solana_sdk::signature::Signature;
use std::sync::Arc;

pub struct ExecutionAgent {
    bus: MessageBus,
    storage: Arc<dyn VectorStorage>,
    personality: Arc<StoicPersonality>,
}

impl ExecutionAgent {
    pub fn new(
        bus: MessageBus,
        storage: Arc<dyn VectorStorage>,
        personality: Arc<StoicPersonality>,
    ) -> Self {
        Self { bus, storage, personality }
    }
}

#[async_trait]
impl Agent for ExecutionAgent {
    async fn run(&self) -> anyhow::Result<()> {
        let mut receiver = self.bus.subscribe("trade_decisions");
        
        while let Ok(msg) = receiver.recv().await {
            if let Message::TradeDecision(decision) = msg {
                // Execute trade on Solana
                let sig: Signature = self.personality.execute_trade(&decision).await?;
                
                // Store execution record
                let execution = TradeExecution {
                    tx_hash: sig.to_string(),
                    mint_address: decision.mint,
                    amount: decision.amount,
                    risk_assessment: decision.risk_score,
                    vector_embedding: decision.to_embedding(),
                    timestamp: Utc::now(),
                };
                
                self.storage
                    .insert("trade_history", execution)
                    .await?;

                self.bus.publish(Message::TradeExecuted(execution)).await;
            }
        }
        Ok(())
    }
} 