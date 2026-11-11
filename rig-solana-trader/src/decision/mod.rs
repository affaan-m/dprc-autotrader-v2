use rig_core::message_bus::{MessageBus, Message};
use rig_solana_trader::personality::StoicPersonality;

pub struct PPODecisionAgent {
    message_bus: MessageBus,
    policy_network: PolicyNetwork,
    personality: StoicPersonality,
}

impl PPODecisionAgent {
    pub fn new(message_bus: MessageBus) -> Self {
        Self {
            message_bus,
            policy_network: PolicyNetwork::new(),
            personality: StoicPersonality::new()
        }
    }

    async fn decide_action(&mut self, state: &State) -> Action {
        // Combine LLM analysis with PPO
        let llm_analysis = self.personality.analyze_state(state).await;
        let ppo_action = self.policy_network.forward(state);
        
        // Risk management
        if state.risk_level > self.personality.risk_tolerance {
            return Action::Hold;
        }
        
        // Combine signals
        match (llm_analysis, ppo_action) {
            (Analysis::Buy, Action::Buy) => Action::Buy,
            (Analysis::Sell, Action::Sell) => Action::Sell,
            _ => Action::Hold
        }
    }
} 