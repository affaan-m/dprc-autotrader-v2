use rig_core::{
    agent::{Agent, AgentSystem},
    message_bus::MessageBus,
    storage::MongoVectorDB,
};
use rig_mongodb::VectorStorageConfig;
use rig_solana_trader::{
    agents::{DataIngestionAgent, DecisionAgent, ExecutionAgent, PredictionAgent, TwitterAgent},
    personality::StoicPersonality,
};
use std::sync::Arc;
use std::env;
use std::time::Duration;

mod data_ingestion;
mod prediction;
mod decision;
mod execution;
mod feedback;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize shared components
    let message_bus = MessageBus::new();
    let personality = Arc::new(StoicPersonality::new());
    
    // Configure MongoDB vector storage
    let vector_db = MongoVectorDB::new(VectorStorageConfig {
        uri: env::var("MONGODB_URI")?,
        database: "trader_agents",
        collections: vec![
            "market_data", 
            "trade_history",
            "risk_models",
            "sentiment_analysis"
        ],
    }).await?;

    // Create agent system
    let mut agent_system = AgentSystem::new()
        .with_retry_policy(3, Duration::from_secs(10))
        .with_health_check_interval(Duration::from_secs(30));

    // Add agents with their dependencies
    agent_system
        .add_agent(DataIngestionAgent::new(
            message_bus.clone(),
            vector_db.clone(),
            personality.clone(),
        ))
        .add_agent(PredictionAgent::new(
            message_bus.clone(),
            vector_db.clone(),
            personality.clone(),
        ))
        .add_agent(DecisionAgent::new(
            message_bus.clone(),
            vector_db.clone(),
            personality.clone(),
        ))
        .add_agent(ExecutionAgent::new(
            message_bus.clone(),
            vector_db.clone(),
            personality.clone(),
        ))
        .add_agent(TwitterAgent::new(
            message_bus.clone(),
            personality.clone(),
        ));

    // Start all agents
    agent_system.run().await?;
    
    Ok(())
}

async fn trading_loop(
    executor: Arc<SolanaExecutor>,
    risk_manager: Arc<RiskManager>,
    twitter: Arc<TwitterClient>,
) -> Result<()> {
    let market_client = MarketDataClient::new(env::var("PUMPFUN_API_KEY")?);
    
    loop {
        let token_data = market_client.get_token_data("TOKEN_MINT").await?;
        let analysis = TradeAnalysis {
            market_cap: token_data.current_market_cap,
            volume_ratio: token_data.buy_volume_4h / token_data.sell_volume_4h,
            risk_assessment: market_client.analyze_market(&token_data),
        };

        let action = TradeAction {
            action_type: TradeType::Buy,
            params: TradeParams {
                mint: "TOKEN_MINT".into(),
                amount: 0.1,
                slippage: 10,
                units: 1_000_000,
            },
            analysis: Some(analysis),
        };

        risk_manager.validate_trade(&action)?;
        
        let signature = executor.execute_trade(action.clone()).await?;
        twitter.post_trade(&action, &signature.to_string()).await?;

        tokio::time::sleep(Duration::from_secs(300)).await;
    }
} 