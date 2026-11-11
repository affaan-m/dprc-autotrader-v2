use anyhow::Result;
use rig_solana_trader::{
    database::DatabaseClient,
    market_data::{MarketDataProvider, loaders::MarketDataLoader},
    strategy::{TradingStrategy, pipeline::TradingPipeline},
    execution::ExecutionEngine,
    agents::TradingAgentSystem,
};
use rig::providers::openai::Client as OpenAIClient;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .pretty()
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize OpenAI client
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let openai_client = OpenAIClient::new(&openai_api_key);
    let model = openai_client.completion_model("gpt-4");

    // Initialize database
    let mongodb_uri = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_client = DatabaseClient::new(&mongodb_uri, "solana_trader").await?;

    // Initialize market data components
    let market_data = MarketDataProvider::new(&openai_api_key, db_client.clone()).await?;
    let data_loader = MarketDataLoader::new();

    // Initialize trading components
    let strategy = TradingStrategy::new();
    let execution = ExecutionEngine::new(1.0); // 1% max slippage

    // Initialize trading pipeline
    let pipeline = TradingPipeline::new(market_data.clone(), strategy, execution);

    // Initialize multi-agent system
    let agents = TradingAgentSystem::new(model);

    // Test tokens
    let test_tokens = vec![
        "So11111111111111111111111111111111111111112", // Wrapped SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
    ];

    for token in test_tokens {
        info!("Processing token {}", token);

        // 1. Load and analyze market data
        let market_report = data_loader.load_market_report("data/market_reports/latest.txt").await?;
        let whitepaper = data_loader.load_token_whitepaper("data/whitepapers/token.pdf").await?;
        
        // 2. Get multi-agent analysis
        let token_data = format!(
            "Token: {}\nMarket Report:\n{}\nWhitepaper:\n{}",
            token, market_report, whitepaper
        );
        let decision = agents.make_trading_decision(&token_data).await?;
        info!("Agent decision: {}", decision);

        // 3. Execute through pipeline
        let tx_signature = pipeline.execute_trade(token.to_string()).await?;
        info!("Transaction executed: {}", tx_signature);
    }

    Ok(())
} 