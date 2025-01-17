use anyhow::Result;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use solana_sdk::signature::Keypair;
use solana_client::rpc_client::RpcClient;

use rig_solana_trader::{
    database::{Database, sync::DataSyncService},
    market_data::{AggregatedDataProvider, birdeye::BirdEyeProvider, DataProvider},
    twitter::TwitterClient,
    strategy::TradingStrategy,
    dex::jupiter::JupiterDex,
};
use rig::agent::AgentBuilder;
use rig::providers::openai::Client as OpenAIClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Initialize database
    let db = Arc::new(
        Database::new(
            &env::var("MONGODB_URI")?,
            &env::var("MONGODB_DB_NAME")?,
        ).await?
    );

    // Initialize data provider
    let birdeye = Arc::new(BirdEyeProvider::new(env::var("BIRDEYE_API_KEY")?)) as Arc<dyn DataProvider>;
    let data_provider = Arc::new(AggregatedDataProvider::new(vec![birdeye]));

    // Initialize Twitter client
    let twitter = Arc::new(TwitterClient::new(
        env::var("TWITTER_USERNAME")?,
        env::var("TWITTER_COOKIES")?,
    )?);

    // Initialize RPC client and wallet
    let rpc_url = env::var("SOLANA_RPC_URL")?;
    let rpc = RpcClient::new(rpc_url.clone());
    let wallet = Arc::new(Keypair::from_base58_string(&env::var("SOLANA_PRIVATE_KEY")?));

    // Initialize Jupiter DEX
    let dex = JupiterDex::new(
        &rpc_url,
        env::var("JUPITER_API_KEY")?,
        env::var("JUPITER_SLIPPAGE")?.parse()?,
    );

    // Initialize trading strategy with OpenAI agent
    let openai_api_key = env::var("OPENAI_API_KEY")?;
    let openai_client = OpenAIClient::new(&openai_api_key);
    let model = openai_client.completion_model(&env::var("MODEL_NAME")?);
    
    let agent = AgentBuilder::new(model)
        .preamble("\
            You are a sophisticated trading assistant that analyzes Solana token market data and makes trading decisions. \
            Your goals are to:\n\
            1. Identify tokens with strong momentum and growing volume\n\
            2. Analyze price movements and market trends\n\
            3. Make trading decisions with clear reasoning\n\
            4. Generate concise, informative tweets about trades\n\
            You should maintain a stoic and analytical tone, focusing on data-driven decisions rather than emotions.\
        ")
        .build();

    let trading_strategy = TradingStrategy::new(
        agent,
        env::var("MAX_POSITION_SIZE_SOL")?.parse()?,
        env::var("MIN_POSITION_SIZE_SOL")?.parse()?,
    );

    // Create sync service
    let sync_service = DataSyncService::new(
        db.clone(),
        data_provider,
        twitter,
        trading_strategy,
        dex,
        wallet.clone(),
        env::var("DATA_SYNC_INTERVAL_SECONDS")?.parse()?,
    );

    // Start sync service
    info!("Starting data sync service");
    sync_service.start().await?;

    Ok(())
} 